use cgmath::{vec3, Vector3};
use three_d::{Blend, Camera, Color, Cull, GeometryFunction, Light, LightingModel, lights_shader_source, Material, MaterialType, NormalDistributionFunction, RenderStates, Texture3D};
use three_d::core::Program;

/// The material properties used for the shader that renders the SDF. It can be applied to any mesh
/// with any transformation, which represents the bounding box of the SDF.
pub struct SDFViewerMaterial {
    /// The voxel data that defines the isosurface.
    pub voxels: Texture3D,
    /// The bounds in world space of the voxel data stored in the 3D texture.
    pub voxels_bounds: [Vector3<f32>; 2],
    /// See `SDFViewer::update`. Determines how many voxels should be used to define the isosurface.
    /// A value of n means that n samples should be skipped in each dimension in between each read.
    pub lod_dist_between_samples: f32,
    /// Threshold (in the range [0..1]) that defines the surface in the voxel data.
    pub threshold: f32,
    /// Base surface color (tint). Assumed to be in linear color space.
    pub color: Color,
    /// The lighting model used to render the voxel data.
    pub lighting_model: LightingModel,
}

impl SDFViewerMaterial {
    pub fn new(voxels: Texture3D, voxels_bounds: [Vector3<f32>; 2]) -> Self {
        Self {
            voxels,
            voxels_bounds,
            lod_dist_between_samples: 1f32,
            threshold: 0.0,
            color: Color::WHITE,
            lighting_model: LightingModel::Cook(
                NormalDistributionFunction::TrowbridgeReitzGGX,
                GeometryFunction::SmithSchlickGGX),
        }
    }
}

impl Material for SDFViewerMaterial {
    fn fragment_shader_source(&self, _use_vertex_colors: bool, lights: &[&dyn Light]) -> String {
        let mut output = lights_shader_source(lights, self.lighting_model);
        output.push_str(include_str!("material.frag"));
        output
    }

    fn use_uniforms(
        &self,
        program: &Program,
        camera: &Camera,
        lights: &[&dyn Light],
    ) {
        for (i, light) in lights.iter().enumerate() {
            light.use_uniforms(program, i as u32);
        }

        program.use_uniform("cameraPosition", camera.position());
        // program.use_uniform("BVP", bvp_matrix(camera));
        program.use_uniform("surfaceColorTint", self.color);

        program.use_texture_3d("sdfTex", &self.voxels);
        program.use_uniform("sdfBoundsMin", self.voxels_bounds[0]);
        program.use_uniform("sdfBoundsMax", self.voxels_bounds[1]);
        program.use_uniform("sdfTexSize", vec3(
            self.voxels.width() as f32, self.voxels.height() as f32, self.voxels.depth() as f32));
        program.use_uniform("sdfLODDistBetweenSamples", self.lod_dist_between_samples);
        program.use_uniform("sdfThreshold", self.threshold);
    }

    fn render_states(&self) -> RenderStates {
        RenderStates {
            blend: Blend::TRANSPARENCY,
            cull: Cull::None, // Also draw the inside
            ..Default::default()
        }
    }

    fn material_type(&self) -> MaterialType {
        MaterialType::Transparent
    }
}

// Copied from https://github.com/asny/three-d/blob/9914fc1eb76dee2cb2a58dc781a59085bc413b10/src/renderer/light.rs#L143
// fn bvp_matrix(camera: &Camera) -> Mat4 {
//     let bias_matrix = Mat4::new(
//         0.5, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.5, 0.5, 0.5, 1.0,
//     );
//     bias_matrix * camera.projection() * camera.view()
// }

// Utility to pack a RGB ([0, 1]) color into a single float in the [0, 1] range.
// WARNING: GLSL highp floats are 24-bit long!
// WARNING: Keep in sync with GPU code!
// FIXME: Still some weird artifacts when changing colors in a gradient.
pub fn pack_color(color: Vector3<f32>) -> f32 {
    const PRECISION: f32 = 4.0;
    const PRECISION_P1: f32 = PRECISION + 1.0;
    let components = color.map(|c| c.min(1.0).max(0.0))
        .map(|c| (c * PRECISION + 0.5).floor());
    (components.x + components.y * PRECISION_P1 + components.z * PRECISION_P1 * PRECISION_P1)
        / (PRECISION_P1 * PRECISION_P1 * PRECISION_P1)
}

#[cfg(test)]
mod test {
    use cgmath::{MetricSpace, Vector3};

    use crate::app::scene::sdf::material::pack_color;

    pub fn unpack_color(packed: f32) -> Vector3<f32> {
        const PRECISION: f32 = 4.0;
        const PRECISION_P1: f32 = PRECISION + 1.0;
        let packed = packed * (PRECISION_P1 * PRECISION_P1 * PRECISION_P1);
        let x = (packed % PRECISION_P1) / PRECISION;
        let y = ((packed / PRECISION_P1).floor() % PRECISION_P1) / PRECISION;
        let z = (packed / (PRECISION_P1 * PRECISION_P1)).floor() / PRECISION;
        Vector3::new(x, y, z)
    }

    #[test]
    fn test_pack_color() {
        // Pack and unpack all colors performing basic sanity checks.
        const PRECISION: f32 = 4.0;
        let mut max_packed: f32 = 0.0;
        for x in 0..=255 {
            for y in 0..=255 {
                for z in 0..=255 {
                    let color = Vector3::new(x as f32 / 255.0, y as f32 / 255.0, z as f32 / 255.0);
                    let packed = pack_color(color);
                    max_packed = max_packed.max(packed);
                    let unpacked = unpack_color(packed);
                    // println!("{:?} --[{:?}]--> {:?}", color, packed, unpacked);
                    approx::assert_relative_eq!(color.distance(unpacked).abs(), 0.0, epsilon = 1.0 / PRECISION);
                }
            }
        }
        println!("Max packed value: {}", max_packed);
    }
}