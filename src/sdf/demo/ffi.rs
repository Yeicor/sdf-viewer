//! This provides a external API for the SDF library. It matches the WebAssembly specification
//! defined at [crate::sdf::wasm].
//!
//! Recommendation: minimize WebAssembly size and maximize build speeds by using a separate crate.
//! We do not do this here because this is easier to maintain in the main repo.

use std::cell::RefCell;
use std::collections::HashMap;
use std::mem::size_of;

use cgmath::{Vector3, Zero};

use crate::sdf::{SDFSample, SDFSurface};
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
#[repr(C)]
pub struct PointerLength<T> {
    ptr: *const T,
    len_bytes: usize, // Length in bytes, not number of elements
}

impl<T> PointerLength<T> {
    /// Creates a Pointer + length from the owned vector, leaking the memory until free is called.
    fn from_vec(mut s: Vec<T>) -> Self {
        s.shrink_to_fit(); // Make sure capacity == len
        let ptr = s.as_ptr();
        let len_bytes = s.len() * size_of::<T>();
        std::mem::forget(s); // Leak memory, until free is called
        PointerLength { ptr, len_bytes }
    }

    /// Returns back the vector referenced by this PointerLength, in order to properly free the memory
    fn free(&self) -> Vec<T> {
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
    ret.free();
}

// Note: ID is automatically known by caller (root is always 0 and knows the rest of IDs)

#[no_mangle]
pub extern "C" fn name(sdf_id: u32) -> Box<PointerLength<u8>> { // Return pointer and length
    Box::new(sdf_registry(|r| r.get(&sdf_id)
        .map(|sdf| {
            PointerLength::from_vec(sdf.name().as_bytes().into_iter().copied().collect::<Vec<_>>())
        })
        .unwrap_or_else(|| {
            eprintln!("Failed to find SDF with ID {}", sdf_id);
            PointerLength::null()
        })))
}

#[no_mangle]
pub extern "C" fn name_free(ret: Box<PointerLength<u8>>) {
    ret.free();
}

// /// The metadata and current state of a parameter of a SDF.
// #[repr(C)]
// pub struct SDFParamC {
//     /// The name of the parameter. Must be unique within the SDF.
//     pub name: PointerLength<u8>,
//     /// The current value of the parameter.
//     pub value: SDFParamValueC,
//     /// The user-facing description for the parameter.
//     pub description: PointerLength<u8>,
// }
//
// /// The type, value, bounds and other type-specific metadata of a parameter.
// #[repr(C, u8)] // https://rust-lang.github.io/unsafe-code-guidelines/layout/enums.html#explicit-repr-annotation-with-c-compatibility
// pub enum SDFParamValueC {
//     Boolean {
//         value: bool,
//     },
//     Int {
//         value: i32,
//         range: Range<i32>,
//         step: i32,
//     },
//     Float {
//         value: f32,
//         range: Range<f32>,
//         step: f32,
//     },
//     String {
//         value: PointerLength<u8>,
//         /// The available options to select from for the parameter. If empty, any string is valid.
//         choices: PointerLength<u8>, // Of PointerLengths!
//     },
// }
//
// #[no_mangle]
// pub extern "C" fn parameters(sdf_id: u32) -> Box<PointerLength> {
//     Box::new(sdf_registry(|r| r.get(&sdf_id)
//         .map(|sdf| {
//             let params = sdf.parameters();
//             // Convert to the C-compatible format
//             let mut params_res = vec![]
//             for ref mut param in params {
//                 params_res.push(SDFParamC {
//                     name: PointerLength::from_str(param.name.as_str()),
//                     value: match &param.value {
//                         SDFParamValue::Boolean { value } => SDFParamValueC::Boolean { value: *value },
//                         SDFParamValue::Int { value, range, step } =>
//                             SDFParamValueC::Int { value: *value, range: range.into(), step: *step },
//                         SDFParamValue::Float { value, range, step } =>
//                             SDFParamValueC::Float { value: *value, range: range.into(), step: *step },
//                         SDFParamValue::String { value, choices } =>
//                             SDFParamValueC::String {
//                                 value: PointerLength::from_str(value.as_str()),
//                                 choices: PointerLength::from_vec(choices.into_iter()
//                                     .map(|s| PointerLength::from_str(s.as_str())).collect()),
//                             },
//                     },
//                     description: PointerLength::from_str(param.description.as_str()),
//                 });
//             }
//             PointerLength::from_vec(&params_res)
//         })
//         .unwrap_or_else(|| {
//             eprintln!("Failed to find SDF with ID {}", sdf_id);
//             PointerLength::null()
//         })))
// }