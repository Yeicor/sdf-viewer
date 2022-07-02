use clap::Parser;
use ehttp::Request;

use crate::app::SDFViewerApp;
use crate::sdf::demo::SDFDemo;
use crate::sdf::wasm::load_sdf_wasm_send_sync;

#[derive(clap::Parser, Debug, Clone, PartialEq, Eq)]
pub struct CliApp {
    #[clap(subcommand)]
    pub sdf_provider: CliSDFProvider,
}

#[derive(Parser, Debug, Clone, PartialEq, Eq)]
pub enum CliSDFProvider {
    /// An embedded demo SDF provider for testing purposes
    Demo(SDFDemo),
    /// Display a WebAssembly file downloaded from the given URL.
    /// TODO: Automatically detect file:// URLs and watch them directly without using external tools
    Url(CliAppWatchUrl),
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

impl CliApp {
    /// Sets up a new instance of the application.
    pub fn apply(&self, app: &mut SDFViewerApp) {
        // Configure the initial SDF provider (may be changed later)
        match self.sdf_provider.clone() {
            CliSDFProvider::Demo(s) => app.set_root_sdf(Box::new(s)),
            CliSDFProvider::Url(watch) => {
                // TODO: Spawn async worker to read, parse, etc., while displaying a loading screen.
                // TODO: Watch file for changes
                // First, try to request the file as an URL on any platform.
                let (sender, promise) = poll_promise::Promise::new();
                app.set_root_sdf_loading(promise);
                ehttp::fetch(Request::get(watch.url.clone()), move |data| {
                    let fut = async move {
                        let res = match data {
                            Ok(resp) => {
                                // TODO: Avoid this blocking code...
                                load_sdf_wasm_send_sync(resp.bytes.as_slice()).await
                            }
                            Err(err_str) => Err(anyhow::anyhow!(err_str)),
                        };
                        match res {
                            Ok(sdf) => { // If successful, load it now
                                sender.send(sdf);
                            }
                            Err(err) => { // If not, try to load it as a local file on native platforms.
                                #[cfg(target_arch = "wasm32")]
                                {
                                    tracing::error!("Failed to load SDF from URL: {:?}", err);
                                }
                                #[cfg(not(target_arch = "wasm32"))]
                                {
                                    // TODO: Avoid this blocking code...
                                    let res = match std::fs::read(watch.url) {
                                        Ok(bytes) => {
                                            // TODO: Avoid this blocking code...
                                            load_sdf_wasm_send_sync(bytes.as_slice()).await
                                        }
                                        Err(err) => Err(anyhow::Error::from(err)),
                                    };
                                    match res {
                                        Ok(sdf) => {
                                            sender.send(sdf);
                                        }
                                        Err(err2) => {
                                            tracing::error!("Failed to load SDF from URL ({:?}) or file ({:?})", err, err2);
                                        }
                                    }
                                }
                            }
                        }
                    };
                    #[cfg(target_arch = "wasm32")]
                    {
                        wasm_bindgen_futures::spawn_local(fut);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        // ehttp::fetch creates a new thread on native, which needs a new tokio runtime!
                        tokio::runtime::Runtime::new().unwrap().block_on(fut);
                    }
                });
            }
        }
        // TODO: More settings
    }
}
