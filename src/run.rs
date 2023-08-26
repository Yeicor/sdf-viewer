use std::future;
use std::future::Future;
use std::pin::Pin;

use tracing::info;

use crate::cli::{Cli, Commands};
use crate::metadata::log_version_info;

#[cfg(feature = "app")]
type AppCreator = Option<eframe::AppCreator>;
#[cfg(not(feature = "app"))]
type AppCreator = Option<()>;

#[cfg(feature = "app")]
type EventLoopBuilderHook = Option<eframe::EventLoopBuilderHook>;
#[cfg(not(feature = "app"))]
type EventLoopBuilderHook = Option<()>;

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

/// All entry-points redirect here after platform-specific initialization and
/// before platform-specific window start (which may be cancelled if None is returned).
#[allow(dead_code)] // Not important for this method
pub fn setup_app_sync() -> AppCreator {
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
        Commands::Server(srv) => {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    srv.run().await;
                    None
                })
        },
        #[cfg(feature = "meshers")]
        Commands::Mesh(mesher) => {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    mesher.run_cli().await.unwrap();
                    None
                })
        }
    }
}

// === Native entry-points redirect here ===
#[cfg(not(any(target_arch = "wasm32")))]
#[allow(dead_code)] // False positive
pub fn native_main(sync: bool, event_loop_builder: EventLoopBuilderHook) -> Pin<Box<dyn Future<Output=()>>> {
    // Setup logging
    tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::default())
        .expect("Failed to set global default subscriber");
    // If no arguments are provided, run a GUI interface for configuring the CLI arguments ;)
    if Cli::get_args().get(1).is_none() {
        #[cfg(feature = "klask")]
        klask::run_derived::<Cli, _>(klask::Settings::default(), |_app| {
            // Ignore me: this block will be skipped the second time (as arguments will be set)
        });
    }
    // Run app
    let app_creator_handler = |app_creator: AppCreator| {
        #[cfg(feature = "app")]
        {
            let native_options = eframe::NativeOptions {
                depth_buffer: 16, // Needed for 3D rendering
                event_loop_builder,
                ..eframe::NativeOptions::default()
            };
            println!("Starting native app");
            eframe::run_native("SDF Viewer", native_options, app_creator.unwrap()).unwrap();
            println!("Native app exited");
        }
    };
    if sync {
        if let Some(app_creator) = setup_app_sync() {
            app_creator_handler(Some(app_creator));
        }
        Box::pin(future::ready(()))
    } else {
        use futures_util::FutureExt;
        Box::pin(setup_app().map(move |t| {
            if let Some(app_creator) = t {
                app_creator_handler(Some(app_creator));
            }
        }))
    }
}