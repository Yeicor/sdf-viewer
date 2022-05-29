
mod app;
mod input;
mod metadata;
mod run;
mod cli;

// === Entry point for desktop ===
#[cfg(not(any(target_arch = "wasm32")))]
// #[tokio::main] // Not compatible with eframe :(
#[allow(dead_code)] // Fix for clippy
fn main() {
    run::native_main();
}
