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
        let web_options = eframe::WebOptions {
            ..eframe::WebOptions::default()
        };
        eframe::WebRunner::new().start(&canvas_id, web_options, app_creator).await?;
    }
    Ok(())
}

// === Entry point for android ===
#[cfg(target_os = "android")]
#[cfg(any(feature = "app", feature = "server"))]
#[no_mangle]
fn android_main(app: android_activity::AndroidApp) {
    android_logger::init_once(android_logger::Config::default());
    log::info!("Starting android_main");
    println!("Starting android_main");

    use winit::platform::android::EventLoopBuilderExtAndroid;
    let _ign = run::native_main(true, Some(Box::new(|b| {
        b.with_android_app(app);
    })));

    println!("Exiting android_main")
}