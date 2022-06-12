
pub mod app;
pub mod input;
pub mod metadata;
pub mod run;
pub mod cli;
pub mod sdf;

// === Entry point for desktop ===
#[cfg(not(any(target_arch = "wasm32")))]
// #[tokio::main] // Not compatible with eframe :(
#[allow(dead_code)] // Fix for clippy
fn main() {
    run::native_main();
}
