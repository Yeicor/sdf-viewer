use std::ops::RangeInclusive;

use auto_impl::auto_impl;
use cgmath::Vector3;

#[cfg(feature = "sdfdemo")]
pub mod demo;

#[cfg(feature = "app")] // Main WebAssembly SDF implementations only available for the app
pub mod wasm;

#[cfg(feature = "meshers")]
pub mod meshers;

// TODO: Native library external SDF implementation?
// TODO: Remote REST API SDF implementation?

pub mod defaults;

/// CPU-side version of the SDF. It is the source from which to extract data to render on GPU.
/// It is fully queryable with read-only references for simplicity.
///
/// You only need to implement the required core methods, and can override all other default method
/// implementations to provide more functionality for your SDF. The default methods also show how to
/// provide a default SDF implementation in a different language.
///
/// The chosen precision is `f32`, as it should be enough for rendering (shaders only require
/// 16 bits for high-precision variables, implementation-dependent).
#[auto_impl(&, & mut, Box, Rc, Arc)]
pub trait SDFSurface {
    // ============ REQUIRED CORE ============
    /// The bounding box of the SDF. Returns the minimum and maximum coordinates of the SDF.
    /// All operations MUST be inside this bounding box.
    fn bounding_box(&self) -> [Vector3<f32>; 2];

    // TODO: Batched sampling to speed up operations for a possible REST API.
    /// Samples the surface at the given point. It should include the effect of all of its children
    /// and none of its parents. See [`SDFSample`] for more information.
    /// `distance_only` is a hint to the implementation that the caller only needs the distance.
    fn sample(&self, p: Vector3<f32>, distance_only: bool) -> SDFSample;

    // ============ OPTIONAL: HIERARCHY (perform the same operations on any sub-SDF) ============
    /// Returns the list of sub-SDFs that are directly children of this node.
    /// Note that modifications to the parameters of the returned children MUST affect this node.
    fn children(&self) -> Vec<Box<dyn SDFSurface>> {
        defaults::children_default_impl(self)
    }

    /// A unique ID within this SDF hierarchy.
    fn id(&self) -> u32 {
        defaults::id_default_impl(self)
    }

    /// A nice display name for the SDF, which does not need to be unique in the hierarchy.
    fn name(&self) -> String {
        defaults::name_default_impl(self)
    }

    // ============ OPTIONAL: PARAMETERS ============

    /// Returns the list of parameters (including values and metadata) that can be modified on this SDF.
    fn parameters(&self) -> Vec<SDFParam> {
        defaults::parameters_default_impl(self)
    }

    /// Modifies the given parameter. The value must be valid for the reported type (same kind and within allowed values)
    /// Implementations will probably need interior mutability to perform this.
    /// Use [`changed`](#method.changed) to notify what part of the SDF needs to be updated.
    fn set_parameter(&self, param_id: u32, param_value: &SDFParamValue) -> Result<(), String> {
        defaults::set_parameter_default_impl(self, param_id, param_value)
    }

    /// Returns the bounding box that was modified since [`changed`](#method.changed) was last called.
    /// It should also report if the children of this SDF need to be updated.
    /// This may happen due to a parameter change ([`set_parameter`](#method.set_parameter)) or any
    /// other event that may have changed the SDF. It should delimit as much as possible the part of the
    /// SDF that should be updated to improve performance.
    ///
    /// Multiple changes should be merged into a single bounding box or queued and returned in several
    /// [`changed`](#method.changed) calls for a similar effect.
    /// After returning Some(...) the implementation should assume that it was updated and no longer
    /// notify of that change (to avoid infinite loops).
    /// This function is called very frequently, so it should be very fast to avoid delaying frames.
    fn changed(&self) -> Option<[Vector3<f32>; 2]> {
        defaults::changed_default_impl(self)
    }

    // ============ OPTIONAL: CUSTOM MATERIALS (GLSL CODE) ============


    // ============ OPTIONAL: UTILITIES ============
    /// Returns the normal at the given point.
    /// Default implementation is to approximate the normal from several samples.
    /// Note that the GPU will always use a predefined normal estimation algorithm.
    fn normal(&self, p: Vector3<f32>, eps: Option<f32>) -> Vector3<f32> {
        defaults::normal_default_impl(self, p, eps)
    }
}

/// The result of sampling the SDF at the given coordinates.
#[repr(C)]
pub struct SDFSample {
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

impl SDFSample {
    /// Creates a new SDF sample using only distance and color. Use the struct initialization if you
    /// want to use other properties.
    pub fn new(distance: f32, color: Vector3<f32>) -> Self {
        Self { distance, color, metallic: 0.0, roughness: 0.0, occlusion: 0.0 }
    }
}

/// The metadata and current state of a parameter of a SDF.
#[derive(Debug, Clone)]
pub struct SDFParam {
    /// The ID of the parameter. Must be unique within this SDF (not necessarily within the SDF hierarchy).
    pub id: u32,
    /// The name of the parameter.
    pub name: String,
    /// The type definition for the parameter.
    pub kind: SDFParamKind,
    /// The current value of the parameter. MUST be of the same kind as the type definition.
    pub value: SDFParamValue,
    /// The user-facing description for the parameter.
    pub description: String,
}

/// The type, including bounds, choices or other type-specific metadata of a parameter.
#[derive(Debug, Clone)]
pub enum SDFParamKind {
    // No parameters required for booleans
    Boolean,
    Int {
        /// The range (inclusive) that must contain the value.
        range: RangeInclusive<i32>,
        /// The step size for the slider.
        step: i32,
    },
    Float {
        /// The range (inclusive) that must contain the value.
        range: RangeInclusive<f32>,
        /// The step size for the slider.
        step: f32,
    },
    String {
        /// The available options to select from for the parameter. If empty, any string is valid.
        choices: Vec<String>,
    },
}

/// The type's value.
#[derive(Debug, Clone)]
pub enum SDFParamValue {
    Boolean(bool),
    Int(i32),
    Float(f32),
    String(String),
}

#[cfg(feature = "app")]
impl SDFParam {
    /// Build the GUI for the parameter. Returns true if the value was changed.
    pub fn gui(&mut self, ui: &mut eframe::egui::Ui) -> bool {
        use eframe::egui;
        use eframe::egui::Slider;
        use eframe::egui::util::hash;
        ui.label(format!("{}:", self.name)).on_hover_text(&self.description);
        let changed = match &mut self.kind {
            SDFParamKind::Boolean => {
                match &mut self.value {
                    SDFParamValue::Boolean(value) =>
                        ui.checkbox(value, value.to_string()).changed(),
                    _ => false, // Ignore invalid values
                }
            }
            SDFParamKind::Int { range, step } => {
                match &mut self.value {
                    SDFParamValue::Int(value) =>
                        ui.add(Slider::new(value, range.clone()).step_by(*step as f64)).changed(),
                    _ => false, // Ignore invalid values
                }
            }
            SDFParamKind::Float { range, step } => {
                match &mut self.value {
                    SDFParamValue::Float(value) =>
                        ui.add(Slider::new(value, range.clone()).step_by(*step as f64)).changed(),
                    _ => false, // Ignore invalid values
                }
            }
            SDFParamKind::String { choices } => {
                match &mut self.value {
                    SDFParamValue::String(value) =>
                        if choices.is_empty() {
                            ui.text_edit_multiline(value).changed()
                        } else {
                            let mut value_index = choices.iter().position(|x| x == value)
                                .expect("SdfParameterValue: value not in choices!");
                            let changed = egui::ComboBox::new(hash(format!("sdf-param-{}", self.name)), value.clone())
                                .show_index(ui, &mut value_index, choices.len(), |i| choices[i].clone())
                                .changed();
                            if changed {
                                *value = choices[value_index].clone();
                            }
                            changed
                        },
                    _ => false, // Ignore invalid values
                }
            }
        };
        ui.end_row();
        changed
    }
}