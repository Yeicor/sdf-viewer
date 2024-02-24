//! An optimized WebAssembly compiler/interpreter that runs at near-native speed!!!
//! It also supports WASI!
//! It may not support target platforms added in the future.

use std::fmt::Debug;
use std::mem::size_of;
use std::ops::RangeInclusive;
use std::sync::{Arc, Mutex};

use cgmath::{Vector3, Zero};
use wasmer::{Function, Instance, Memory, Module, Store, Value};
use wasmer::AsStoreMut;
use wasmer::AsStoreRef;

use crate::sdf::{SDFParam, SDFParamKind, SDFParamValue, SDFSample, SDFSurface};
use crate::sdf::defaults::{children_default_impl, name_default_impl, parameters_default_impl, set_parameter_default_impl};
use crate::sdf::wasm::util::reinterpret_i32_as_u32;
use crate::sdf::wasm::util::reinterpret_u32_as_i32;

//use wasmer_wasi::*;

#[cfg(all(not(feature = "web"), target_arch = "wasm32"))]
compile_error!("On wasm32 targets, you need to enable the web feature (and disable any native* features).");

/// Loads the given bytes as a WebAssembly module that is then queried to satisfy the SDF trait.
pub async fn load_sdf_wasm(wasm_bytes: &[u8]) -> anyhow::Result<Box<dyn SDFSurface>> {
    unsafe { std::mem::transmute(load_sdf_wasm_send_sync(wasm_bytes).await?) }
}

/// Loads the given bytes as a WebAssembly module that is then queried to satisfy the SDF trait.
pub async fn load_sdf_wasm_send_sync(wasm_bytes: &[u8]) -> anyhow::Result<Box<dyn SDFSurface + Send + Sync>> {
    // TODO: Test other compilers provided by the wasmer crate

    let mut store = Store::default();
    #[cfg(target_arch = "wasm32")]
    {
        // HACK: Basic validation because the wasm32 wasmer crate doesn't do it for us.
        Module::validate(&store, wasm_bytes)?;
    }
    let module = Module::new(&store, wasm_bytes)?;

    // // The module shouldn't import anything. TODO: except maybe WASI.
    // let wasi_state = WasiState::new("program_name")
    //    // .env(b"HOME", "/home/home".to_string())
    //    // .arg("--help")
    //    // .preopen(|p| p.directory("src").read(true).write(true).create(true))?
    //    // .preopen(|p| p.directory(".").alias("dot").read(true))?
    //    .build()?;
    // let mut wasi_env = WasiEnv::new(wasi_state);
    // let import_object = wasi_env.import_object_for_all_wasi_versions(&module)?;
    let import_object = super::wasi::wasi_imports(&mut store);

    let instance = Instance::new(&mut store, &module, &import_object)?;

    // Call init() to initialize the module (optional).
    if let Ok(init) = instance.exports.get_function("init") {
        if let Err(err) = init.call(&mut store, &[]) {
            tracing::error!("Calling init() failed: {:?}", err);
        }
    }

    // Cache the exports of the module.
    let memory = instance.exports.get_memory("memory")?.clone();
    let f_bounding_box = instance.exports.get_function("bounding_box")?.clone();
    let f_bounding_box_free = instance.exports.get_function("bounding_box_free").ok().cloned();
    let f_sample = instance.exports.get_function("sample")?.clone();
    let f_sample_free = instance.exports.get_function("sample_free").ok().cloned();
    let f_children = instance.exports.get_function("children").ok().cloned();
    let f_children_free = instance.exports.get_function("children_free").ok().cloned();
    let f_name = instance.exports.get_function("name").ok().cloned();
    let f_name_free = instance.exports.get_function("name_free").ok().cloned();
    let f_parameters = instance.exports.get_function("parameters").ok().cloned();
    let f_parameters_free = instance.exports.get_function("parameters_free").ok().cloned();
    let f_set_parameter = instance.exports.get_function("set_parameter").ok().cloned();
    let f_set_parameter_free = instance.exports.get_function("set_parameter_free").ok().cloned();
    let f_changed = instance.exports.get_function("changed").ok().cloned();
    let f_changed_free = instance.exports.get_function("changed_free").ok().cloned();
    let f_normal = instance.exports.get_function("normal").ok().cloned();
    let f_normal_free = instance.exports.get_function("normal_free").ok().cloned();

    Ok(Box::new(WasmerSDF {
        sdf_id: 0, // This must always be the ID of the root SDF (as specified by the docs)
        store: Arc::new(Mutex::new(store)),
        memory,
        f_bounding_box,
        f_bounding_box_free,
        f_sample,
        f_sample_free,
        f_children,
        f_children_free,
        f_name,
        f_name_free,
        f_parameters,
        f_parameters_free,
        f_set_parameter,
        f_set_parameter_free,
        f_changed,
        f_changed_free,
        f_normal,
        f_normal_free,
    }))
}

// HACK: Is there a better alternative to implement both return types than macros for code duplication?
// load_sdf_wasm_code!(load_sdf_wasm_send_sync, anyhow::Result<Box<dyn SDFSurface + Send + Sync>>);
// load_sdf_wasm_code!(load_sdf_wasm, anyhow::Result<Box<dyn SDFSurface>>);

#[derive(Debug, Clone)] // Note: cloning is "cheap" (implementation details of wasmer)
pub struct WasmerSDF {
    sdf_id: u32,
    store: Arc<Mutex<Store>>,
    memory: Memory,
    f_bounding_box: Function,
    f_bounding_box_free: Option<Function>,
    f_sample: Function,
    f_sample_free: Option<Function>,
    f_children: Option<Function>,
    f_children_free: Option<Function>,
    f_name: Option<Function>,
    f_name_free: Option<Function>,
    f_parameters: Option<Function>,
    f_parameters_free: Option<Function>,
    f_set_parameter: Option<Function>,
    f_set_parameter_free: Option<Function>,
    f_changed: Option<Function>,
    f_changed_free: Option<Function>,
    f_normal: Option<Function>,
    f_normal_free: Option<Function>,
}

impl WasmerSDF {
    fn read_memory(&self, mem_pointer: u32, length: usize, store: &impl AsStoreRef) -> Vec<u8> {
        let mut res = vec![0u8; length];
        let mem_view = self.memory.view(store);
        for (i, v) in res.iter_mut().enumerate() {
            *v = mem_view.read_u8(mem_pointer as u64 + i as u64).unwrap_or_else(|err| {
                debug_assert!(false, "Out of bounds memory access at index {mem_pointer}, length {length}");
                tracing::error!("Out of bounds memory access at index {}, length {}: {:?}", mem_pointer, length, err);
                0
            })
        }
        res
    }

    fn write_memory(&self, mem_pointer: u32, to_write: &[u8], store: &impl AsStoreRef) {
        // HACK: How to reserve free memory for this instead of randomly overwriting it?
        // TODO: Maybe try malloc/free, which is published by tinygo binaries.
        #[allow(unused_unsafe)] // This is not unsafe on wasm32
        unsafe { // SAFETY: No data races.
            self.memory.view(store).write(mem_pointer as u64, to_write).unwrap_or_else(|err| {
                tracing::error!("Out of bounds memory access at index {}, length {}: {:?}", mem_pointer, to_write.len(), err);
            });
        }
    }

    fn read_pointer_length_memory(&self, mem_bytes: Vec<u8>, store: &impl AsStoreRef) -> Vec<u8> {
        let pointer = u32::from_le_bytes(mem_bytes[0..size_of::<u32>()].try_into().unwrap());
        let length_bytes = u32::from_le_bytes(mem_bytes[size_of::<u32>()..2 * size_of::<u32>()].try_into().unwrap());
        // if length_bytes == 8 {
        //     // DEBUG: "Should" be very stable over time if wasm is properly freeing the memory
        //     println!("Pointer to 8 bytes at {}", pointer);
        // }
        self.read_memory(pointer, length_bytes as usize, store)
    }
}

impl SDFSurface for WasmerSDF {
    fn bounding_box(&self) -> [Vector3<f32>; 2] {
        let mut _store = self.store.lock().unwrap();
        let result = self.f_bounding_box.call(&mut _store.as_store_mut(), &[
            Value::I32(reinterpret_u32_as_i32(self.sdf_id)),
        ]).unwrap_or_else(|err| {
            tracing::error!("Failed to get bounding box of wasm SDF with ID {}: {}", self.sdf_id, err);
            Box::new([])
        });
        let mut res = [Vector3::<f32>::zero(), Vector3::<f32>::new(1.0, 1.0, 1.0)];
        let mem_pointer = match return_value_to_mem_pointer(&result) {
            Some(mem_pointer) => mem_pointer,
            None => return res, // Errors already logged
        };
        let mem_bytes = self.read_memory(mem_pointer, size_of::<[Vector3<f32>; 2]>(), &_store.as_store_ref());
        self.f_bounding_box_free.as_ref().map(|f| f.call(&mut _store.as_store_mut(), &result)); // Free the memory, now that we copied it
        res[0].x = f32::from_le_bytes(mem_bytes[0..size_of::<f32>()].try_into().unwrap());
        res[0].y = f32::from_le_bytes(mem_bytes[size_of::<f32>()..2 * size_of::<f32>()].try_into().unwrap());
        res[0].z = f32::from_le_bytes(mem_bytes[2 * size_of::<f32>()..3 * size_of::<f32>()].try_into().unwrap());
        res[1].x = f32::from_le_bytes(mem_bytes[3 * size_of::<f32>()..4 * size_of::<f32>()].try_into().unwrap());
        res[1].y = f32::from_le_bytes(mem_bytes[4 * size_of::<f32>()..5 * size_of::<f32>()].try_into().unwrap());
        res[1].z = f32::from_le_bytes(mem_bytes[5 * size_of::<f32>()..6 * size_of::<f32>()].try_into().unwrap());
        res
    }

    fn sample(&self, p: Vector3<f32>, distance_only: bool) -> SDFSample {
        let mut _store = self.store.lock().unwrap();
        let result = self.f_sample.call(&mut _store.as_store_mut(), &[
            Value::I32(reinterpret_u32_as_i32(self.sdf_id)),
            Value::F32(p.x),
            Value::F32(p.y),
            Value::F32(p.z),
            Value::I32(i32::from(distance_only)),
        ]).unwrap_or_else(|err| {
            tracing::error!("Failed to get sample of wasm SDF with ID {}: {}", self.sdf_id, err);
            Box::new([])
        });
        let mem_pointer = match return_value_to_mem_pointer(&result) {
            Some(mem_pointer) => mem_pointer,
            None => return SDFSample::new(1.0, Vector3::zero()), // Errors already logged
        };
        let mem_bytes = self.read_memory(mem_pointer, size_of::<SDFSample>(), &_store.as_store_ref());
        self.f_sample_free.as_ref().map(|f| f.clone().call(&mut _store.as_store_mut(), &result)); // Free the memory, now that we copied it
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

    fn children(&self) -> Vec<Box<dyn SDFSurface>> {
        let mut _store = self.store.lock().unwrap();
        let f_children = match &self.f_children {
            Some(f_children) => f_children,
            None => return children_default_impl(self),
        };
        let result = f_children.clone().call(&mut _store.as_store_mut(), &[
            Value::I32(reinterpret_u32_as_i32(self.sdf_id)),
        ]).unwrap_or_else(|err| {
            tracing::error!("Failed to get children of wasm SDF with ID {}: {}", self.sdf_id, err);
            Box::new([])
        });
        let mem_pointer = match return_value_to_mem_pointer(&result) {
            Some(mem_pointer) => mem_pointer,
            None => return children_default_impl(self), // Errors already logged
        };
        let pointer_length = self.read_memory(mem_pointer, 2 * size_of::<u32>(), &_store.as_store_ref());
        let mem_bytes = self.read_pointer_length_memory(pointer_length, &_store.as_store_ref());
        self.f_children_free.as_ref().map(|f| f.call(&mut _store.as_store_mut(), &result)); // Free the memory, now that we copied it
        mem_bytes.chunks_exact(size_of::<u32>())
            .map(|ch| u32::from_le_bytes(ch.try_into().unwrap()))
            .filter_map(|child_sdf_id| {
                if child_sdf_id == self.sdf_id {
                    tracing::error!("Children of wasm SDF with ID {} include itself! Skipping, but this should be fixed.", self.sdf_id);
                    return None;
                }
                Some(Box::new(Self {
                    sdf_id: child_sdf_id,
                    ..self.clone() // Cloning is cheap and shares the memory of children parameters
                }) as Box<dyn SDFSurface>)
            }).collect()
    }

    fn id(&self) -> u32 {
        self.sdf_id // Already known!
    }

    fn name(&self) -> String {
        let mut _store = self.store.lock().unwrap();
        let f_name = match &self.f_name {
            Some(f_name) => f_name,
            None => return name_default_impl(self),
        };
        let result = f_name.call(&mut _store.as_store_mut(), &[
            Value::I32(reinterpret_u32_as_i32(self.sdf_id)),
        ]).unwrap_or_else(|err| {
            tracing::error!("Failed to get name of wasm SDF with ID {}: {}", self.sdf_id, err);
            Box::new([])
        });
        let mem_pointer = match return_value_to_mem_pointer(&result) {
            Some(mem_pointer) => mem_pointer,
            None => return name_default_impl(self), // Errors already logged
        };
        let pointer_length = self.read_memory(mem_pointer, 2 * size_of::<u32>(), &_store.as_store_ref());
        let mem_bytes = self.read_pointer_length_memory(pointer_length, &_store.as_store_ref());
        self.f_name_free.as_ref().map(|f| f.call(&mut _store.as_store_mut(), &result)); // Free the memory, now that we copied it
        String::from_utf8_lossy(mem_bytes.as_slice()).to_string()
    }

    fn parameters(&self) -> Vec<SDFParam> {
        let mut _store = self.store.lock().unwrap();
        let f_parameters = match &self.f_parameters {
            Some(f_parameters) => f_parameters,
            None => return parameters_default_impl(self),
        };
        let result = f_parameters.call(&mut _store.as_store_mut(), &[
            Value::I32(reinterpret_u32_as_i32(self.sdf_id)),
        ]).unwrap_or_else(|err| {
            tracing::error!("Failed to get parameters of wasm SDF with ID {}: {}", self.sdf_id, err);
            Box::new([])
        });
        let mem_pointer = match return_value_to_mem_pointer(&result) {
            Some(mem_pointer) => mem_pointer,
            None => return parameters_default_impl(self), // Errors already logged
        };
        let pointer_length = self.read_memory(mem_pointer, 2 * size_of::<u32>(), &_store.as_store_ref());
        let mem_bytes = self.read_pointer_length_memory(pointer_length, &_store.as_store_ref());
        let res = mem_bytes.chunks_exact(
            size_of::<u32>() /* param ID */ +
                2 * size_of::<u32>() /* name pointer */ +
                size_of::<u32>() + 3 * size_of::<f32>() /* SDFParamKindC */ +
                size_of::<u32>() + 2 * size_of::<u32>() /* SDFParamValueC */ +
                2 * size_of::<u32>() /* description pointer */)
            .filter_map(|sdf_param_mem| {
                let mut cur_offset = 0;
                /* param ID */
                let param_id = u32::from_le_bytes(sdf_param_mem[cur_offset..cur_offset + size_of::<u32>()].try_into().unwrap());
                cur_offset += size_of::<u32>();
                /* name pointer */
                let name_pointer_length = sdf_param_mem[cur_offset..cur_offset + 2 * size_of::<u32>()].into();
                let name_mem_bytes = self.read_pointer_length_memory(name_pointer_length, &_store.as_store_ref());
                // println!("sdf_param_mem: {:?} (name: {:?})", sdf_param_mem, String::from_utf8_lossy(&name_mem_bytes));
                cur_offset += 2 * size_of::<u32>();
                /* SDFParamKindC */
                let sdf_param_kind_enum_type = /* enum index = u32 */
                    u32::from_le_bytes(sdf_param_mem[cur_offset..cur_offset + size_of::<u32>()].try_into().unwrap());
                cur_offset += size_of::<u32>();
                let sdf_param_kind = match sdf_param_kind_enum_type {
                    0 => SDFParamKind::Boolean,
                    1 => SDFParamKind::Int {
                        range: RangeInclusive::new(
                            i32::from_le_bytes(sdf_param_mem[cur_offset..cur_offset + size_of::<i32>()].try_into().unwrap()),
                            i32::from_le_bytes(sdf_param_mem[cur_offset + size_of::<i32>()..cur_offset + 2 * size_of::<i32>()].try_into().unwrap()),
                        ),
                        step: i32::from_le_bytes(sdf_param_mem[cur_offset + 2 * size_of::<i32>()..cur_offset + 3 * size_of::<i32>()].try_into().unwrap()),
                    },
                    2 => SDFParamKind::Float {
                        range: RangeInclusive::new(
                            f32::from_le_bytes(sdf_param_mem[cur_offset..cur_offset + size_of::<f32>()].try_into().unwrap()),
                            f32::from_le_bytes(sdf_param_mem[cur_offset + size_of::<f32>()..cur_offset + 2 * size_of::<f32>()].try_into().unwrap()),
                        ),
                        step: f32::from_le_bytes(sdf_param_mem[cur_offset + 2 * size_of::<f32>()..cur_offset + 3 * size_of::<f32>()].try_into().unwrap()),
                    },
                    3 => {
                        let choices_pointer_length = sdf_param_mem[cur_offset..cur_offset + 2 * size_of::<u32>()].into();
                        let choices_mem_bytes = self.read_pointer_length_memory(choices_pointer_length, &_store.as_store_ref());
                        let choices = choices_mem_bytes.chunks_exact(2 * size_of::<u32>())
                            .map(|choice_mem_bytes| {
                                let choice_mem_bytes = self.read_pointer_length_memory(choice_mem_bytes.to_vec(), &_store.as_store_ref());
                                String::from_utf8_lossy(choice_mem_bytes.as_slice()).to_string()
                            })
                            .collect();
                        SDFParamKind::String { choices }
                    }
                    _ => {
                        debug_assert!(false, "Unknown SDF param kind enum type {sdf_param_kind_enum_type}");
                        tracing::error!("Unknown SDF param kind enum type {}", sdf_param_kind_enum_type); // TODO: less logging in case of multiple errors
                        return None;
                    }
                };
                cur_offset += size_of::<f32>() * 3; // Maximum size of SDFParamKindC
                /* SDFParamValueC */
                let sdf_param_value_enum_type = /* enum index = u32 */
                    u32::from_le_bytes(sdf_param_mem[cur_offset..cur_offset + size_of::<u32>()].try_into().unwrap());
                debug_assert_eq!(sdf_param_kind_enum_type, sdf_param_value_enum_type, "SDF param kind enum type {sdf_param_kind_enum_type} != SDF param value enum type {sdf_param_value_enum_type}");
                cur_offset += size_of::<u32>();
                let sdf_param_value = match sdf_param_value_enum_type {
                    0 => SDFParamValue::Boolean(sdf_param_mem[cur_offset] != 0) /* bool = u8 */,
                    1 => SDFParamValue::Int(i32::from_le_bytes(sdf_param_mem[cur_offset..cur_offset + size_of::<i32>()].try_into().unwrap())),
                    2 => SDFParamValue::Float(f32::from_le_bytes(sdf_param_mem[cur_offset..cur_offset + size_of::<f32>()].try_into().unwrap())),
                    3 => {
                        let value_pointer_length = sdf_param_mem[cur_offset..cur_offset + 2 * size_of::<u32>()].into();
                        let value_mem_bytes = self.read_pointer_length_memory(value_pointer_length, &_store.as_store_ref());
                        SDFParamValue::String(String::from_utf8_lossy(value_mem_bytes.as_slice()).to_string())
                    }
                    _ => {
                        debug_assert!(false, "Unknown SDF param value enum type {sdf_param_value_enum_type}");
                        tracing::error!("Unknown SDF param value enum type {}", sdf_param_value_enum_type); // TODO: less logging in case of multiple errors
                        return None;
                    }
                };
                cur_offset += 2 * size_of::<u32>(); // Maximum size of SDFParamValueC
                /* description */
                let desc_pointer_length = sdf_param_mem[cur_offset..cur_offset + 2 * size_of::<u32>()].into();
                let desc_mem_bytes = self.read_pointer_length_memory(desc_pointer_length, &_store.as_store_ref());

                Some(SDFParam {
                    id: param_id,
                    name: String::from_utf8_lossy(name_mem_bytes.as_slice()).to_string(),
                    kind: sdf_param_kind,
                    value: sdf_param_value,
                    description: String::from_utf8_lossy(desc_mem_bytes.as_slice()).to_string(),
                })
            })
            .collect();
        self.f_parameters_free.as_ref().map(|f| f.call(&mut _store.as_store_mut(), &result)); // Free the memory, now that we copied it
        res
    }

    fn set_parameter(&self, param_id: u32, param_value: &SDFParamValue) -> Result<(), String> {
        let mut _store = self.store.lock().unwrap();
        let f_set_parameter = match &self.f_set_parameter {
            Some(f_set_parameter) => f_set_parameter,
            None => return set_parameter_default_impl(self, param_id, param_value),
        };
        let tmp_store_mut = &mut _store.as_store_mut();
        let result = f_set_parameter.call(tmp_store_mut, &[
            Value::I32(reinterpret_u32_as_i32(self.sdf_id)),
            Value::I32(reinterpret_u32_as_i32(param_id)),
            Value::I32(match param_value {
                SDFParamValue::Boolean(_value) => 0,
                SDFParamValue::Int(_value) => 1,
                SDFParamValue::Float(_value) => 2,
                SDFParamValue::String(_value) => 3,
            }),
            Value::I32(match param_value {
                SDFParamValue::Boolean(value) => i32::from(*value),
                SDFParamValue::Int(value) => *value,
                SDFParamValue::Float(value) => unsafe { *(value as *const f32 as *const i32) }, // f32 bits to i32
                SDFParamValue::String(value) => {
                    let write_string_address = 0x12345;
                    self.write_memory(write_string_address, value.as_bytes(), &tmp_store_mut);
                    reinterpret_u32_as_i32(write_string_address)
                }
            }),
            Value::I32(match param_value {
                SDFParamValue::String(value) => reinterpret_u32_as_i32(value.len() as u32),
                _ => 0, // Unused
            }),
        ]).unwrap_or_else(|err| {
            tracing::error!("Failed to get parameters of wasm SDF with ID {}: {}", self.sdf_id, err);
            Box::new([])
        });
        let mem_pointer = match return_value_to_mem_pointer(&result) {
            Some(mem_pointer) => mem_pointer,
            None => return set_parameter_default_impl(self, param_id, param_value), // Errors already logged
        };
        let mem_bytes = self.read_memory(mem_pointer, 3 * size_of::<u32>(), &_store.as_store_ref());
        let mut cur_offset = 0;
        let enum_result_kind = u32::from_le_bytes(mem_bytes[cur_offset..cur_offset + size_of::<u32>()].try_into().unwrap());
        cur_offset += size_of::<u32>();
        let res = match enum_result_kind {
            0 => Ok(()),
            1 => {
                let error_string_ptr = u32::from_le_bytes(mem_bytes[cur_offset..cur_offset + size_of::<u32>()].try_into().unwrap());
                cur_offset += size_of::<u32>();
                let error_string_length = u32::from_le_bytes(mem_bytes[cur_offset..cur_offset + size_of::<u32>()].try_into().unwrap());
                // cur_offset += size_of::<u32>();
                let error_string_bytes = self.read_memory(error_string_ptr, error_string_length as usize, &_store.as_store_ref());
                Err(String::from_utf8_lossy(&error_string_bytes[..]).to_string())
            }
            _ => {
                debug_assert!(false, "Unknown SDF set parameter result kind enum type {enum_result_kind}");
                tracing::error!("Unknown SDF set parameter result kind enum type {}", enum_result_kind); // TODO: less logging in case of multiple errors
                Err(String::from("Unknown SDF set parameter result kind enum type"))
            }
        };
        self.f_set_parameter_free.as_ref().map(|f| f.call(&mut _store.as_store_mut(), &result)); // Free the memory, now that we copied it
        res
    }

    fn changed(&self) -> Option<[Vector3<f32>; 2]> {
        let mut _store = self.store.lock().unwrap();
        let f_changed = match &self.f_changed {
            Some(f_changed) => f_changed,
            None => return None,
        };
        let result = f_changed.clone().call(&mut _store.as_store_mut(), &[
            Value::I32(reinterpret_u32_as_i32(self.sdf_id)),
        ]).unwrap_or_else(|err| {
            tracing::error!("Failed to get changed of wasm SDF with ID {}: {}", self.sdf_id, err);
            Box::new([])
        });
        let mem_pointer = match return_value_to_mem_pointer(&result) {
            Some(mem_pointer) => mem_pointer,
            None => return None, // Errors already logged
        };
        let mem_bytes = self.read_memory(mem_pointer, (1 + 6) * size_of::<f32>(), &_store.as_store_ref());
        let mut cur_offset = 0;
        let enum_result_kind = u32::from_le_bytes(mem_bytes[cur_offset..cur_offset + size_of::<u32>()].try_into().unwrap());
        cur_offset += size_of::<u32>();
        let res = match enum_result_kind {
            0 => None,
            1 => {
                let x = f32::from_le_bytes(mem_bytes[cur_offset..cur_offset + size_of::<f32>()].try_into().unwrap());
                cur_offset += size_of::<f32>();
                let y = f32::from_le_bytes(mem_bytes[cur_offset..cur_offset + size_of::<f32>()].try_into().unwrap());
                cur_offset += size_of::<f32>();
                let z = f32::from_le_bytes(mem_bytes[cur_offset..cur_offset + size_of::<f32>()].try_into().unwrap());
                cur_offset += size_of::<f32>();
                let x2 = f32::from_le_bytes(mem_bytes[cur_offset..cur_offset + size_of::<f32>()].try_into().unwrap());
                cur_offset += size_of::<f32>();
                let y2 = f32::from_le_bytes(mem_bytes[cur_offset..cur_offset + size_of::<f32>()].try_into().unwrap());
                cur_offset += size_of::<f32>();
                let z2 = f32::from_le_bytes(mem_bytes[cur_offset..cur_offset + size_of::<f32>()].try_into().unwrap());
                // cur_offset += size_of::<f32>();
                Some([Vector3::new(x, y, z), Vector3::new(x2, y2, z2)])
            }
            _ => {
                debug_assert!(false, "Unknown SDF changed result kind enum type {enum_result_kind}");
                tracing::error!("Unknown SDF changed result kind enum type {}", enum_result_kind); // TODO: less logging in case of multiple errors
                None
            }
        };
        self.f_changed_free.as_ref().map(|f| f.call(&mut _store.as_store_mut(), &result)); // Free the memory, now that we copied it
        res
    }

    fn normal(&self, p: Vector3<f32>, eps: Option<f32>) -> Vector3<f32> {
        let mut _store = self.store.lock().unwrap();
        let f_normal = match &self.f_normal {
            Some(f_normal) => f_normal,
            None => return Vector3::new(0.0, 0.0, 0.0),
        };
        let tmp_store_mut = &mut _store.as_store_mut();
        let result = f_normal.call(tmp_store_mut, &[
            Value::I32(reinterpret_u32_as_i32(self.sdf_id)),
            Value::F32(p.x),
            Value::F32(p.y),
            Value::F32(p.z),
            Value::I32({
                let write_string_address = 0x12300;
                self.write_memory(write_string_address, match eps {
                    None => &[0, 0, 0, 0],
                    Some(_) => &[1, 0, 0, 0], // Little-endian 1 for error
                }, &tmp_store_mut);
                self.write_memory(write_string_address + size_of::<u32>() as u32, &match eps {
                    None => [0; size_of::<f32>()],
                    Some(eps) => eps.to_le_bytes(),
                }, &tmp_store_mut);
                reinterpret_u32_as_i32(write_string_address)
            }),
        ]).unwrap_or_else(|err| {
            tracing::error!("Failed to get normal of wasm SDF with ID {}: {}", self.sdf_id, err);
            Box::new([])
        });
        let mem_pointer = match return_value_to_mem_pointer(&result) {
            Some(mem_pointer) => mem_pointer,
            None => return Vector3::new(0.0, 0.0, 0.0), // Errors already logged
        };
        let mem_bytes = self.read_memory(mem_pointer, 3 * size_of::<f32>(), &_store.as_store_ref());
        let x = f32::from_le_bytes(mem_bytes[0..size_of::<f32>()].try_into().unwrap());
        let y = f32::from_le_bytes(mem_bytes[size_of::<f32>()..2 * size_of::<f32>()].try_into().unwrap());
        let z = f32::from_le_bytes(mem_bytes[2 * size_of::<f32>()..3 * size_of::<f32>()].try_into().unwrap());
        self.f_normal_free.as_ref().map(|f| f.call(&mut _store.as_store_mut(), &result)); // Free the memory, now that we copied it
        Vector3::new(x, y, z)
    }
}

fn return_value_to_mem_pointer(result: &[Value]) -> Option<u32> {
    if result.len() != 1 {
        tracing::error!("Expected 1 output, got {}", result.len());
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
