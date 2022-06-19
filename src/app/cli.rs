use std::path::PathBuf;

use clap::Parser;
use reqwest::Url;

use crate::app::SDFViewerApp;

use crate::sdf::demo::SDFDemo;

#[derive(clap::Parser, Debug)]
pub struct CliApp {
    #[clap(subcommand)]
    pub sdf_provider: CliSDFProvider,
}

#[derive(Parser, Debug, Clone)]
pub enum CliSDFProvider {
    /// An embedded demo SDF provider for testing purposes
    Demo(SDFDemo),
    /// Display a WebAssembly file directly, listening for changes in the local filesystem.
    /// Only works on native environments: for web (or all others) use the url mode
    // Could be simplified to automatic URL detection, but this is more explicit
    File(CliAppWatchFile),
    /// Display a WebAssembly file downloaded from the given URL, which listens for changes if ?wait=true is added.
    /// Works on all environments: requires the server command running (and watching the wasm file)
    Url(CliAppWatchUrl),
}

#[derive(Parser, Debug, Clone)]
pub struct CliAppWatchFile {
    /// The file to watch for changes and display
    #[clap(parse(from_os_str))]
    pub file: PathBuf,
}

#[derive(Parser, Debug, Clone)]
pub struct CliAppWatchUrl {
    /// The url to watch for changes and display
    #[clap(parse(try_from_str))]
    pub file: Url,
}

impl CliApp {
    /// Sets up a new instance of the application.
    pub fn init(&self, app: &mut SDFViewerApp) {
        // Configure the initial SDF provider (may be changed later)
        app.set_root_sdf(match self.sdf_provider.clone() {
            CliSDFProvider::Demo(s) => s,
            _ => todo!()
        });
        // TODO: More settings
    }
}
