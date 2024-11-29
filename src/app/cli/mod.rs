use clap::Parser;
use tokio::sync::mpsc;

use crate::app::SDFViewerApp;
use crate::sdf::demo::SDFDemo;
use crate::sdf::wasm::load;

pub(crate) mod settings;

#[derive(clap::Parser, Debug, Clone, PartialEq, Eq, Default)]
pub struct CliApp {
    /// The maximum number of voxels to compute per side of the SDF.
    #[clap(long, default_value = "64")]
    pub max_voxels_side: usize,
    /// The number of passes to load the SDF.
    /// Initial passes use a lower resolution to give feedback faster.
    #[clap(long, default_value = "2")]
    pub loading_passes: usize,
    /// The SDF provider to use.
    #[clap(subcommand)]
    pub sdf_provider: CliSDFProvider,
}

impl CliApp {
    /// Sets up a new instance of the application.
    pub fn apply(&self, app: &mut SDFViewerApp) {
        // Configure the initial SDF provider (may be changed later)
        let (sender_of_updates, receiver_of_updates) = mpsc::channel(16);
        app.set_root_sdf_loading_manager(receiver_of_updates);
        match self.sdf_provider.clone() {
            CliSDFProvider::Demo(s) => app.set_root_sdf(Box::new(s), Some(self.max_voxels_side), Some(self.loading_passes)),
            CliSDFProvider::Url(watch) => {
                load::load_sdf_from_path_or_url(sender_of_updates, watch.url);
            }
        }
        // TODO: Many more settings! (should be easy to add and automatically update the CLI and UI)
    }
}

#[derive(Parser, Debug, Clone, PartialEq, Eq)]
pub enum CliSDFProvider {
    /// Display a WebAssembly file downloaded from the given URL.
    Url(CliAppWatchUrl),
    /// An embedded demo SDF provider for testing and feature-showcasing purposes
    Demo(SDFDemo),
}

impl Default for CliSDFProvider {
    fn default() -> Self {
        Self::Demo(SDFDemo::default())
    }
}

impl CliSDFProvider {
    /// Get the URL to download the SDF from, if available.
    pub fn url(&self) -> Option<&str> {
        match self {
            CliSDFProvider::Url(watch) => Some(&watch.url),
            _ => None,
        }
    }
}

#[derive(Parser, Debug, Clone, PartialEq, Eq)]
pub struct CliAppWatchUrl {
    /// The url where a WebAssembly file representing a SDF is hosted.
    ///
    /// Supported schemes are http(s):// and file://.
    /// If no scheme is given, it is assumed to be a local file in native and a relative URL in web.
    ///
    /// The app expects the server to listen for file changes if ?wait=true is appended to the URL, but
    /// detects servers that don't support this and displays a warning, disabling this feature.
    #[clap(parse(try_from_str))]
    pub url: String,
}
