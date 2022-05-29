use eframe::AppCreator;
use tracing::info;
use crate::app::SDFViewerApp;

use crate::cli::{Cli, Commands};

/// All entry-points redirect here after platform-specific initialization and
/// before platform-specific window start (which may be cancelled if None is returned).
pub fn setup_app() -> Option<AppCreator> {
    // Parse CLI arguments here to initialize the app.
    // Note that the window is already created and we could do this earlier.
    let args = Cli::parse_args();
    info!("Arguments: {:?}", args);

    match args.command {
        Commands::App(app_args) => {
            Some(Box::new(move |cc| {
                let mut app = SDFViewerApp::new(cc);
                app_args.init(&mut app); // Initialize the app with the given arguments.
                Box::new(app)
            }))
        }
        Commands::Server(_srv) => {
            // TODO: Run the server (until
            None
        }
    }
}

// === Native entry-points redirect here ===
#[cfg(not(any(target_arch = "wasm32")))]
#[allow(dead_code)] // False positive
pub fn native_main() {
    // Setup logging
    tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::default())
        .expect("Failed to set global default subscriber");
    // Run app
    let native_options = eframe::NativeOptions::default();
    if let Some(app_creator) = setup_app() {
        eframe::run_native("SDF Viewer", native_options, app_creator);
    }
}