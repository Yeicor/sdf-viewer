use std::str::FromStr;

use cgmath::{InnerSpace, MetricSpace, Vector3, Zero};

use crate::sdf::{SdfParameter, SdfParameterValue, SdfSample, SDFSurface};
use crate::sdf::demo::cube::{Material, RefCellMaterial};
use crate::sdf::demo::{RefCellBool, RefCellF32};

#[derive(clap::Parser, Debug, Clone)]
pub struct SDFDemoSphere {
    #[clap(short = 'l', long, default_value = "normal")]
    sphere_material: RefCellMaterial,
    #[clap(short, long, default_value = "1.05")]
    sphere_radius: RefCellF32,
    #[clap(skip)]
    changed: RefCellBool,
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
        let dist_sphere = p.distance(Vector3::zero()) - *self.sphere_radius.borrow();
        // Optimization: the air has no texture, so we can skip the texture lookup.
        distance_only = distance_only || dist_sphere > 0.1;
        if distance_only {
            SdfSample::new(dist_sphere, Vector3::zero())
        } else {
            self.sphere_material.borrow().render(dist_sphere, p, self.normal(p, None))
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

    //noinspection DuplicatedCode
    /// Optional: parameters.
    fn parameters(&self) -> Vec<SdfParameter> {
        vec![
            SdfParameter {
                name: "material".to_string(),
                value: SdfParameterValue::String {
                    value: self.sphere_material.to_string(),
                    choices: vec![
                        Material::Brick.to_string(),
                        Material::Normal.to_string(),
                    ],
                },
                description: "The material to use for the sphere.".to_string(),
            },
            SdfParameter {
                name: "sphere_radius".to_string(),
                value: SdfParameterValue::Float {
                    value: *self.sphere_radius.borrow(),
                    range: 0.0..=1.25,
                    step: 0.01,
                },
                description: "The radius of the sphere.".to_string(),
            },
        ]
    }

    //noinspection DuplicatedCode
    /// Optional: parameters.
    fn set_parameter(&self, param: &SdfParameter) -> Result<(), String> {
        if param.name == "sphere_radius" {
            if let SdfParameterValue::Float { value, .. } = &param.value {
                *self.sphere_radius.borrow_mut() = *value;
                *self.changed.borrow_mut() = true;
                return Ok(());
            }
        } else if param.name == "material" {
            if let SdfParameterValue::String { value, .. } = &param.value {
                *self.sphere_material.borrow_mut() = Material::from_str(value.as_str())
                    .expect("predefined choices, should not receive an invalid value");
                *self.changed.borrow_mut() = true;
                return Ok(());
            }
        }
        Err(format!("Unknown parameter {} with value {:?}", param.name, param.value))
    }

    //noinspection DuplicatedCode
    /// Optional: parameters.
    fn changed(&self) -> Option<[Vector3<f32>; 2]> {
        self.changed_default_impl().or_else(|| {
            // Note: bounding_box() change could be improved.
            let mut changed = self.changed.borrow_mut();
            if *changed {
                *changed = false;
                Some(self.bounding_box())
            } else { None }
        })
    }

    /// Optional: optimized normal computation for the sphere.
    fn normal(&self, p: Vector3<f32>, _eps: Option<f32>) -> Vector3<f32> {
        p.normalize()
    }
}
