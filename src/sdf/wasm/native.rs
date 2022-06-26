//! An optimized WebAssembly compiler/interpreter that runs at near-native speed!!!
//! It may not support target platforms added in the future.

use std::mem::size_of;

use cgmath::{Vector3, Zero};
use wasmer::*;

use crate::sdf::{SDFSample, SDFSurface};

use super::reinterpret_i32_as_u32;
use super::reinterpret_u32_as_i32;

#[cfg(all(not(feature = "web"), target_arch = "wasm32"))]
compile_error!("On wasm32 targets, you need to enable the js feature to be able to run wasmer.");

macro_rules! load_sdf_wasm_code {
    ($name: ident, $kind: ty) => {
        /// Loads the given bytes as a WebAssembly module that is then queried to satisfy the SDF trait.
        pub async fn $name(wasm_bytes: &[u8]) -> $kind {
            // TODO: Test other compilers provided by the wasmer crate

            let store = Store::default();
            let module = { // HACK: chrome requires asynchronous compilation
                #[cfg(target_arch = "wasm32")]
                {
                    Module::new(&store, wasm_bytes).await?
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    Module::new(&store, wasm_bytes)?
                }
            };
            // The module shouldn't import anything, so we create an empty import object.
            let import_object = imports! {};
            let instance = { // HACK: chrome requires asynchronous instantiation
                #[cfg(target_arch = "wasm32")]
                {
                    Instance::new(&module, &import_object).await?
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    Instance::new(&module, &import_object)?
                }
            };

            // Cache the exports of the module.
            let memory = instance.exports.get_memory("memory")?.clone();
            let f_bounding_box = instance.exports.get_function("bounding_box")?.clone();
            let f_sample = instance.exports.get_function("sample")?.clone();

            Ok(Box::new(WasmerSDF {
                memory,
                f_bounding_box,
                f_sample,
                sdf_id: 0,
            }))
        }
    };
}

// HACK: Is there a better alternative to implement both return types than macros for code duplication?
load_sdf_wasm_code!(load_sdf_wasm_send_sync, anyhow::Result<Box<dyn SDFSurface + Send + Sync>>);
load_sdf_wasm_code!(load_sdf_wasm, anyhow::Result<Box<dyn SDFSurface>>);

pub struct WasmerSDF {
    memory: Memory,
    f_bounding_box: Function,
    f_sample: Function,
    sdf_id: u32,
}

impl WasmerSDF {
    fn read_memory(&self, mem_pointer: u32, length: usize) -> Vec<u8> {
        let mut res = vec![0u8; length];
        #[cfg(target_arch = "wasm32")]
        {
            self.memory.uint8view()
                .subarray(mem_pointer, mem_pointer + length as u32)
                .copy_to(&mut res);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Future version of wasmer:
            // self.memory.read(mem_pointer as u64, &mut res).unwrap_or_else(|err| {
            //     tracing::error!("Out of bounds memory access at index {}, length {}: {:?}", mem_pointer, length, err);
            // });
            let mem_view = self.memory.view::<u8>();
            for (i, v) in res.iter_mut().enumerate() {
                *v = mem_view.get(mem_pointer as usize + i).map(|b| b.get()).unwrap_or_else(|| {
                    tracing::error!("Out of bounds memory access at index {}", mem_pointer as usize + i);
                    0
                })
            }
        }
        res
    }
}

impl SDFSurface for WasmerSDF {
    fn bounding_box(&self) -> [Vector3<f32>; 2] {
        let result = self.f_bounding_box.call(&[
            Val::I32(reinterpret_u32_as_i32(self.sdf_id)),
        ]).unwrap_or_else(|err| {
            tracing::error!("Failed to get bounding box of wasm SDF with ID {}: {}", self.sdf_id, err);
            Box::new([])
        });
        let mut res = [Vector3::<f32>::zero(); 2];
        let mem_pointer = match return_value_to_mem_pointer(&result) {
            Some(mem_pointer) => mem_pointer,
            None => return res, // Errors already logged
        };
        let mem_bytes = self.read_memory(mem_pointer, size_of::<[Vector3<f32>; 2]>());
        res[0].x = f32::from_le_bytes(mem_bytes[0..size_of::<f32>()].try_into().unwrap());
        res[0].y = f32::from_le_bytes(mem_bytes[size_of::<f32>()..2 * size_of::<f32>()].try_into().unwrap());
        res[0].z = f32::from_le_bytes(mem_bytes[2 * size_of::<f32>()..3 * size_of::<f32>()].try_into().unwrap());
        res[1].x = f32::from_le_bytes(mem_bytes[3 * size_of::<f32>()..4 * size_of::<f32>()].try_into().unwrap());
        res[1].y = f32::from_le_bytes(mem_bytes[4 * size_of::<f32>()..5 * size_of::<f32>()].try_into().unwrap());
        res[1].z = f32::from_le_bytes(mem_bytes[5 * size_of::<f32>()..6 * size_of::<f32>()].try_into().unwrap());
        res
    }

    fn sample(&self, p: Vector3<f32>, distance_only: bool) -> SDFSample {
        let result = self.f_sample.call(&[
            Val::I32(reinterpret_u32_as_i32(self.sdf_id)),
            Val::F32(p.x),
            Val::F32(p.y),
            Val::F32(p.z),
            Val::I32(if distance_only { 1 } else { 0 }),
        ]).unwrap_or_else(|err| {
            tracing::error!("Failed to get bounding box of wasm SDF with ID {}: {}", self.sdf_id, err);
            Box::new([])
        });
        let mem_pointer = match return_value_to_mem_pointer(&result) {
            Some(mem_pointer) => mem_pointer,
            None => return SDFSample::new(1.0, Vector3::zero()), // Errors already logged
        };
        let mem_bytes = self.read_memory(mem_pointer, size_of::<SDFSample>());
        SDFSample {
            distance: f32::from_le_bytes(mem_bytes[0..size_of::<f32>()].try_into().unwrap()),
            color: Vector3::new(
                f32::from_le_bytes(mem_bytes[size_of::<f32>()..2 * size_of::<f32>()].try_into().unwrap()),
                f32::from_le_bytes(mem_bytes[2 * size_of::<f32>()..3 * size_of::<f32>()].try_into().unwrap()),
                f32::from_le_bytes(mem_bytes[3 * size_of::<f32>()..4 * size_of::<f32>()].try_into().unwrap()),
            ),
            metallic: f32::from_le_bytes(mem_bytes[4 * size_of::<f32>()..5 * size_of::<f32>()].try_into().unwrap()),
            roughness: f32::from_le_bytes(mem_bytes[5 * size_of::<f32>()..6 * size_of::<f32>()].try_into().unwrap()),
            occlusion: f32::from_le_bytes(mem_bytes[6 * size_of::<f32>()..7 * size_of::<f32>()].try_into().unwrap()),
        }
    }
}

fn return_value_to_mem_pointer(result: &[Val]) -> Option<u32> {
    if result.len() != 1 {
        tracing::error!("Expected 1 output for bounding_box(), got {}", result.len());
        return None;
    }
    let mem_pointer = match result[0].i32() {
        Some(pointer) => reinterpret_i32_as_u32(pointer),
        None => {
            tracing::error!("Expected i32 output for bounding_box(), got {:?}", result[0]);
            return None;
        }
    };
    Some(mem_pointer)
}