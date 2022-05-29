mod app;
mod input;

// === Entry point for desktop ===
#[cfg(not(any(target_arch = "wasm32")))]
// #[tokio::main] // Not compatible with eframe :(
#[allow(dead_code)] // Fix for clippy
pub(crate) fn main() {
    // Setup logging
    tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::default())
        .expect("Failed to set global default subscriber");
    // Run app
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("SDF Viewer", native_options, Box::new(|cc|
        Box::new(app::SDFViewerApp::new(cc))));
}
