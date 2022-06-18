use auto_impl::auto_impl;
use three_d::{InnerSpace, Vector3};

pub mod demo;

/// CPU-side version of the SDF. It is the source from which to extract data to render on GPU.
/// It is mostly read-only, except for some extensions like parameters.
///
/// The chosen precision is `f32`, as it should be enough for rendering (shaders only require
/// 16 bits for high-precision variables, implementation-dependent).
#[auto_impl(&, & mut, Box, Rc, Arc)]
pub trait SDFSurface: SDFSurfaceClone {
    // ============ REQUIRED CORE ============
    /// The bounding box of the SDF. Returns the minimum and maximum coordinates of the SDF.
    /// All operations MUST be inside this bounding box.
    fn bounding_box(&self) -> [Vector3<f32>; 2];

    /// Samples the surface at the given point. See `SdfSample` for more information.
    /// `distance_only` is a hint to the implementation that the caller only needs the distance.
    fn sample(&self, p: Vector3<f32>, distance_only: bool) -> SdfSample;

    // ============ OPTIONAL: HIERARCHY (perform the same operations on any sub-SDF) ============
    /// Returns the list of sub-SDFs that are directly children of this node.
    /// All returned children are owned, as they are most likely shallow-copied references.
    fn children(&self) -> Vec<Box<dyn SDFSurface>> {
        vec![]
    }

    // An unique ID for this SDF.
    fn id(&self) -> usize {
        0
    }

    /// A nice display name for the SDF, which does not need to be unique in the hierarchy.
    fn name(&self) -> String {
        "Object".to_string()
    }

    // ============ OPTIONAL: PARAMETERS ============


    // ============ OPTIONAL: CUSTOM MATERIALS (GLSL CODE) ============


    // ============ OPTIONAL: UTILITIES ============
    /// Returns the normal at the given point.
    /// Default implementation is to approximate the normal from several samples.
    /// Note that the GPU will always use a predefined normal estimation algorithm.
    fn normal(&self, p: Vector3<f32>, eps: Option<f32>) -> Vector3<f32> {
        let eps = eps.unwrap_or(0.001);
        // Based on https://iquilezles.org/articles/normalsSDF/
        (Vector3::new(1., -1., -1.) * self.sample(p + Vector3::new(eps, -eps, -eps), true).distance +
            Vector3::new(-1., 1., -1.) * self.sample(p + Vector3::new(-eps, eps, -eps), true).distance +
            Vector3::new(-1., -1., 1.) * self.sample(p + Vector3::new(-eps, -eps, eps), true).distance +
            Vector3::new(1., 1., 1.) * self.sample(p + Vector3::new(eps, eps, eps), true).distance).normalize()
    }
}

/// From https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object.
///
/// Splitting SDFSurfaceClone into its own trait allows us to provide a blanket
/// implementation for all compatible types, without having to implement the
/// rest of SDFSurface.  In this case, we implement it for all types that have
/// 'static lifetime (*i.e.* they don't contain non-'static pointers), and
/// implement both SDFSurface and Clone.  Don't ask me how the compiler resolves
/// implementing SDFSurfaceClone for dyn SDFSurface when SDFSurface requires SDFSurfaceClone;
/// I have *no* idea why this works.
pub trait SDFSurfaceClone {
    /// Clone the SDF. It is assumed to be a cheap operation.
    fn clone_box(&self) -> Box<dyn SDFSurface>;
}

impl<T> SDFSurfaceClone for T where T: 'static + SDFSurface + Clone {
    fn clone_box(&self) -> Box<dyn SDFSurface> {
        Box::new(self.clone())
    }
}

// // We can now implement Clone manually by forwarding to clone_box.
// impl Clone for Box<dyn SDFSurface> {
//     fn clone(&self) -> Box<dyn SDFSurface> {
//         self.clone_box()
//     }
// }

/// The result of sampling the SDF at the given coordinates.
pub struct SdfSample {
    /// The signed distance to surface.
    pub distance: f32,

    // ============ OPTIONAL: MATERIAL PROPERTIES ============
    /// The RGB color of the sample.
    pub color: Vector3<f32>,
    /// The metallicness of the sample.
    pub metallic: f32,
    /// The roughness of the sample.
    pub roughness: f32,
    /// The occlusion of the sample.
    pub occlusion: f32,
}

impl SdfSample {
    /// Creates a new SDF sample using only distance and color. Use the struct initialization if you
    /// want to use other properties.
    pub fn new(distance: f32, color: Vector3<f32>) -> Self {
        Self { distance, color, metallic: 0.0, roughness: 0.0, occlusion: 0.0 }
    }

    // TODO: some procedural material defaults like checkerboard, wood, brick or ground.
}

