use three_d::{Blend, Camera, Color, Cull, Light, lights_shader_source, Material, RenderStates, Texture3D, ThreeDResult, Vec3, vec3};
use three_d::core::Program;
use three_d_asset::LightingModel;

/// The material properties used for the shader that renders the SDF.
pub struct SDFViewerMaterial {
    /// The voxel data that defines the isosurface.
    pub voxels: std::rc::Rc<Texture3D>,
    /// The size of the cube that is used to render the voxel data. The texture is scaled to fill the entire cube.
    pub size: Vec3,
    /// Threshold (in the range [0..1]) that defines the surface in the voxel data.
    pub threshold: f32,
    /// Base surface color (tint). Assumed to be in linear color space.
    pub color: Color,
    /// The lighting model used to render the voxel data.
    pub lighting_model: LightingModel,
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
        program.use_uniform("surfaceColor", self.color)?;
        program.use_uniform("size", self.size)?;
        program.use_uniform("threshold", self.threshold)?;
        program.use_uniform(
            "h",
            vec3(
                1.0 / self.voxels.width() as f32,
                1.0 / self.voxels.height() as f32,
                1.0 / self.voxels.depth() as f32,
            ),
        )?;
        program.use_texture_3d("tex", &self.voxels)
    }
    fn render_states(&self) -> RenderStates {
        RenderStates {
            blend: Blend::TRANSPARENCY,
            cull: Cull::None, // Render from the inside too.
            ..Default::default()
        }
    }
    fn is_transparent(&self) -> bool {
        true
    }
}
