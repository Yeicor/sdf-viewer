use three_d::{Blend, Camera, Color, Light, lights_shader_source, Material, RenderStates, Texture3D, ThreeDResult, vec3};
use three_d::core::Program;
use three_d_asset::LightingModel;

/// The material properties used for the shader that renders the SDF.
pub struct SDFViewerMaterial {
    /// The voxel data that defines the isosurface.
    pub voxels: Texture3D,
    /// See `SDFViewer::update`. Determines how many voxels should be used to define the isosurface.
    /// A value of n means that 2**n samples are skipped in between each read.
    pub level_of_detail: usize,
    /// Threshold (in the range [0..1]) that defines the surface in the voxel data.
    pub threshold: f32,
    /// Base surface color (tint). Assumed to be in linear color space.
    pub color: Color,
    /// The lighting model used to render the voxel data.
    pub lighting_model: LightingModel,
}

impl SDFViewerMaterial {
    pub fn new(voxels: Texture3D) -> Self {
        Self { voxels, level_of_detail: 0, threshold: 0.0, color: Color::WHITE, lighting_model: LightingModel::Blinn }
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
        program.use_uniform("sdfTexInvSize", vec3(
            1.0 / self.voxels.width() as f32,
            1.0 / self.voxels.height() as f32,
            1.0 / self.voxels.depth() as f32,
        ))?;
        // program.use_uniform("sdfLevelOfDetail", self.level_of_detail as u32)?;
        program.use_uniform("sdfThreshold", self.threshold)?;
        Ok(())
    }

    fn render_states(&self) -> RenderStates {
        RenderStates {
            blend: Blend::TRANSPARENCY,
            ..Default::default()
        }
    }

    fn is_transparent(&self) -> bool {
        false
    }
}
