use cgmath::{vec3, Vector3};
use three_d::{Blend, Camera, Color, Cull, FragmentAttributes, FragmentShader, Light, LightingModel, lights_shader_source, Material, MaterialType, RenderStates, Texture3D};
use three_d::core::Program;

/// The material properties used for the shader that renders the SDF. It can be applied to any mesh
/// with any transformation, which represents the bounding box of the SDF.
pub struct SDFViewerMaterial {
    /// For each voxel, contains the distance in the red channel, and the color in the other channels.
    pub tex0: Texture3D,
    /// For each voxel, Contains the material properties.
    pub tex1: Texture3D,
    /// The bounds in world space of the voxel data stored in the 3D texture.
    pub voxels_bounds: [Vector3<f32>; 2],
    /// See `SDFViewer::update`. Determines how many voxels should be used to define the isosurface.
    /// A value of n means that n samples should be skipped in each dimension in between each read.
    pub lod_dist_between_samples: f32,
    /// Base surface color (tint). Assumed to be in linear color space.
    pub color: Color,
    /// The lighting model used to render the voxel data.
    pub lighting_model: LightingModel,
}

impl SDFViewerMaterial {
    pub fn new(tex0: Texture3D, tex1: Texture3D, voxels_bounds: [Vector3<f32>; 2]) -> Self {
        Self {
            tex0,
            tex1,
            voxels_bounds,
            lod_dist_between_samples: 1f32,
            color: Color::WHITE,
            lighting_model: LightingModel::Phong,
        }
    }
}

impl Material for SDFViewerMaterial {
    fn fragment_shader(&self, lights: &[&dyn Light]) -> FragmentShader {
        let mut output = lights_shader_source(lights, self.lighting_model);
        output.push_str(include_str!("material.frag"));
        FragmentShader {
            source: output,
            attributes: FragmentAttributes {
                position: true,
                normal: false,
                tangents: false,
                uv: false,
                color: false,
            },
        }
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
        program.use_uniform("BVP", bvp_matrix(camera));
        program.use_uniform("surfaceColorTint", self.color);

        program.use_texture_3d("sdfTex0", &self.tex0);
        program.use_texture_3d("sdfTex1", &self.tex1);
        program.use_uniform("sdfBoundsMin", self.voxels_bounds[0]);
        program.use_uniform("sdfBoundsMax", self.voxels_bounds[1]);
        program.use_uniform("sdfTexSize", vec3(
            self.tex0.width() as f32, self.tex0.height() as f32, self.tex0.depth() as f32));
        program.use_uniform("sdfLODDistBetweenSamples", self.lod_dist_between_samples);
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
fn bvp_matrix(camera: &Camera) -> three_d::Mat4 {
    let bias_matrix = three_d::Mat4::new(
        0.5, 0.0, 0.0, 0.0,
        0.0, 0.5, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.5, 0.5, 0.5, 1.0,
    );
    bias_matrix * camera.projection() * camera.view()
}
