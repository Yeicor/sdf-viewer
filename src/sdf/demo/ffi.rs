use crate::sdf::demo::SDFDemo;
use crate::sdf::ffi::set_root_sdf;

/// Entrypoint: only needs to set the root SDFSurface implementation.
#[no_mangle]
pub extern "C" fn init() {
    set_root_sdf(Box::new(SDFDemo::default()));
}