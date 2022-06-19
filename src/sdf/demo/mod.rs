use std::ffi::OsString;

use cgmath::Vector3;

use crate::sdf::{SdfSample, SDFSurface};
use crate::sdf::demo::cube::SDFDemoCubeBrick;
use crate::sdf::demo::sphere::SDFDemoSphere;

pub mod cube;
pub mod sphere;

/// An embedded demo `Sdf` implementation to showcase/test most features. Subtracts a cube and a sphere.
#[derive(clap::Parser, Debug, Clone)]
pub struct SDFDemo {
    #[clap(flatten)]
    cube: SDFDemoCubeBrick,
    #[clap(flatten)]
    sphere: SDFDemoSphere,
}

impl Default for SDFDemo {
    fn default() -> Self {
        use clap::Parser;
        Self::parse_from::<_, OsString>([])
    }
}

impl SDFSurface for SDFDemo {
    fn bounding_box(&self) -> [Vector3<f32>; 2] {
        [Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0)]
    }

    fn sample(&self, p: Vector3<f32>, distance_only: bool) -> SdfSample {
        // Compute the distance to the surface by subtracting a sphere to a cube.
        let sample_box = self.cube.sample(p, distance_only);
        let sample_sphere = self.sphere.sample(p, distance_only);
        let dist = sample_box.distance.max(-sample_sphere.distance);
        // Choose the material based on which object's surface is closer.
        let inter_surface_dist = sample_box.distance.abs() - sample_sphere.distance.abs();
        let mut sample = if inter_surface_dist < 0.0 { sample_box } else { sample_sphere };
        const GRADIENT_SIZE: f32 = 0.05;
        if inter_surface_dist.abs() <= GRADIENT_SIZE {
            // - On the connection between the two original surfaces, force an specific material
            // let force = 1.0;// - inter_surface_dist.abs() / GRADIENT_SIZE;
            // println!("Force: {}", force);
            sample.color = Vector3::new(0.5, 0.6, 0.7);// * force + sample.color * (1.0 - force);
            sample.metallic = 0.5;// + sample.metallic * (1.0 - force);
            sample.roughness = 0.0;// + sample.roughness * (1.0 - force);
            sample.occlusion = 0.0;// + sample.occlusion * (1.0 - force);
        }
        // Overwrite the sample with the combined distance.
        sample.distance = dist;
        sample
    }

    /// Optional: hierarchy.
    fn children(&self) -> Vec<&dyn SDFSurface> {
        vec![&self.cube, &self.sphere]
    }

    /// Optional: hierarchy.
    fn id(&self) -> usize {
        0
    }

    /// Optional: hierarchy.
    fn name(&self) -> String {
        "Demo".to_string()
    }

    /// Optional: optimized normal computation for the difference.
    fn normal(&self, p: Vector3<f32>, eps: Option<f32>) -> Vector3<f32> {
        // Return the normal of the closest surface.
        let sample_box = self.cube.sample(p, true);
        let sample_sphere = self.sphere.sample(p, true);
        if sample_box.distance.abs() < sample_sphere.distance.abs() {
            self.cube.normal(p, eps)
        } else {
            -self.sphere.normal(p, eps) // Negated!
        }
    }
}
