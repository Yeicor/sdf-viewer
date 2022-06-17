uniform vec3 cameraPosition;
uniform mat4 modelMatrix;// Geometry matrix.
uniform vec4 surfaceColorTint;

uniform sampler3D sdfTex;
uniform vec3 sdfTexSize;
uniform vec3 sdfBoundsMin;
uniform vec3 sdfBoundsMax;
uniform uint sdfLevelOfDetail;
uniform float sdfThreshold;

in vec3 pos;// Geometry hit position. The original mesh (before transformation) must be a cube from (0,0,0) to (1,1,1).

layout (location = 0) out vec4 outColor;

// FIXME: Cube seams visible from far away (on web only)?

// Utility to pack a 3D color to a single float. Keep in sync with CPU code!
float packColor(vec3 color) {
    return color.r + color.g * 256.0 + color.b * 256.0 * 256.0;
}

// Utility to unpack a 3D color from a single float. Keep in sync with CPU code!
vec3 unpackColor(float f) {
    vec3 color;
    color.b = floor(f / 256.0 / 256.0);
    color.g = floor((f - color.b * 256.0 * 256.0) / 256.0);
    color.r = floor(f - color.b * 256.0 * 256.0 - color.g * 256.0);
    // now we have a vec3 with the 3 components in range [0..255]. Let's normalize it!
    return color / 255.0;
}

// Same as sdfSampleRaw (below), but returns the nearest exact information as stored in the texture.
// It avoids unwanted interpolation which may break packed data.
vec4 sdfSampleRawNearest(vec3 p) {
    vec3 p01 = (p - sdfBoundsMin) / (sdfBoundsMax - sdfBoundsMin);
    vec3 roundSteps = sdfTexSize / pow(2.0, float(sdfLevelOfDetail));
    vec3 p01round = round(p01 * roundSteps) / roundSteps;
    return texture(sdfTex, p01round);
}

// Evaluate the SDF at the given position.
// Performs manual interpolation when sdfLevelOfDetail > 0.
vec4 sdfSampleRaw(vec3 p) {
    if (sdfLevelOfDetail == uint(0) || true) { // Automatic interpolation by the GPU!
        vec3 p01 = (p - sdfBoundsMin) / (sdfBoundsMax - sdfBoundsMin);
        return texture(sdfTex, p01);
    } else { // Manual interpolation, much more GPU-intensive but more accurate (while loading)
        // FIXME: Remove bugs ;) (and then remove the true above)
        vec3 p01 = (p - sdfBoundsMin) / (sdfBoundsMax - sdfBoundsMin);
        vec3 roundSteps = sdfTexSize / pow(2.0, float(sdfLevelOfDetail));
        vec3 p01round = round(p01 * roundSteps) / roundSteps;
        vec3 p2 = p01round * (sdfBoundsMax - sdfBoundsMin) + sdfBoundsMin;
        vec3 moveBy = (sdfBoundsMax - sdfBoundsMin) / roundSteps;
        vec3 p000 = p2;
        vec3 p001 = p2 + vec3(0.0, 0.0, moveBy.z);
        vec3 p010 = p2 + vec3(0.0, moveBy.y, 0.0);
        vec3 p011 = p2 + vec3(0.0, moveBy.y, moveBy.z);
        vec3 p100 = p2 + vec3(moveBy.x, 0.0, 0.0);
        vec3 p101 = p2 + vec3(moveBy.x, 0.0, moveBy.z);
        vec3 p110 = p2 + vec3(moveBy.x, moveBy.y, 0.0);
        vec3 p111 = p2 + moveBy;
        vec4 p000v = sdfSampleRawNearest(p000);
        vec4 p001v = sdfSampleRawNearest(p001);
        vec4 p010v = sdfSampleRawNearest(p010);
        vec4 p011v = sdfSampleRawNearest(p011);
        vec4 p100v = sdfSampleRawNearest(p100);
        vec4 p101v = sdfSampleRawNearest(p101);
        vec4 p110v = sdfSampleRawNearest(p110);
        vec4 p111v = sdfSampleRawNearest(p111);
        vec4 p00v = mix(p000v, p001v, p01.z);
        vec4 p01v = mix(p010v, p011v, p01.z);
        vec4 p10v = mix(p100v, p101v, p01.z);
        vec4 p11v = mix(p110v, p111v, p01.z);
        vec4 p0v = mix(p00v, p01v, p01.y);
        vec4 p1v = mix(p10v, p11v, p01.y);
        return mix(p0v, p1v, p01.x);
    }
}

/// Extract distance from raw SDF sample
float sdfSampleDist(vec4 raw) {
    return raw.r - sdfThreshold;
}

/// Extract RGB color from raw SDF sample.
vec3 sdfSampleColor(vec4 raw) {
    return unpackColor(raw.g);
}

/// Extract material properties from raw SDF sample.
vec3 sdfSampleMetallicRoughnessOcclussion(vec4 raw) {
    return unpackColor(raw.b);
}

/// Approximate the SDF's normal at the given position
vec3 sdfNormal(vec3 p) {
    const float eps = 0.001;// FIXME: Normals at inside-volume bounds (worth the extra performance hit?)
    // TODO(performance): Tetrahedron based normal calculation.
    float x = sdfSampleDist(sdfSampleRaw(p + vec3(eps, 0.0, 0.0))) - sdfSampleDist(sdfSampleRaw(p - vec3(eps, 0.0, 0.0)));
    float y = sdfSampleDist(sdfSampleRaw(p + vec3(0.0, eps, 0.0))) - sdfSampleDist(sdfSampleRaw(p - vec3(0.0, eps, 0.0)));
    float z = sdfSampleDist(sdfSampleRaw(p + vec3(0.0, 0.0, eps))) - sdfSampleDist(sdfSampleRaw(p - vec3(0.0, 0.0, eps)));
    return -normalize(vec3(x, y, z));
}

void main() {
    const int steps = 200;
    vec3 sdfBoundsSize = sdfBoundsMax - sdfBoundsMin;
    mat4 invModelMatrix = inverse(modelMatrix);

    // The ray origin in world? space.
    vec3 rayPos = (invModelMatrix*vec4(cameraPosition, 1.0)).xyz;
    // The ray direction in world space is given by the camera implementation.
    vec3 rayDir = normalize(pos - cameraPosition);
    // Start the ray from the camera position by default (optimization: start from bounds if outside).
    const float minDistFromCamera = 0.2;// FIXME: why are there artifacts with lower values?
    rayPos += minDistFromCamera * rayDir;

    // The ray is casted until it hits the surface or the maximum number of steps is reached.
    for (int i = 0; i < steps; i++) {
        // Stop condition: out of steps
        if (i == steps-1) {
            outColor = vec4(0.0, 0.0, 0.0, 0.0);// transparent
            break;
        }
        // Stop condition: out of bounds
        if (rayPos.x <= sdfBoundsMin.x || rayPos.y <= sdfBoundsMin.y || rayPos.z <= sdfBoundsMin.z ||
        rayPos.x >= sdfBoundsMax.x || rayPos.y >= sdfBoundsMax.y || rayPos.z >= sdfBoundsMax.z) {
            if (i == 0) {
                // Use the contact point on the box as the starting point (in world space)
                const float minDistFromBounds = 0.00001;
                rayPos = (invModelMatrix*vec4(pos, 1.0)).xyz;
                rayPos += minDistFromBounds * rayDir;
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
        vec4 sampleRaw = sdfSampleRaw(rayPos);
        float sampleDist = sdfSampleDist(sampleRaw);

        if (sampleDist <= 0.0) { // We hit the surface
            // Read material properties from the texture color
            vec3 normal = sdfNormal(rayPos);
            vec4 sampleRawNearest = sdfSampleRawNearest(rayPos);
            vec3 sampleColor = sdfSampleColor(sampleRawNearest);
            sampleColor *= surfaceColorTint.rgb; // Usually white, does nothing to the surface's color
            vec3 sampleProps = sdfSampleMetallicRoughnessOcclussion(sampleRawNearest);

            // Compute the color using the lighting model.
            outColor.rgb = calculate_lighting(cameraPosition, sampleColor, rayPos, normal, sampleProps.x, sampleProps.y, sampleProps.z);
            outColor.rgb = reinhard_tone_mapping(outColor.rgb);
            outColor.rgb = srgb_from_rgb(outColor.rgb);
            outColor.a = surfaceColorTint.a;

            // Compute the depth to fix rendering order of multiple objects.
            //float depth = length(cameraPosition - rayPos);
            //gl_FragDepth = 0.5;// TODO: Figure out how to set this...
            break;
        }
        rayPos += rayDir * sampleDist;
    }
}