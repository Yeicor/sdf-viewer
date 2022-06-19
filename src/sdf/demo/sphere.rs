use cgmath::{InnerSpace, MetricSpace, Vector3, Zero};
use crate::sdf::{SdfSample, SDFSurface};

#[derive(clap::Parser, Debug, Clone)]
pub struct SDFDemoSphere {
    #[clap(short, long, default_value = "1.05")]
    sphere_radius: f32,
}

impl Default for SDFDemoSphere {
    fn default() -> Self {
        use clap::Parser;
        use std::ffi::OsString;
        Self::parse_from::<_, OsString>([])
    }
}

impl SDFSurface for SDFDemoSphere {
    fn bounding_box(&self) -> [Vector3<f32>; 2] {
        [Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0)]
    }

    fn sample(&self, p: Vector3<f32>, mut distance_only: bool) -> SdfSample {
        // Compute the distance to the surface.
        let dist_sphere = p.distance(Vector3::zero()) - self.sphere_radius;
        // Optimization: the air has no texture, so we can skip the texture lookup.
        distance_only = distance_only || dist_sphere > 0.1;
        if distance_only {
            SdfSample::new(dist_sphere, Vector3::zero())
        } else {
            // Simple color gradient texture for the sphere based on normals.
            let color = self.normal(p, None).map(|n| n.abs());
            // info!("Sphere normal color: {:?}", color);
            SdfSample::new(dist_sphere, color)
        }
    }

    /// Optional: hierarchy.
    fn id(&self) -> usize {
        2
    }

    /// Optional: hierarchy.
    fn name(&self) -> String {
        "DemoSphere".to_string()
    }

    /// Optional: optimized normal computation for the sphere.
    fn normal(&self, p: Vector3<f32>, _eps: Option<f32>) -> Vector3<f32> {
        p.normalize()
    }
}
