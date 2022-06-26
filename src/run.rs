use eframe::AppCreator;
use tracing::info;

use crate::app::SDFViewerApp;
use crate::cli::{Cli, Commands};
use crate::metadata::log_version_info;

/// All entry-points redirect here after platform-specific initialization and
/// before platform-specific window start (which may be cancelled if None is returned).
pub async fn setup_app() -> Option<AppCreator> {
    // Test logging and provide useful information
    log_version_info();

    // Parse CLI arguments here to initialize the app.
    // Note that the window is already created and we could do this earlier.
    let args = Cli::parse_args();
    info!("Arguments: {:?}", args);

    match args.command {
        #[cfg(feature = "app")]
        Commands::App(app_args) => {
            Some(Box::new(move |cc| {
                let mut app = SDFViewerApp::new(cc);
                app_args.init(&mut app); // Initialize the app with the given arguments.
                Box::new(app)
            }))
        }
        #[cfg(feature = "server")]
        Commands::Server(_srv) => {
            // TODO: Run the server
            None
        }
    }
}

// === Native entry-points redirect here ===
#[cfg(not(any(target_arch = "wasm32")))]
#[allow(dead_code)] // False positive
pub async fn native_main() {
    // Setup logging
    tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::default())
        .expect("Failed to set global default subscriber");
    // Run app
    let native_options = eframe::NativeOptions::default();
    if let Some(app_creator) = setup_app().await {
        eframe::run_native("SDF Viewer", native_options, app_creator);
    }
}