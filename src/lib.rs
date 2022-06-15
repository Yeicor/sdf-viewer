extern crate core;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Not public as this is not a library
mod app;
mod metadata;
mod run;
mod cli;
mod sdf;

// === Entry point for web ===
#[cfg(target_arch = "wasm32")]
// #[wasm_bindgen(start)] // Disable auto-start to provide configuration and possibly use wasm-bin
// dgen-rayon
#[wasm_bindgen]
pub async fn run_app(canvas_id: String) -> Result<(), JsValue> {
    // console_log::init_with_level(log::Level::Debug).unwrap();
    tracing_wasm::set_as_global_default();
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    if let Some(app_creator) = run::setup_app() {
        eframe::start_web(&canvas_id, app_creator)?;
    }
    Ok(())
}

// === Entry point for android ===
#[cfg(target_os = "android")]
#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "full"))] // TODO: , logger(level = "debug", tag = "rust.sdf-viewer")
// #[tokio::main] // Not compatible with eframe :(
fn main() {
    run::native_main();
}