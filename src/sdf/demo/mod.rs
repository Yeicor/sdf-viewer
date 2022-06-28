use std::cell::RefCell;
use std::ffi::OsString;
use std::num::ParseFloatError;
use std::ops::Deref;
use std::rc::Rc;
use std::str::{FromStr, ParseBoolError};

use cgmath::Vector3;

use crate::sdf::{SDFParam, SDFParamValue, SDFSample, SDFSurface};
use crate::sdf::demo::cube::SDFDemoCube;
use crate::sdf::demo::sphere::SDFDemoSphere;

#[cfg(feature = "sdfdemoffi")]
pub mod ffi;
pub mod cube;
pub mod sphere;

/// An embedded demo `Sdf` implementation to showcase/test most features. Subtracts a cube and a sphere.
#[derive(clap::Parser, Debug, Clone)]
pub struct SDFDemo {
    #[clap(flatten)]
    cube: SDFDemoCube,
    #[clap(flatten)]
    sphere: SDFDemoSphere,
    #[clap(short, long, default_value = "0.05")]
    max_distance_custom_material: RcRefCellF32,
    #[clap(short, long, default_value = "false")]
    disable_sphere: RcRefCellBool,
    #[clap(skip)]
    changed: RcRefCellBool,
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

    fn sample(&self, p: Vector3<f32>, distance_only: bool) -> SDFSample {
        // Compute the distance to the surface by subtracting a sphere to a cube.
        let sample_box = self.cube.sample(p, distance_only);
        if *self.disable_sphere.borrow() {
            sample_box
        } else {
            let sample_sphere = self.sphere.sample(p, distance_only);
            let dist = sample_box.distance.max(-sample_sphere.distance);
            // Choose the material based on which object's surface is closer.
            let inter_surface_dist = sample_box.distance.abs() - sample_sphere.distance.abs();
            let mut sample = if inter_surface_dist < 0.0 { sample_box } else { sample_sphere };
            if inter_surface_dist.abs() <= *self.max_distance_custom_material.borrow() {
                // - On the connection between the two original surfaces, force an specific material
                // let force = 1.0;// - inter_surface_dist.abs() / self.max_distance_custom_material;
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
    }

    /// Optional: hierarchy.
    fn children(&self) -> Vec<Box<dyn SDFSurface>> {
        // Important: cheap clone with shared references to parameters (to receive modifications)
        vec![Box::new(self.cube.clone()), Box::new(self.sphere.clone())]
    }

    /// Optional: hierarchy.
    fn id(&self) -> u32 {
        0
    }

    /// Optional: hierarchy.
    fn name(&self) -> String {
        "Demo".to_string()
    }

    /// Optional: parameters.
    fn parameters(&self) -> Vec<SDFParam> {
        vec![
            SDFParam {
                name: "max_distance_custom_material".to_string(),
                value: SDFParamValue::Float {
                    value: *self.max_distance_custom_material.borrow(),
                    range: 0.0..=0.25,
                    step: 0.01,
                },
                description: "The maximum distance between both surfaces at which the two materials are merged.".to_string(),
            },
            SDFParam {
                name: "disable_sphere".to_string(),
                value: SDFParamValue::Boolean {
                    value: *self.disable_sphere.borrow(),
                },
                description: "Whether to hide the sphere or not.".to_string(),
            },
        ]
    }

    /// Optional: parameters.
    fn set_parameter(&self, param: &SDFParam) -> Result<(), String> {
        if param.name == "max_distance_custom_material" {
            if let SDFParamValue::Float { value, .. } = param.value {
                *self.max_distance_custom_material.borrow_mut() = value;
                *self.changed.borrow_mut() = true;
                return Ok(());
            }
        } else if param.name == "disable_sphere" {
            if let SDFParamValue::Boolean { value, .. } = param.value {
                *self.disable_sphere.borrow_mut() = value;
                *self.changed.borrow_mut() = true;
                return Ok(());
            }
        }
        Err(format!("Unknown parameter {} with value {:?}", param.name, param.value))
    }

    //noinspection DuplicatedCode
    /// Optional: parameters.
    fn changed(&self) -> Option<[Vector3<f32>; 2]> {
        super::defaults::changed_default_impl(self).or_else(|| {
            // Note: bounding_box() change could be improved.
            let mut changed = self.changed.borrow_mut();
            if *changed {
                *changed = false;
                Some(self.bounding_box())
            } else { None }
        })
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

#[derive(Debug, Clone, Default)]
struct RcRefCellF32(Rc<RefCell<f32>>);

impl FromStr for RcRefCellF32 {
    type Err = ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        f32::from_str(s).map(|f| RcRefCellF32(Rc::new(RefCell::new(f))))
    }
}

impl Deref for RcRefCellF32 {
    type Target = RefCell<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Default)]
struct RcRefCellBool(Rc<RefCell<bool>>);

impl FromStr for RcRefCellBool {
    type Err = ParseBoolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        bool::from_str(s).map(|f| RcRefCellBool(Rc::new(RefCell::new(f))))
    }
}

impl Deref for RcRefCellBool {
    type Target = RefCell<bool>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}