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
#[cfg(all(feature = "app", target_arch = "wasm32"))]
mod web;
#[cfg(all(feature = "app", target_arch = "wasm32"))]
pub use web::*;

// === Entry point for android ===
#[cfg(target_os = "android")]
#[cfg(any(feature = "app", feature = "server"))]
#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(android_logger::Config::default());
    log::info!("Starting android_main");
    println!("Starting android_main");
    
    use winit::platform::android::EventLoopBuilderExtAndroid;
    let _ign = run::native_main(true, Some(Box::new(|b| {
        b.with_android_app(app);
    })));

    println!("Exiting android_main")
}