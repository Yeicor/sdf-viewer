use three_d::{Camera, CpuTexture3D, Light, Mat4, Material, Matrix4, RenderStates, ThreeDResult, Vector3};
use three_d::core::Program;
use material::SDFViewerMaterial;

pub mod material;

/// The main SDF viewer, capable of rendering an SDF for a `SDFViewerAppScene`.
#[allow(dead_code)] // TODO: remove
pub struct SDFViewer {
    /// The CPU side of the SDF texture, containing the distance to the surface and other metadata for each voxel.
    pub texture: CpuTexture3D,
    /// The material properties used for the shader that renders the SDF.
    pub material: SDFViewerMaterial,
    /// The transformation applied to the SDF in order to properly size and place it.
    /// Before transforming the SDF is centered on the origin, and has volume 1 (which is shared using
    /// the dimensions of the texture as weights).
    pub transform: Mat4,
}
