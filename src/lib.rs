// Entry point for wasm
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use run::MyEguiApp;

mod run;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    console_log::init_with_level(log::Level::Debug).unwrap();

    use log::info;
    info!("Logging works!");

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    eframe::start_web(env!("CARGO_PKG_NAME"), Box::new(|cc| Box::new(MyEguiApp::new(cc))))?;
    Ok(())
}

#[cfg(target_os = "android")]
#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "full"))] // TODO: , logger(level = "debug", tag = "rust.sdf-viewer")
// #[tokio::main] // Not compatible with eframe :(
pub fn main() {
    use log::info;
    info!("Logging works!");

    let native_options = eframe::NativeOptions::default();
    eframe::run_native("SDF Viewer", native_options, Box::new(|cc| Box::new(MyEguiApp::new(cc))));
}
