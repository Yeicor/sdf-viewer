use cgmath::{InnerSpace, Vector3};

use crate::sdf::{SDFParam, SDFParamValue, SDFSurface};

/// Just a default implementation
#[doc(hidden)]
pub fn children_default_impl(_slf: impl SDFSurface) -> Vec<Box<dyn SDFSurface>> {
    vec![]
}

/// Just a default implementation
#[doc(hidden)]
pub fn id_default_impl(_slf: impl SDFSurface) -> u32 {
    0
}

/// Just a default implementation
#[doc(hidden)]
pub fn name_default_impl(_slf: impl SDFSurface) -> String {
    "Object".to_string()
}

/// Just a default implementation
#[doc(hidden)]
pub fn parameters_default_impl(_slf: impl SDFSurface) -> Vec<SDFParam> {
    vec![]
}

/// Just a default implementation
#[doc(hidden)]
pub fn set_parameter_default_impl(_slf: impl SDFSurface, _param_id: u32, _param_value: &SDFParamValue) -> Result<(), String> {
    Err("no parameters implemented by default, overwrite this method".to_string())
}

/// Just a default implementation
#[doc(hidden)]
pub fn changed_default_impl(slf: impl SDFSurface) -> Option<[Vector3<f32>; 2]> {
    for ch in slf.children() {
        if let Some(changed_box) = ch.changed() {
            // Note: will return changes to other children in the next call, which is allowed by docs.
            return Some(changed_box);
        }
    }
    None
}

/// Just a default implementation
#[doc(hidden)]
pub fn normal_default_impl(slf: impl SDFSurface, p: Vector3<f32>, eps: Option<f32>) -> Vector3<f32> {
    let eps = eps.unwrap_or(0.001);
    // Based on https://iquilezles.org/articles/normalsSDF/
    (Vector3::new(1., -1., -1.) * slf.sample(p + Vector3::new(eps, -eps, -eps), true).distance +
        Vector3::new(-1., 1., -1.) * slf.sample(p + Vector3::new(-eps, eps, -eps), true).distance +
        Vector3::new(-1., -1., 1.) * slf.sample(p + Vector3::new(-eps, -eps, eps), true).distance +
        Vector3::new(1., 1., 1.) * slf.sample(p + Vector3::new(eps, eps, eps), true).distance).normalize()
}

/// Merges two bounding boxes by performing the union.
pub fn merge_bounding_boxes(bbox: &[Vector3<f32>; 2], bbox2: &[Vector3<f32>; 2]) -> [Vector3<f32>; 2] {
    [ // Merge both bounding boxes
        Vector3::new(
            bbox[0].x.min(bbox2[0].x),
            bbox[0].y.min(bbox2[0].y),
            bbox[0].z.min(bbox2[0].z),
        ),
        Vector3::new(
            bbox[1].x.max(bbox2[1].x),
            bbox[1].y.max(bbox2[1].y),
            bbox[1].z.max(bbox2[1].z),
        ),
    ]
}
