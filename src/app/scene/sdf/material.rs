use cgmath::{vec3, Vector3};
use three_d::{Blend, Camera, Color, Cull, GeometryFunction, Light, LightingModel, lights_shader_source, Material, NormalDistributionFunction, RenderStates, Texture3D, ThreeDResult};
use three_d::core::Program;

/// The material properties used for the shader that renders the SDF. It can be applied to any mesh
/// with any transformation, which represents the bounding box of the SDF.
pub struct SDFViewerMaterial {
    /// The voxel data that defines the isosurface.
    pub voxels: Texture3D,
    /// The bounds in world space of the voxel data stored in the 3D texture.
    pub voxels_bounds: [Vector3<f32>; 2],
    /// See `SDFViewer::update`. Determines how many voxels should be used to define the isosurface.
    /// A value of n means that 2**n samples should be skipped in between each read.
    pub level_of_detail: usize,
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
            level_of_detail: 0,
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
    ) -> ThreeDResult<()> {
        for (i, light) in lights.iter().enumerate() {
            light.use_uniforms(program, i as u32)?;
        }

        program.use_uniform("cameraPosition", camera.position())?;
        program.use_uniform("surfaceColorTint", self.color)?;

        program.use_texture_3d("sdfTex", &self.voxels)?;
        program.use_uniform("sdfBoundsMin", self.voxels_bounds[0])?;
        program.use_uniform("sdfBoundsMax", self.voxels_bounds[1])?;
        program.use_uniform("sdfTexSize", vec3(
            self.voxels.width() as f32, self.voxels.height() as f32, self.voxels.depth() as f32))?;
        program.use_uniform("sdfLevelOfDetail", self.level_of_detail as u32)?;
        program.use_uniform("sdfThreshold", self.threshold)?;
        Ok(())
    }

    fn render_states(&self) -> RenderStates {
        RenderStates {
            blend: Blend::TRANSPARENCY, // TODO: breaks opaque surfaces if used anywhere?!
            cull: Cull::None, // Also draw the inside
            ..Default::default()
        }
    }

    fn is_transparent(&self) -> bool {
        false
    }
}


/// Utility to pack a 3D color to a single float. Keep in sync with GPU code!
pub fn pack_color(color: Vector3<f32>) -> f32 {
    color.x + color.y * 256.0 + color.z * 256.0 * 256.0
}

/// Utility to unpack a 3D color from a single float. Keep in sync with GPU code!
#[allow(dead_code)]
pub fn unpack_color(f: f32) -> Vector3<f32> {
    let mut color = Vector3::new(0., 0., 0.);
    color.y = (f / 256.0 / 256.0).floor();
    color.z = ((f - color.z * 256.0 * 256.0) / 256.0).floor();
    color.x = (f - color.z * 256.0 * 256.0 - color.y * 256.0).floor();
    // now we have a vec3 with the 3 components in range [0..255]. Let's normalize it!
    color / 255.0
}