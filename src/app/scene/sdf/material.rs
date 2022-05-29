use three_d::{Camera, Color, Light, LightingModel, Material, RenderStates, Texture3D, ThreeDResult, Vec3};
use three_d::core::Program;

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
    // TODO: pub lighting_model: LightingModel,
}

impl Material for SDFViewerMaterial {
    fn fragment_shader_source(&self, _use_vertex_colors: bool, _lights: &[&dyn Light]) -> String {
        todo!()
    }

    fn use_uniforms(&self, _program: &Program, _camera: &Camera, _lights: &[&dyn Light]) -> ThreeDResult<()> {
        todo!()
    }

    fn render_states(&self) -> RenderStates {
        todo!()
    }

    fn is_transparent(&self) -> bool {
        todo!()
    }
}
