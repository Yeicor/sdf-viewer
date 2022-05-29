// === Native entry-points redirect here ===
#[cfg(not(any(target_arch = "wasm32")))]
pub fn main() {
    // Setup logging
    tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::default())
        .expect("Failed to set global default subscriber");
    // Run app
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("SDF Viewer", native_options, Box::new(|cc|
        Box::new(crate::app::SDFViewerApp::new(cc))));
}