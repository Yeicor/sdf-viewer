use run::MyEguiApp;

mod run;

// Entry point for non-wasm
#[cfg(not(any(target_arch = "wasm32")))]
// #[tokio::main] // Not compatible with eframe :(
#[allow(dead_code)] // Fix for clippy
fn main() {
    use log::info;
    info!("Logging works!");

    #[cfg(not(target_arch = "wasm32"))] // <- eframe::run_native does not exist for web
    {
        let native_options = eframe::NativeOptions::default();
        eframe::run_native("SDF Viewer", native_options, Box::new(|cc| Box::new(MyEguiApp::new(cc))));
    }
}
