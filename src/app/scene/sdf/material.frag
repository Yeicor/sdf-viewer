uniform vec3 cameraPosition;
uniform mat4 modelMatrix;// Geometry matrix.
uniform vec4 surfaceColorTint;

uniform sampler3D sdfTex;
uniform vec3 sdfBoundsMin;
uniform vec3 sdfBoundsMax;
//uniform vec3 sdfTexInvSize;
// TODO: uniform uint sdfLevelOfDetail;
uniform float sdfThreshold;

in vec3 pos;// Geometry hit position. The original mesh (before transformation) must be a cube from (0,0,0) to (1,1,1).

layout (location = 0) out vec4 outColor;

// FIXME: Cube seams visible from far away (on web only)?

/// Evaluate the SDF at the given position. The position must be in the range ([0, 1], [0, 1], [0, 1]).
float sdfDist(vec3 p) {
    return texture(sdfTex, (p - sdfBoundsMin) / (sdfBoundsMax - sdfBoundsMin)).r - sdfThreshold;
}

vec3 sdfNormal(vec3 p) {
    const float eps = 0.0001;// FIXME: Normals at inside-volume bounds (worth the extra performance hit?)
    // TODO(performance): Tetrahedron based normal calculation.
    float x = sdfDist(p + vec3(eps, 0.0, 0.0)) - sdfDist(p - vec3(eps, 0.0, 0.0));
    float y = sdfDist(p + vec3(0.0, eps, 0.0)) - sdfDist(p - vec3(0.0, eps, 0.0));
    float z = sdfDist(p + vec3(0.0, 0.0, eps)) - sdfDist(p - vec3(0.0, 0.0, eps));
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
        float surfaceDistance = sdfDist(rayPos);
        if (surfaceDistance <= 0.0) { // We hit the surface
            // TODO: Read material properties from the texture
            // Compute the color using the lighting model.
            vec3 normal = sdfNormal(rayPos);
            outColor.rgb = calculate_lighting(cameraPosition, surfaceColorTint.rgb, rayPos, normal, 0.5, 0.5, 1.0);
            outColor.rgb = reinhard_tone_mapping(outColor.rgb);
            outColor.rgb = srgb_from_rgb(outColor.rgb);
            outColor.a = surfaceColorTint.a;
            // Compute the depth to fix rendering order of multiple objects.
            //float depth = length(cameraPosition - rayPos);
            //gl_FragDepth = 0.5;// TODO: Figure out how to set this...
            break;
        }
        rayPos += rayDir * surfaceDistance;// TODO: Multiply by scale transform?
    }
}