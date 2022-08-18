uniform vec3 cameraPosition;
uniform mat4 modelMatrix;// Geometry matrix.
//uniform mat4 BVP;
uniform vec4 surfaceColorTint;

uniform sampler3D sdfTex0;// Distance (R), Color (GBA)
uniform sampler3D sdfTex1;// Material properties (RGB)
uniform vec3 sdfTexSize;
uniform vec3 sdfBoundsMin;
uniform vec3 sdfBoundsMax;
uniform float sdfLODDistBetweenSamples;
uniform float sdfThreshold;

in vec3 pos;// Geometry hit position. The original mesh (before transformation) must be a cube from (0,0,0) to (1,1,1).

layout (location = 0) out vec4 outColor;

float sdfOutOfBoundsDist(vec3 p) {
    const float eps = 0.00001;
    float res = max(
    max(sdfBoundsMin.x - p.x, p.x - sdfBoundsMax.x),
    max(max(sdfBoundsMin.y - p.y, p.y - sdfBoundsMax.y),
    max(sdfBoundsMin.z - p.z, p.z - sdfBoundsMax.z))
    );
    if (res >= 0.0) res += eps;// Is outside bounds -> avoid defining a surface at the bounds
    return res;
}

vec4 sdfTexSample(int sdfTexIndex, vec3 pos) {
    if (sdfTexIndex == 0) return texture(sdfTex0, pos);
    if (sdfTexIndex == 1) return texture(sdfTex1, pos);
    return vec4(0.0);
}

// Same as sdfSampleRaw (below), but returns the nearest exact information as stored in the texture.
// It avoids unwanted interpolation which may break packed data.
vec4 sdfSampleRawNearest(int sdfTexIndex, vec3 p) {
    //    float oobDist = sdfOutOfBoundsDist(p);
    //    if (oobDist >= 0) return vec4(oobDist, 0.0, 0.0, 0.0);// Out of bounds -> return distance to bounds
    vec3 p01 = (p - sdfBoundsMin) / (sdfBoundsMax - sdfBoundsMin);
    // Move from [0,1] to [0,sdfTexSize], rounding to nearest integer to find the neighbour and back to [0,1] range.
    // WARNING: This seems broken: loading has slightly wrong color samples. Relies on nearest neighbor interpolation set from CPU side!
    vec3 roundSteps = sdfTexSize / sdfLODDistBetweenSamples;
    vec3 p01nearestExact = round(p01 * roundSteps) / roundSteps;
    return sdfTexSample(sdfTexIndex, p01nearestExact);
}

// Evaluate the SDF at the given position, interpolating all values (interpreted as 4 floats) from the nearest available samples.
// NOTE: While loading (sdfLODDistBetweenSamples > 1) it gives a rough estimate of the SDF value, which is invalid for SDF distance
// so it performs a (buggy) blocky (but holeless) render. The correct way would be to perform manual interpolation of
// non-contiguous sdfTex values, but that is too GPU-intensive and slows down the loading process.
vec4 sdfSampleRawInterp(int sdfTexIndex, vec3 p) {
    //    float oobDist = sdfOutOfBoundsDist(p);
    //    if (oobDist >= 0) return vec4(oobDist, 0.0, 0.0, 0.0);// Out of bounds -> return distance to bounds
    if (sdfLODDistBetweenSamples == 1.0) { // Automatic interpolation by the GPU!
        vec3 p01 = (p - sdfBoundsMin) / (sdfBoundsMax - sdfBoundsMin);
        return sdfTexSample(sdfTexIndex, p01);
    } else { // While loading (see function docs)
        // Note that in order to avoid holes in the render, we are only incrementing the detail once the next layer is
        // available. This means that the user won't see the detail layer being populated in real-time.
        // TODO: Why does this not happen for the last layer?
        return sdfSampleRawNearest(sdfTexIndex, p);
        // TODO: Try to perform fast interpolation?
    }
}

/// Extract distance from raw SDF sample
float sdfSampleTex0Dist(vec4 raw) {
    return raw.r - sdfThreshold;
}

/// Extract RGB color from raw SDF sample.
vec3 sdfSampleTex0Color(vec4 raw) {
    return raw.gba;
}

/// Extract material properties from raw SDF sample.
vec3 sdfSampleTex1MetallicRoughnessOcclussion(vec4 raw) {
    return raw.rgb;
}

/// Approximate the SDF's normal at the given position. From https://iquilezles.org/articles/normalsSDF/.
vec3 sdfNormal(vec3 p) {
    // FIXME: Normals at inside-volume bounds (worth the slower performance?)
    float h = 1./length(sdfTexSize / sdfLODDistBetweenSamples);
    const vec2 k = vec2(1, -1);
    return normalize(k.xyy*sdfSampleTex0Dist(sdfSampleRawInterp(0, p + k.xyy*h)) +
    k.yyx*sdfSampleTex0Dist(sdfSampleRawInterp(0, p + k.yyx*h)) +
    k.yxy*sdfSampleTex0Dist(sdfSampleRawInterp(0, p + k.yxy*h)) +
    k.xxx*sdfSampleTex0Dist(sdfSampleRawInterp(0, p + k.xxx*h)));
}

void main() {
    const int steps = 400;
    vec3 sdfBoundsSize = sdfBoundsMax - sdfBoundsMin;
    mat4 invModelMatrix = inverse(modelMatrix);

    // The ray origin in world? space.
    vec3 rayPos = (invModelMatrix*vec4(cameraPosition, 1.0)).xyz;
    // The ray direction in world space is given by the camera implementation.
    vec3 rayDir = normalize(pos - cameraPosition);
    // Start the ray from the camera position by default (optimization: start from bounds if outside).
    const float minDistFromCamera = 0.2;
    rayPos += minDistFromCamera * rayDir;

    // The ray is casted until it hits the surface or the maximum number of steps is reached.
    for (int i = 0; i < steps; i++) {
        // Stop condition: out of steps
        if (i == steps-1) {
            outColor = vec4(0.0, 0.0, 0.0, 0.0);// transparent
            break;
        }
        // Stop condition: out of bounds
        if (sdfOutOfBoundsDist(rayPos) >= 0.0) {
            if (i == 0) {
                // Use the contact point on the box as the starting point (in world space)
                const float minDistFromBounds = 0.00001;
                rayPos = (invModelMatrix*vec4(pos, 1.0)).xyz;
                rayPos += minDistFromBounds * rayDir;
                continue;// This fixes the bug where if the surface touches the bounds it overlays everything else (why?!).
            } else {
                // Debug the number of steps and bounds: will break rendering order
                //                outColor = vec4(float(i)/float(steps), 0.0, 0.0, 0.25);
                // Output an transparent color and infinite depth
                outColor = vec4(0.0, 0.0, 0.0, 0.0);
                gl_FragDepth = 1.0;
                break;
            }
        }
        // The SDF is evaluated at the current position in the ray.
        vec4 sampleTex0Raw = sdfSampleRawInterp(0, rayPos);
        float sampleDist = sdfSampleTex0Dist(sampleTex0Raw);
        // FIXME: Some samples pass through the surface near interpolated corners, leading to single-pixel holes!

        if (sampleDist <= 0.0) { // We hit the surface
            // Read material properties from the texture color
            vec3 normal = sdfNormal(rayPos);
            vec4 sampleTex1Raw = sdfSampleRawInterp(1, rayPos);
            vec3 sampleColor = sdfSampleTex0Color(sampleTex0Raw);
            sampleColor *= surfaceColorTint.rgb;// Usually white, does nothing to the surface's color
            vec3 sampleProps = sdfSampleTex1MetallicRoughnessOcclussion(sampleTex1Raw);

            // Compute the color using the lighting model.
            outColor.rgb = calculate_lighting(cameraPosition, sampleColor, rayPos, normal, sampleProps.x, sampleProps.y, sampleProps.z);
            outColor.rgb = reinhard_tone_mapping(outColor.rgb);
            outColor.rgb = srgb_from_rgb(outColor.rgb);
            outColor.a = surfaceColorTint.a;

            // Compute the depth to fix rendering order of multiple objects.
            //vec4 rayPosProj = BVP * vec4(pos, 1.0); // TODO: Figure out how to set this...
            //gl_FragDepth = rayPosProj.z / rayPosProj.w;
            break;
        }
        rayPos += rayDir * sampleDist;
    }
}