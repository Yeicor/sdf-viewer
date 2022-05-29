// Entry point for wasm
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod app;
mod input;
mod metadata;

// === Entry point for web ===
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    // console_log::init_with_level(log::Level::Debug).unwrap();
    tracing_wasm::set_as_global_default();
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    eframe::start_web(env!("CARGO_PKG_NAME"), Box::new(|cc| Box::new(app::SDFViewerApp::new(cc))))?;
    Ok(())
}

// === Entry point for android ===
#[cfg(target_os = "android")]
#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "full"))] // TODO: , logger(level = "debug", tag = "rust.sdf-viewer")
// #[tokio::main] // Not compatible with eframe :(
pub fn main() {
    crate::main();
}
