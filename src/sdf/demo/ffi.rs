//! This provides a external API for the SDF library. It matches the WebAssembly specification
//! defined at [crate::sdf::wasm].
//!
//! Recommendation: minimize WebAssembly size and maximize build speeds by using a separate crate.
//! We do not do this here because this is easier to maintain in the main repo.

use std::cell::RefCell;
use std::collections::HashMap;
use std::mem::size_of;
use std::ops::Range;

use cgmath::{Vector3, Zero};

use crate::sdf::{SDFParamKind, SDFParamValue, SDFSample, SDFSurface};
use crate::sdf::demo::SDFDemo;

/// Creates the SDF scene. It only gets called once
fn sdf_scene() -> Box<dyn SDFSurface> {
    Box::new(SDFDemo::default())
}

/// Returns the reference to the already initialized SDF registry, which links each ID to the [`SDFSurface`] implementation.
fn sdf_registry<R>(f: impl FnOnce(&HashMap<u32, Box<dyn SDFSurface>>) -> R) -> R {
    thread_local! {
        pub static REGISTRY: RefCell<HashMap<u32, Box<dyn SDFSurface>>> = RefCell::new(HashMap::new());
    }
    REGISTRY.with(|registry| {
        let mut registry_ref = registry.borrow();
        if registry_ref.is_empty() { // Only run initialization once
            drop(registry_ref);
            let mut registry_ref_mut = registry.borrow_mut();
            let root_sdf = sdf_scene();
            // Find all children and store them
            let mut to_process = vec![root_sdf];
            while !to_process.is_empty() {
                let cur_sdf = to_process.pop().unwrap();
                for ch in cur_sdf.children() {
                    to_process.push(ch);
                }
                registry_ref_mut.insert(cur_sdf.id(), cur_sdf);
            }
            drop(registry_ref_mut);
            registry_ref = registry.borrow();
        }
        f(&*registry_ref)
    })
}

#[no_mangle]
pub extern "C" fn bounding_box(sdf_id: u32) -> Box<[Vector3<f32>; 2]> {
    Box::new(sdf_registry(|r| r.get(&sdf_id)
        .map(|sdf| sdf.bounding_box())
        .unwrap_or_else(|| {
            eprintln!("Failed to find SDF with ID {}", sdf_id);
            [Vector3::zero(); 2]
        })))
}

#[no_mangle]
pub extern "C" fn bounding_box_free(ret: Box<[Vector3<f32>; 2]>) {
    drop(ret); // The function is required to free memory. The drop() call is optional but specifies what this should do
}

#[no_mangle]
pub extern "C" fn sample(sdf_id: u32, p: Vector3<f32>, distance_only: bool) -> Box<SDFSample> {
    Box::new(sdf_registry(|r| r.get(&sdf_id)
        .map(|sdf| sdf.sample(p, distance_only))
        .unwrap_or_else(|| {
            eprintln!("Failed to find SDF with ID {}", sdf_id);
            SDFSample::new(0.0, Vector3::zero())
        })))
}

#[no_mangle]
pub extern "C" fn sample_free(ret: Box<SDFSample>) {
    drop(ret); // The function is required to free memory. The drop() call is optional but specifies what this should do
}

/// The structure returned for strings and other arrays.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct PointerLength<T> {
    ptr: *const u8,
    len_bytes: usize,
    // Length in bytes, not number of elements
    phantom: std::marker::PhantomData<T>,
}

impl<T> PointerLength<T> {
    /// Creates a Pointer + length from the owned vector, leaking the memory until free is called.
    fn from_vec(mut s: Vec<T>) -> Self {
        s.shrink_to_fit(); // Make sure capacity == len
        let ptr = s.as_ptr() as *const _;
        let len_bytes = s.len() * size_of::<T>();
        std::mem::forget(s); // Leak memory, until free is called
        PointerLength { ptr, len_bytes, phantom: std::marker::PhantomData }
    }

    /// Returns back the vector referenced by this PointerLength, in order to properly free the memory
    fn own_again(self) -> Vec<T> {
        if self.ptr.is_null() {
            return Vec::new();
        }
        // SAFETY: We assume that the pointer is valid and that the length is correct.
        unsafe { Vec::from_raw_parts(self.ptr as *mut T, self.len_bytes / size_of::<T>(), self.len_bytes / size_of::<T>()) }
    }

    fn null() -> Self {
        PointerLength {
            ptr: std::ptr::null(),
            len_bytes: 0,
            phantom: std::marker::PhantomData,
        }
    }
}

#[no_mangle]
pub extern "C" fn children(sdf_id: u32) -> Box<PointerLength<u32>> {
    Box::new(sdf_registry(|r| r.get(&sdf_id)
        .map(|sdf| {
            let children_ids = sdf.children().into_iter()
                .map(|ch| ch.id()).collect::<Vec<_>>();
            PointerLength::from_vec(children_ids)
        })
        .unwrap_or_else(|| {
            eprintln!("Failed to find SDF with ID {}", sdf_id);
            PointerLength::null()
        })))
}

#[no_mangle]
pub extern "C" fn children_free(ret: Box<PointerLength<u32>>) {
    ret.own_again();
}

// Note: ID is automatically known by caller (root is always 0 and knows the rest of IDs)

#[no_mangle]
pub extern "C" fn name(sdf_id: u32) -> Box<PointerLength<u8>> { // Return pointer and length
    Box::new(sdf_registry(|r| r.get(&sdf_id)
        .map(|sdf| {
            PointerLength::from_vec(sdf.name().as_bytes().to_vec())
        })
        .unwrap_or_else(|| {
            eprintln!("Failed to find SDF with ID {}", sdf_id);
            PointerLength::null()
        })))
}

#[no_mangle]
pub extern "C" fn name_free(ret: Box<PointerLength<u8>>) {
    ret.own_again();
}

/// The metadata and current state of a parameter of a SDF.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct SDFParamC {
    /// The ID of the parameter. Must be unique within this SDF (not necessarily within the SDF hierarchy).
    pub id: u32,
    /// The name of the parameter.
    pub name: PointerLength<u8>,
    /// The type definition for the parameter.
    pub kind: SDFParamKindC,
    /// The current value of the parameter. MUST be of the same kind as the type definition.
    pub value: SDFParamValueC,
    /// The user-facing description for the parameter.
    pub description: PointerLength<u8>,
}

/// The type, including bounds, choices or other type-specific metadata of a parameter.
#[derive(Debug, Clone)]
#[repr(C, u32)]
pub enum SDFParamKindC {
    // No parameters required for booleans
    Boolean,
    Int {
        /// The range (inclusive) that must contain the value.
        range: Range<i32>,
        /// The step size for the slider.
        step: i32,
    },
    Float {
        /// The range (inclusive) that must contain the value.
        range: Range<f32>,
        /// The step size for the slider.
        step: f32,
    },
    String {
        /// The available options to select from for the parameter. If empty, any string is valid.
        choices: PointerLength<PointerLength<u8>>,
    },
}

impl SDFParamKindC {
    fn from_api(kind: &SDFParamKind) -> Self {
        match kind {
            SDFParamKind::Boolean => SDFParamKindC::Boolean,
            SDFParamKind::Int { range, step } => SDFParamKindC::Int { range: *range.start()..*range.end(), step: *step },
            SDFParamKind::Float { range, step } => SDFParamKindC::Float { range: *range.start()..*range.end(), step: *step },
            SDFParamKind::String { choices } => SDFParamKindC::String {
                choices: PointerLength::from_vec(choices.iter()
                    .map(|s| PointerLength::from_vec(s.as_bytes().to_vec())).collect()),
            },
        }
    }
}

/// The type's value.
#[derive(Debug, Clone)]
#[repr(C, u32)]
pub enum SDFParamValueC {
    Boolean(bool),
    Int(i32),
    Float(f32),
    String(PointerLength<u8>),
}

impl SDFParamValueC {
    fn from_api(value: &SDFParamValue) -> Self {
        match value {
            SDFParamValue::Boolean(b) => SDFParamValueC::Boolean(*b),
            SDFParamValue::Int(i) => SDFParamValueC::Int(*i),
            SDFParamValue::Float(f) => SDFParamValueC::Float(*f),
            SDFParamValue::String(s) => SDFParamValueC::String(
                PointerLength::from_vec(s.as_bytes().to_vec())),
        }
    }

    fn to_api(&self) -> SDFParamValue {
        match self {
            SDFParamValueC::Boolean(b) => SDFParamValue::Boolean(*b),
            SDFParamValueC::Int(i) => SDFParamValue::Int(*i),
            SDFParamValueC::Float(f) => SDFParamValue::Float(*f),
            SDFParamValueC::String(s) => SDFParamValue::String(
                String::from_utf8_lossy(s.clone().own_again().as_slice()).to_string()),
        }
    }
}

#[no_mangle]
pub extern "C" fn parameters(sdf_id: u32) -> Box<PointerLength<SDFParamC>> {
    Box::new(sdf_registry(|r| r.get(&sdf_id)
        .map(|sdf| {
            let params = sdf.parameters();
            // Convert to the C-compatible format
            let mut params_res = vec![];
            for ref mut param in params {
                params_res.push(SDFParamC {
                    id: param.id,
                    name: PointerLength::from_vec(param.name.as_bytes().to_vec()),
                    kind: SDFParamKindC::from_api(&param.kind),
                    value: SDFParamValueC::from_api(&param.value),
                    description: PointerLength::from_vec(param.description.as_bytes().to_vec()),
                });
            }
            PointerLength::from_vec(params_res)
        })
        .unwrap_or_else(|| {
            eprintln!("Failed to find SDF with ID {}", sdf_id);
            PointerLength::null()
        })))
}

#[no_mangle]
pub extern "C" fn parameters_free(ret: Box<PointerLength<SDFParamC>>) {
    let ret = ret.own_again();
    for param in ret {
        param.name.own_again();
        param.description.own_again();
        match param.kind {
            SDFParamKindC::Boolean { .. } => {}
            SDFParamKindC::Int { .. } => {}
            SDFParamKindC::Float { .. } => {}
            SDFParamKindC::String { choices } => {
                for previous_value in choices.own_again() {
                    previous_value.own_again();
                }
            }
        }
        match param.value {
            SDFParamValueC::Boolean(_) => {}
            SDFParamValueC::Int(_) => {}
            SDFParamValueC::Float(_) => {}
            SDFParamValueC::String(s) => {
                s.own_again();
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn set_parameter(sdf_id: u32, param_id: u32, param_value: SDFParamValueC) -> Box<Result<(), PointerLength<u8>>> {
    Box::new(sdf_registry(|r| r.get(&sdf_id)
        .map(|sdf| {
            sdf.set_parameter(param_id, &param_value.to_api())
                .map_err(|err_str| PointerLength::from_vec(
                    err_str.as_bytes().to_vec()))
        })
        .unwrap_or_else(|| {
            let msg = format!("Failed to find SDF with ID {}", sdf_id);
            eprintln!("{}", msg);
            Err(PointerLength::from_vec(msg.as_bytes().to_vec()))
        })))
}

#[no_mangle]
pub extern "C" fn set_parameter_free(ret: Box<Result<(), PointerLength<u8>>>) {
    let _ignore = ret.map_err(|err| err.own_again());
}

#[no_mangle]
pub extern "C" fn changed(sdf_id: u32) -> Box<Option<[Vector3<f32>; 2]>> {
    Box::new(sdf_registry(|r| r.get(&sdf_id)
        .map(|sdf| {
            sdf.changed()
        })
        .unwrap_or_else(|| {
            eprintln!("Failed to find SDF with ID {}", sdf_id);
            None
        })))
}

#[no_mangle]
pub extern "C" fn changed_free(ret: Box<Option<[Vector3<f32>; 2]>>) {
    drop(ret); // The function is required to free memory. The drop() call is optional but specifies what this should do
}

#[no_mangle]
pub extern "C" fn normal(sdf_id: u32, p: Vector3<f32>, eps: Box<Option<f32>>) -> Box<Vector3<f32>> {
    Box::new(sdf_registry(|r| r.get(&sdf_id)
        .map(|sdf| {
            sdf.normal(p, *eps)
        })
        .unwrap_or_else(|| {
            eprintln!("Failed to find SDF with ID {}", sdf_id);
            Vector3::zero()
        })))
}

#[no_mangle]
pub extern "C" fn normal_free(ret: Box<Vector3<f32>>) {
    drop(ret); // The function is required to free memory. The drop() call is optional but specifies what this should do
}
