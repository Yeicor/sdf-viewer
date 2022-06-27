//! This provides a external API for the SDF library. It matches the WebAssembly specification
//! defined at [crate::sdf::wasm].
//!
//! Recommendation: minimize WebAssembly size and maximize build speeds by using a separate crate.
//! We do not do this here because this is easier to maintain in the main repo.

use std::cell::RefCell;
use std::collections::HashMap;

use cgmath::{Vector3, Zero};

use crate::sdf::{SDFSample, SDFSurface};
use crate::sdf::demo::SDFDemo;

/// Creates the SDF scene. It only gets called once
fn sdf_scene() -> Box<dyn SDFSurface> {
    Box::new(SDFDemo::default())
}

/// Returns the reference to the already initialized SDF registry, which links each ID to the [`SDFSurface`] implementation.
fn sdf_registry<R>(f: impl FnOnce(&HashMap<u32, &'static (dyn SDFSurface)>) -> R) -> R {
    thread_local! {
        pub static REGISTRY: RefCell<HashMap<u32, &'static (dyn SDFSurface)>> = RefCell::new(HashMap::new());
    }
    REGISTRY.with(|registry| {
        let mut registry_ref = registry.borrow();
        if registry_ref.is_empty() { // Only run initialization once
            drop(registry_ref);
            let mut registry_ref_mut = registry.borrow_mut();
            let root_sdf = Box::leak(sdf_scene());
            // Find all children and store them
            let mut to_process = vec![&*root_sdf];
            while !to_process.is_empty() {
                let cur_sdf = to_process.pop().unwrap();
                registry_ref_mut.insert(cur_sdf.id(), cur_sdf);
                for ch in cur_sdf.children() {
                    to_process.push(ch);
                }
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
pub extern "C" fn sample(sdf_id: u32, p: Vector3<f32>, distance_only: bool) -> Box<SDFSample> {
    Box::new(sdf_registry(|r| r.get(&sdf_id)
        .map(|sdf| sdf.sample(p, distance_only))
        .unwrap_or_else(|| {
            eprintln!("Failed to find SDF with ID {}", sdf_id);
            SDFSample::new(0.0, Vector3::zero())
        })))
}
