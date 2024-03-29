#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// === KEEP MODULES IN SYNC WITH lib.rs ===
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

// === Entry point for desktop ===
#[cfg(not(any(target_arch = "wasm32")))]
#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run::native_main(false, None).await;
}
