use three_d::{MetricSpace, Vector3, Zero};

use crate::sdf::{SdfSample, SDFSurface};

/// An embedded demo `Sdf` implementation
pub struct SDFDemo {}

impl SDFSurface for SDFDemo {
    fn bounding_box(&self) -> [Vector3<f32>; 2] {
        [Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0)]
    }

    fn sample(&self, p: Vector3<f32>, _distance_only: bool) -> SdfSample {
        let dist_box = p.x.abs().max(p.y.abs()).max(p.z.abs()) - 0.99;
        let dist_sphere = p.distance(Vector3::zero()) - 0.99;
        SdfSample::new(dist_box.max(-dist_sphere))
    }
}