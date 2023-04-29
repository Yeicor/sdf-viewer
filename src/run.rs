use tracing::info;

use crate::cli::{Cli, Commands};
use crate::metadata::log_version_info;

#[cfg(feature = "app")]
type AppCreator = Option<eframe::AppCreator>;
#[cfg(not(feature = "app"))]
type AppCreator = Option<()>;

/// All entry-points redirect here after platform-specific initialization and
/// before platform-specific window start (which may be cancelled if None is returned).
pub async fn setup_app() -> AppCreator {
    // Test logging and provide useful information
    log_version_info();

    // Parse CLI arguments here to initialize the app.
    // Note that the window is already created and we could do this earlier.
    let args = Cli::parse_args();
    info!("Arguments: {:?}", args);

    match args.command {
        #[cfg(feature = "app")]
        Commands::App(app_args) => { // Start the GUI app
            Some(Box::new(move |cc| {
                Box::new(crate::app::SDFViewerApp::new(cc, app_args))
            }))
        }
        #[cfg(feature = "server")]
        Commands::Server(srv) => { // Runs the server forever
            srv.run().await;
            None
        }
        #[cfg(feature = "meshers")]
        Commands::Mesh(mesher) => { // Run the meshing algorithm and exit
            mesher.run_cli().await.unwrap();
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
        klask::run_derived::<Cli, _>(klask::Settings::default(), |_app| {
            // Ignore me: this block will be skipped the second time (as arguments will be set)
        });
    }
    // Run app
    #[allow(unused_variables)]
    if let Some(app_creator) = setup_app().await {
        #[cfg(feature = "app")]
        {
            let native_options = eframe::NativeOptions {
                depth_buffer: 16, // Needed for 3D rendering
                ..eframe::NativeOptions::default()
            };
            eframe::run_native("SDF Viewer", native_options, app_creator).unwrap();
        }
    }
}