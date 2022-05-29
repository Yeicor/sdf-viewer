use std::path::PathBuf;

use clap::Parser;
use reqwest::Url;

use crate::app::SDFViewerApp;

#[derive(clap::Parser, Debug)]
pub struct CliApp {
    #[clap(subcommand)]
    pub watch: CliAppWatch,
}

#[derive(Parser, Debug)]
pub enum CliAppWatch {
    /// Display a WebAssembly file directly, listening for changes in the filesystem.
    /// Only works on native environments: for web (or all others) use the url mode
    // Could be simplified to automatic URL detection, but this is more explicit
    File(CliAppWatchFile),
    /// Display a WebAssembly file downloaded from the given URL, which listens for changes if ?wait=true is added.
    /// Works on all environments: requires the server command running (and watching the wasm file)
    Url(CliAppWatchUrl),
}

#[derive(Parser, Debug)]
pub struct CliAppWatchFile {
    /// The file to watch for changes and display
    #[clap(parse(from_os_str))]
    pub file: PathBuf,
}

#[derive(Parser, Debug)]
pub struct CliAppWatchUrl {
    /// The url to watch for changes and display
    #[clap(parse(try_from_str))]
    pub file: Url,
}

impl CliApp {
    /// Sets up a new instance of the application.
    pub fn init(&self, _app: &mut SDFViewerApp) {
        // TODO
    }
}
