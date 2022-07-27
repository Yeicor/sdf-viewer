//! Defines an SDF by interpreting a WebAssembly module. The module MUST adhere to the following specification.
//!
//! See [`super::demo::ffi`] for a valid rust implementation of this specification.
//!
//! # WebAssembly API specification
//!
//! The WebAssembly module:
//! - MUST NOT use any import from the host (it may have unused imports).
//! - MUST export (at least) the required methods from the [`SDF`](crate::sdf::SDFSurface) trait
//! (check the documentation of the trait).
//!
//! As an SDF can provide access to the whole hierarchy it contains, each function must have an extra
//! initial parameter indicating the ID (u32) of the SDF it refers to, with 0 being the root SDF.
//! The special case of the `children` method only returns these IDs.
//!
//! When multiple values need to be returned, like in the case of the `bounding_box` method, they must be
//! returned in a memory location, and the return value of the function (u32) must be the pointer to the location.
//!
//! The layout in memory is the following:
//! - Primitives: as i32 or f32 depending on the type of the primitive.
//! - Structs/fixed length arrays (Vector3, SDFSample, \[u8; 2]):
//!     - input: flattened attributes.
//!     - output: pointer to memory location with flattened attributes.
//! - Array/String:
//!     - input: only Vector3 for now, so flattener.
//!     - output: pointer to (pointer to another memory location + length of the array in bytes).
//! - Enums: same as structs (known length), but with an extra initial u32 indicating the ordinal.
//!     - Note that Result and Option are just an enum.
//!
//! Some types may be wrapped in a Box (pointer to heap memory) if required, see reference implementation.
//!
//! If the <method>_free is available, it will be called after the data is used with the only argument
//! of the previously returned value by that method. It should be used to properly free the memory.
//!

use crate::sdf::SDFSurface;

mod native;

/// See [`load_sdf_wasm`] for more information.
pub async fn load_sdf_wasm_send_sync(wasm_bytes: &[u8]) -> anyhow::Result<Box<dyn SDFSurface + Send + Sync>> {
    native::load_sdf_wasm_send_sync(wasm_bytes).await
}

/// Loads the given bytes as a WebAssembly module that is then queried to satisfy the SDF trait.
///
/// It uses the default WebAssembly interpreter for the platform.
#[allow(dead_code)] // Not used in the current implementation because Send + Sync is needed for the WebAssembly engine.
pub async fn load_sdf_wasm(wasm_bytes: &[u8]) -> anyhow::Result<Box<dyn SDFSurface>> {
    native::load_sdf_wasm(wasm_bytes).await
}

fn reinterpret_u32_as_i32(sdf_id: u32) -> i32 {
    i32::from_le_bytes(sdf_id.to_le_bytes()) // Reinterpret without modifications
}

fn reinterpret_i32_as_u32(sdf_id: i32) -> u32 {
    u32::from_le_bytes(sdf_id.to_le_bytes()) // Reinterpret without modifications
}