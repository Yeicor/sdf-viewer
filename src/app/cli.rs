use std::path::PathBuf;

use cgmath::Vector3;
use clap::Parser;
use reqwest::Url;

use crate::app::SDFViewerApp;
use crate::sdf::{SdfSample, SDFSurface};
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

/// Redirects the implementation for the configured SDF provider for the App
impl SDFSurface for CliSDFProvider {
    fn bounding_box(&self) -> [Vector3<f32>; 2] {
        match self {
            CliSDFProvider::Demo(s) => s.bounding_box(),
            _ => todo!()
        }
    }

    fn sample(&self, p: Vector3<f32>, distance_only: bool) -> SdfSample {
        match self {
            CliSDFProvider::Demo(s) => s.sample(p, distance_only),
            _ => todo!()
        }
    }

    fn normal(&self, p: Vector3<f32>, eps: Option<f32>) -> Vector3<f32> {
        match self {
            CliSDFProvider::Demo(s) => s.normal(p, eps),
            _ => todo!()
        }
    }
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
        app.scene_mut(|scene| scene.sdf = Box::new(self.sdf_provider.clone()));
        // TODO: More settings
    }
}
