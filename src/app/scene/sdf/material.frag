uniform vec3 cameraPosition;
uniform mat4 viewProjection;
uniform mat4 modelMatrix;
uniform mat4 BVP;
uniform vec4 surfaceColorTint;

uniform sampler3D sdfTex0;// Distance (R), Color (GBA)
uniform sampler3D sdfTex1;// Material properties (RGB)
uniform vec3 sdfTexSize;
uniform vec3 sdfBoundsMin;
uniform vec3 sdfBoundsMax;
uniform float sdfLODDistBetweenSamples;

in vec3 pos;// Geometry hit position. The original mesh (before transformation) must be a cube from sdfBoundsMin to sdfBoundsMax.

layout (location = 0) out vec4 outColor;

// Sample the texture at the given index and position.
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

// Extract distance from raw SDF sample
float sdfSampleTex0Dist(vec4 raw) {
    // Apply a function to restore the distance (-inf, inf) from the texture [0, 1].
    // KEEP IN SYNC WITH CPU CODE!
    return raw.r - 1e-1;
}

// Extract RGB color from raw SDF sample.
vec3 sdfSampleTex0Color(vec4 raw) {
    return raw.gba;
}

// Extract material properties from raw SDF sample.
vec3 sdfSampleTex1MetallicRoughnessOcclusion(vec4 raw) {
    return raw.rgb;
}

/// Approximate the SDF's normal at the given position. From https://iquilezles.org/articles/normalsSDF/.
vec3 sdfNormal(vec3 p) {
    float h = 1./length(sdfTexSize / sdfLODDistBetweenSamples);
    const vec2 k = vec2(1, -1);
    return normalize(k.xyy*sdfSampleTex0Dist(sdfSampleRawInterp(0, p + k.xyy*h)) +
    k.yyx*sdfSampleTex0Dist(sdfSampleRawInterp(0, p + k.yyx*h)) +
    k.yxy*sdfSampleTex0Dist(sdfSampleRawInterp(0, p + k.yxy*h)) +
    k.xxx*sdfSampleTex0Dist(sdfSampleRawInterp(0, p + k.xxx*h)));
}

// How far away (underestimate) from the bounding box the position is.
float sdfOutOfBoundsDist(vec3 p) {
    float oobX = max(sdfBoundsMin.x - p.x, p.x - sdfBoundsMax.x);
    float oobY = max(sdfBoundsMin.y - p.y, p.y - sdfBoundsMax.y);
    float oobZ = max(sdfBoundsMin.z - p.z, p.z - sdfBoundsMax.z);
    return max(oobX, max(oobY, oobZ));
}

// Launch a ray against the SDF and return the hit position and raw tex0 sample.
// The fourth value of the hit position is the distance to the surface, -1 for out of steps and -2 for out of bounds.
vec4[2] sdfRaycast(vec3 rayPos, vec3 rayDir, int maxSteps) {
    vec4[2] hitPosAndSample = vec4[2](vec4(0.0), vec4(0.0));
    float distanceFromOrigin = 0.0;

    // The ray is casted until it hits the surface or a limit is reached.
    for (int i = 0; i < maxSteps; i++) {
        // Stop condition: out of steps
        if (i >= maxSteps-1) {
            hitPosAndSample[0] = vec4(rayPos, -1.0);
            break;
        }

        // Stop condition: out of bounds
        // NOTE: small epsilon to avoid pixel artifacts
        if (sdfOutOfBoundsDist(rayPos) > 1e-4) {
            hitPosAndSample[0] = vec4(rayPos, -2.0);
            break;
        }

        // The SDF is evaluated at the current position in the ray.
        vec4 sampleTex0Raw = sdfSampleRawInterp(0, rayPos);
        float sampleDist = sdfSampleTex0Dist(sampleTex0Raw);

        // Stop condition: actually hit the surface.
        // NOTE: floating point precision mitigation: use a small epsilon to avoid hitting the surface exactly.
        if (sampleDist < 1e-5) {
            hitPosAndSample[0] = vec4(rayPos, distanceFromOrigin);
            hitPosAndSample[1] = sampleTex0Raw;
            break;
        }

        // Move the ray forward by the minimum distance to the surface.
        distanceFromOrigin += sampleDist;
        rayPos += rayDir * sampleDist;
    }
    return hitPosAndSample;
}

void main() {
    // Find the starting point for the search
    // Default to starting from the hit position...
    vec3 rayOrigin = pos;
    vec3 rayDir = normalize(rayOrigin - cameraPosition);
    // ...but if the hit position + small step is out of bounds
    if (sdfOutOfBoundsDist(rayOrigin + rayDir * 0.2) > 0.0) {
        // we are inside the volume and should start from the camera's position (+ small step).
        rayOrigin = cameraPosition + rayDir * 0.2;
    }

    // Cast the ray against the SDF and return the hit position and sample.
    vec4[2] hitPosAndSample = sdfRaycast(rayOrigin, rayDir, 256);

    // Check for no hit (out of bounds or out of steps)
    if (hitPosAndSample[0].w < 0.0) {
        outColor = vec4(0.0, 0.0, 0.0, 0.0);// transparent
        gl_FragDepth = 1.0;
        return;
    }

    // Get the hit position and sample data.
    vec3 hitPos = hitPosAndSample[0].xyz;
    vec4 sampleTex0Raw = hitPosAndSample[1];
    vec4 sampleTex1Raw = sdfSampleRawInterp(1, hitPos);
    vec3 normal = sdfNormal(hitPos);

    // Read material properties from the texture color
    vec3 sampleColor = sdfSampleTex0Color(sampleTex0Raw);
    sampleColor *= surfaceColorTint.rgb;// Usually white, does nothing to the surface's color
    vec3 sampleProps = sdfSampleTex1MetallicRoughnessOcclusion(sampleTex1Raw);

    // Compute the color using the lighting model.
    outColor.rgb = calculate_lighting(cameraPosition, sampleColor, hitPos, normal, sampleProps.x, sampleProps.y, sampleProps.z);
    //outColor.rgb = sampleColor;

    // Apply tone mapping, color mapping and transparency.
    outColor.rgb = tone_mapping(outColor.rgb);
    outColor.rgb = color_mapping(outColor.rgb);
    outColor.a = surfaceColorTint.a;

#ifdef GAMMA_CORRECTION
    outColor.rgb = pow(outColor.rgb, vec3(GAMMA_CORRECTION));
#endif

    // FIXME: Circle artifacts when computing normals while looking straight at an object
//    outColor.rgb *= 0.001;
//    outColor.rgb += abs(normal);

    // Compute the depth to fix rendering order of multiple objects.
    vec4 hitPosProj = BVP * vec4(hitPos, 1.0);// TODO: Figure out how to set this...
    gl_FragDepth = hitPosProj.z / hitPosProj.w;
}
