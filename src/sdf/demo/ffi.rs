//! This provides a external API for the SDF library. It matches the WebAssembly specification
//! defined at [crate::sdf::wasm].
//!
//! Recommendation: minimize WebAssembly size and maximize build speeds by using a separate crate.
//! We do not do this here because this is easier to maintain in the main repo.

use std::collections::HashMap;

use cgmath::{Vector3, Zero};
use clap::once_cell::sync::OnceCell;

use crate::sdf::{SDFSample, SDFSurface};
use crate::sdf::demo::SDFDemo;

/// Returns the reference to the already initialized SDF registry.
fn sdf_registry() -> &'static HashMap<u32, &'static (dyn SDFSurface + Send + Sync)> {
    static REGISTRY: OnceCell<HashMap<u32, &'static (dyn SDFSurface + Send + Sync)>> = OnceCell::new();
    REGISTRY.get_or_init(|| {
        // We have no need to dynamically initialize the registry, but it could be done
        let mut m = HashMap::new();
        let root_sdf = Box::leak(Box::new(SDFDemo::default()));
        // Find all children and store them only the first time
        let mut to_process: Vec<&'static (dyn SDFSurface)> = vec![root_sdf];
        while !to_process.is_empty() {
            let cur_sdf = to_process.pop().unwrap();
            m.insert(cur_sdf.id(), cur_sdf);
            for ch in cur_sdf.children() {
                to_process.push(ch);
            }
        }
        // FIXME: There should be a better solution for this (without reducing performance of the SDFDemo)
        unsafe { std::mem::transmute(m) }
    })
}

#[no_mangle]
pub extern "C" fn bounding_box(sdf_id: u32) -> Box<[Vector3<f32>; 2]> {
    Box::new(sdf_registry().get(&sdf_id)
        .map(|sdf| sdf.bounding_box())
        .unwrap_or_else(|| {
            eprintln!("Failed to find SDF with ID {}", sdf_id);
            [Vector3::zero(); 2]
        }))
}

#[no_mangle]
pub extern "C" fn sample(sdf_id: u32, p: Vector3<f32>, distance_only: bool) -> Box<SDFSample> {
    Box::new(sdf_registry().get(&sdf_id)
        .map(|sdf| sdf.sample(p, distance_only))
        .unwrap_or_else(|| {
            eprintln!("Failed to find SDF with ID {}", sdf_id);
            SDFSample::new(0.0, Vector3::zero())
        }))
}
