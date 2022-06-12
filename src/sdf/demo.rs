use three_d::{MetricSpace, Vector3};

use crate::sdf::{SdfSample, SDFSurface};

/// An integrated demo `Sdf` implementation
pub struct SDFDemo {}

impl SDFSurface for SDFDemo {
    fn bounding_box(&self) -> [Vector3<f32>; 2] {
        [Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0)]
    }

    fn sample(&self, p: Vector3<f32>, _distance_only: bool) -> SdfSample {
        let dist_sphere = p.distance(Vector3::new(0.5, 0.5, 0.5)) - 0.25;
        SdfSample::new(dist_sphere)
    }
}