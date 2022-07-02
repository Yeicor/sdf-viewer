extern crate core;

// === KEEP MODULES IN SYNC WITH main.rs ===
#[cfg(feature = "sdf")] // Only public module to export the SDF trait and implementations.
pub mod sdf;
#[cfg(feature = "app")]
mod app;
#[cfg(feature = "server")]
mod server;
#[cfg(any(feature = "app", feature = "server"))]
mod metadata;
#[cfg(any(feature = "app", feature = "server"))]
mod run;
#[cfg(any(feature = "app", feature = "server"))]
mod cli;

// === Entry point for web ===
#[cfg(target_arch = "wasm32")]
#[cfg(any(feature = "app", feature = "server"))]
// #[wasm_bindgen(start)] // Disable auto-start to provide configuration and possibly use wasm-bindgen-rayon
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn run_app(canvas_id: String) -> Result<(), wasm_bindgen::prelude::JsValue> {
    // console_log::init_with_level(log::Level::Debug).unwrap();
    tracing_wasm::set_as_global_default();
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    if let Some(app_creator) = run::setup_app().await {
        eframe::start_web(&canvas_id, app_creator)?;
    }
    Ok(())
}
