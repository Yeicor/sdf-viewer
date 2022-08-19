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
//! The WASM module may also optionally export the init() method with no arguments and no return value,
//! which will be called once before any other method.
//!


pub(crate) mod load;
mod native;
mod util;
