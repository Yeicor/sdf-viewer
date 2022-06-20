use std::ops::RangeInclusive;

use auto_impl::auto_impl;
use eframe::egui;
use eframe::egui::Slider;
use eframe::egui::util::hash;
use three_d::{InnerSpace, Vector3};

pub mod demo;

/// CPU-side version of the SDF. It is the source from which to extract data to render on GPU.
/// It is fully queryable with read-only references for simplicity, (some extensions like parameters may require
/// interior mutability).
///
/// The chosen precision is `f32`, as it should be enough for rendering (shaders only require
/// 16 bits for high-precision variables, implementation-dependent).
#[auto_impl(&, & mut, Box, Rc, Arc)]
pub trait SDFSurface/*: SDFSurfaceClone*/ {
    // ============ REQUIRED CORE ============
    /// The bounding box of the SDF. Returns the minimum and maximum coordinates of the SDF.
    /// All operations MUST be inside this bounding box.
    fn bounding_box(&self) -> [Vector3<f32>; 2];

    /// Samples the surface at the given point. See `SdfSample` for more information.
    /// `distance_only` is a hint to the implementation that the caller only needs the distance.
    fn sample(&self, p: Vector3<f32>, distance_only: bool) -> SdfSample;

    // ============ OPTIONAL: HIERARCHY (perform the same operations on any sub-SDF) ============
    /// Returns the list of sub-SDFs that are directly children of this node.
    /// All returned children are references that share the lifetime of the parent.
    fn children(&self) -> Vec<&dyn SDFSurface> {
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

    /// Returns the list of parameters (including values and metadata) that can be modified on this SDF.
    fn parameters(&self) -> Vec<SdfParameter> {
        vec![]
    }

    /// Modifies the given parameter (only name and value.value matter here).
    /// If it returns ok, the sub-SDF hierarchy changed and it should be re-rendered.
    /// Implementations will probably need interior mutability to perform this.
    fn set_parameter(&self, _parameter: &SdfParameter) -> Result<(), String> {
        Err("no parameters implemented by default, overwrite this method".to_string())
    }

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
}

/// The metadata and current state of a parameter of a SDF.
#[derive(Debug, Clone)]
pub struct SdfParameter {
    /// The name of the parameter. Must be unique within the SDF.
    pub name: String,
    /// The current value of the parameter.
    pub value: SdfParameterValue,
    /// The user-facing description for the parameter.
    pub description: String,
}

impl SdfParameter {
    /// Build the GUI for the parameter. Returns true if the value was changed.
    pub fn gui(&mut self, ui: &mut egui::Ui) -> bool {
        ui.label(format!("{}:", self.name));
        let changed = match &mut self.value {
            SdfParameterValue::Boolean { value } => {
                ui.checkbox(value, value.to_string()).changed()
            }
            SdfParameterValue::Int { value, range, step } => {
                ui.add(Slider::new(value, range.clone()).step_by(*step as f64)).changed()
            }
            SdfParameterValue::Float { value, range, step } => {
                ui.add(Slider::new(value, range.clone()).step_by(*step as f64)).changed()
            }
            SdfParameterValue::String { value, choices } => {
                if choices.is_empty() {
                    ui.text_edit_multiline(value).changed()
                } else {
                    egui::ComboBox::new(hash(format!("sdf-param-{}", self.name)), value.clone())
                        .show_ui(ui, |ui| {
                            for choice in choices.iter() {
                                ui.selectable_value(value, choice.clone(), choice);
                            }
                        }).response.changed()
                }
            }
        };
        ui.end_row();
        changed
    }
}

/// The type, value, bounds and other type-specific metadata of a parameter.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum SdfParameterValue {
    Boolean {
        value: bool,
    },
    Int {
        value: i32,
        range: RangeInclusive<i32>,
        step: i32,
    },
    Float {
        value: f32,
        range: RangeInclusive<f32>,
        step: f32,
    },
    String {
        value: String,
        /// The available options to select from for the parameter. If empty, any string is valid.
        choices: Vec<String>,
    },
}