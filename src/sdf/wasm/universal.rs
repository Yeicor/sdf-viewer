//! A cross-platform (pure-rust) WebAssembly interpreter that works anywhere, but is very slow.
//! Only the core functionality is implemented.

use anyhow::anyhow;
use cgmath::{Vector3, Zero};
use wasmi::{Error, Externals, FuncInstance, FuncRef, ImportsBuilder, MemoryRef, Module, ModuleImportResolver, ModuleInstance, ModuleRef, RuntimeArgs, RuntimeValue, Signature, Trap, TrapKind};
use wasmi::nan_preserving_float::*;

use crate::sdf::{SDFSample, SDFSurface};

use super::reinterpret_i32_as_u32;
use super::reinterpret_u32_as_i32;

/// Loads the given bytes as a WebAssembly module that is then queried to satisfy the SDF trait.
pub fn load_sdf_wasm(wasm_bytes: &[u8]) -> anyhow::Result<Box<dyn SDFSurface>> {
    let module = Module::from_buffer(wasm_bytes)?;
    let mut imports = ImportsBuilder::default();
    // HACK: Pushing default resolvers (warning/error-only) for known module names
    imports.push_resolver("env", &RuntimeModuleImportResolver);
    imports.push_resolver("__wbindgen_placeholder__", &RuntimeModuleImportResolver);
    imports.push_resolver("__wbindgen_externref_xform__", &RuntimeModuleImportResolver);
    let instance = ModuleInstance::new(&module, &imports)?
        .run_start(&mut MyExternals)?;
    Ok(Box::new(WasiSDF {
        wasm_instance: instance,
        sdf_id: 0,
    }))
}

#[derive(Debug, Clone)]
struct WasiSDF {
    wasm_instance: ModuleRef,
    sdf_id: u32,
}

impl SDFSurface for WasiSDF {
    // TODO: Macros for implementing these methods?

    fn bounding_box(&self) -> [Vector3<f32>; 2] {
        match self.wasm_instance.invoke_export(
            "bounding_box", &[
                RuntimeValue::I32(reinterpret_u32_as_i32(self.sdf_id))
            ], &mut MyExternals)
            .map_err(anyhow::Error::from)
            .and_then(|out| {
                let out = out.ok_or_else(|| anyhow!("No output for bounding_box"))?;
                let pointer = extract_pointer(out)?;
                let memory = self.memory()?;
                let mut res = [Vector3::<f32>::zero(); 2];
                for i in 0u32..6 {
                    let mem_pointer = pointer as u32 + i * 4;
                    res[(i / 3) as usize][i as usize % 3] = memory.get_value(mem_pointer)?;
                }
                Ok(res)
            }) {
            Ok(bounding_box) => bounding_box,
            Err(err) => {
                tracing::error!("Failed to get bounding box of wasm SDF with ID {}: {}", self.sdf_id, err);
                [Vector3::zero(); 2]
            }
        }
    }

    fn sample(&self, p: Vector3<f32>, distance_only: bool) -> SDFSample {
        match self.wasm_instance.invoke_export(
            "sample", &[
                RuntimeValue::I32(reinterpret_u32_as_i32(self.sdf_id)),
                RuntimeValue::F32(F32::from(p.x)),
                RuntimeValue::F32(F32::from(p.y)),
                RuntimeValue::F32(F32::from(p.z)),
                RuntimeValue::I32(if distance_only { 1 } else { 0 }),
            ], &mut MyExternals)
            .map_err(anyhow::Error::from)
            .and_then(|out| {
                let out = out.ok_or_else(|| anyhow!("No output for sample"))?;
                let pointer = extract_pointer(out)? as u32;
                let memory = self.memory()?;
                Ok(SDFSample {
                    distance: memory.get_value(pointer)?,
                    color: Vector3::new(
                        memory.get_value(pointer + 4)?,
                        memory.get_value(pointer + 8)?,
                        memory.get_value(pointer + 12)?,
                    ),
                    metallic: memory.get_value(pointer + 16)?,
                    roughness: memory.get_value(pointer + 20)?,
                    occlusion: memory.get_value(pointer + 24)?,
                })
            }) {
            Ok(bounding_box) => bounding_box,
            Err(err) => {
                tracing::error!("Failed to get bounding box of wasm SDF with ID {}: {}", self.sdf_id, err);
                SDFSample::new(1.0, Vector3::zero())
            }
        }
    }
}

impl WasiSDF {
    fn memory(&self) -> anyhow::Result<MemoryRef> {
        Ok(self.wasm_instance.export_by_name("memory")
            .ok_or_else(|| anyhow!("Export with name \"memory\" not found"))?
            .as_memory()
            .ok_or_else(|| anyhow!("Export with name \"memory\" is not a memory"))?
            .clone())
    }
}


fn extract_pointer(v: RuntimeValue) -> anyhow::Result<u32> {
    match v {
        RuntimeValue::I32(v) => Ok(reinterpret_i32_as_u32(v)),
        _ => Err(anyhow!("Invalid output type: expecting i32")),
    }
}

struct RuntimeModuleImportResolver;

impl ModuleImportResolver for RuntimeModuleImportResolver {
    fn resolve_func(
        &self,
        field_name: &str,
        signature: &Signature,
    ) -> Result<FuncRef, Error> {
        tracing::warn!("WebAssembly should not need any external functions, but it requires {}, with signature {:?}.", field_name, signature);
        Ok(FuncInstance::alloc_host(signature.clone(), 0))
    }
}

struct MyExternals;

impl Externals for MyExternals {
    fn invoke_index(
        &mut self,
        _index: usize,
        _args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        tracing::error!("WebAssembly should not execute any external functions, but it did.");
        Err(Trap::new(TrapKind::Unreachable))
    }
}