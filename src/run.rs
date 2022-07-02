use eframe::AppCreator;
use klask::Settings;
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
                Box::new(SDFViewerApp::new(cc, app_args))
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
    // If no arguments are provided, run a GUI interface for configuring the CLI arguments ;)
    if std::env::args().nth(1).is_none() {
        #[cfg(feature = "klask")]
        klask::run_derived::<Cli, _>(Settings::default(), |_app| {
            // Ignore me: this block will be skipped the second time (as arguments will be set)
        });
    }
    // Run app
    let native_options = eframe::NativeOptions::default();
    if let Some(app_creator) = setup_app().await {
        eframe::run_native("SDF Viewer", native_options, app_creator);
    }
}