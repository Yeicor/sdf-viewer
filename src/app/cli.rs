use std::future::Future;
use std::sync::mpsc;

use clap::Parser;
use eframe::egui;
use eframe::egui::{Context, TextEdit};
use eframe::egui::Widget;
use ehttp::Request;
use klask::app_state::AppState;

use crate::app::SDFViewerApp;
use crate::metadata::short_version_info_is_ours;
use crate::sdf::demo::SDFDemo;
use crate::sdf::SDFSurface;
use crate::sdf::wasm::load_sdf_wasm_send_sync;

#[derive(clap::Parser, Debug, Clone, PartialEq, Eq, Default)]
pub struct CliApp {
    #[clap(subcommand)]
    pub sdf_provider: CliSDFProvider,
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
        let (sender_of_updates, receiver_of_updates) = mpsc::channel();
        app.set_root_sdf_loading_manager(receiver_of_updates);
        match self.sdf_provider.clone() {
            CliSDFProvider::Demo(s) => app.set_root_sdf(Box::new(s)),
            CliSDFProvider::Url(watch) => {
                // TODO: Spawn async worker to read, parse, etc., while displaying a loading screen.
                ehttp::fetch(Request::get(watch.url.clone()), move |data| {
                    handle_sdf_data_response(data, watch.url, sender_of_updates)
                });
            }
        }
        // TODO: More settings
    }
}

/// Native: creates a new runtime that blocks on the given task.
/// Web: spawns the asynchronous task (should not block the main thread)
pub fn block_on_or_spawn_async(fut: impl Future<Output=()> + 'static) {
    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(fut);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        // ehttp::fetch creates a new thread on native, which needs a new tokio runtime!
        tokio::runtime::Runtime::new().unwrap().block_on(fut);
    }
}

/// This is a helper function to load a SDF from a WebAssembly binary. It initially tries to load the
/// HTTP response as a WebAssembly binary, but falls back to loading it as a local file if that fails.
fn handle_sdf_data_response(data: ehttp::Result<ehttp::Response>, watch_url_closure: String,
                            sender_of_updates: mpsc::Sender<mpsc::Receiver<Box<dyn SDFSurface + Send + Sync>>>) {
    // First, try to request the file as an URL on any platform (with some fallbacks).
    let fut = async move {
        let (sender_single_update, receiver_single_update) = mpsc::channel();
        sender_of_updates.send(receiver_single_update).unwrap();
        let res = match data {
            Ok(resp) => {
                // If the server properly supports the ?watch query parameter, we can start checking for changes.
                tracing::info!("HTTP headers: {:?}", resp.headers);
                let supports_watching = resp.headers.get("server").map(|v|
                    short_version_info_is_ours(v)).unwrap_or(false);
                if supports_watching {
                    tracing::info!("Server supports watching for file changes, enabling continuous updates.");
                    // Queue a ?watch request to the server, which will wait for source updates, recompile and return the new WASM file!
                    let watch_url_closure_clone = watch_url_closure.clone();
                    ehttp::fetch(Request::get(watch_url_closure.clone() + "?watch"), move |data| {
                        handle_sdf_data_response(data, watch_url_closure_clone, sender_of_updates)
                    });
                } else {
                    // Otherwise, give up on continuous updates by dropping the sender_of_updates!
                    tracing::warn!("HTTP Server does not support watching for file changes, disabling continuous updates.");
                    drop(sender_of_updates); // This is not needed, but states what we want
                }
                // TODO: Avoid this blocking code...
                load_sdf_wasm_send_sync(resp.bytes.as_slice()).await
            }
            Err(err_str) => Err(anyhow::anyhow!(err_str)),
        };
        match res {
            Ok(sdf) => { // If successful, load it now
                sender_single_update.send(sdf).unwrap();
            }
            Err(err) => { // If not, try to load it as a local file on native platforms.
                #[cfg(target_arch = "wasm32")]
                {
                    tracing::error!("Failed to load SDF from URL: {:?}", err);
                    sender_single_update.send(unsafe {
                        // FIXME: Extremely unsafe code (forcing SDFDemo Send+Sync), but only used for this error path
                        std::mem::transmute(Box::new(SDFDemo::default()) as Box<dyn SDFSurface>)
                    }).unwrap();
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    // TODO: Avoid this blocking code...
                    let res = match std::fs::read(watch_url_closure) {
                        Ok(bytes) => {
                            // TODO: Avoid this blocking code...
                            load_sdf_wasm_send_sync(bytes.as_slice()).await
                        }
                        Err(err) => Err(anyhow::Error::from(err)),
                    };
                    match res {
                        Ok(sdf) => {
                            sender_single_update.send(sdf).unwrap();
                        }
                        Err(err2) => {
                            tracing::error!("Failed to load SDF from URL ({:?}) or file ({:?})", err, err2);
                            sender_single_update.send(unsafe {
                                // FIXME: Extremely unsafe code (forcing SDFDemo Send+Sync), but only used for this error path
                                std::mem::transmute(Box::new(SDFDemo::default()) as Box<dyn SDFSurface>)
                            }).unwrap();
                        }
                    }
                }
            }
        }
    };
    block_on_or_spawn_async(fut)
}

pub enum SDFViewerAppSettings {
    /// The settings window is closed.
    Configured { settings: CliApp },
    /// The settings window is open, but we still remember old values in case editing is cancelled.
    Configuring { previous: CliApp, editing: AppState<'static> },
}

impl SDFViewerAppSettings {
    pub fn previous(&self) -> &CliApp {
        match self {
            SDFViewerAppSettings::Configured { settings } => settings,
            SDFViewerAppSettings::Configuring { previous, .. } => previous,
        }
    }

    pub fn current(&self) -> CliApp {
        match self {
            SDFViewerAppSettings::Configured { settings } => settings.clone(),
            SDFViewerAppSettings::Configuring { editing, .. } => Self::editing_to_instance(editing),
        }
    }

    fn editing_to_instance(editing: &AppState) -> CliApp {
        let args = editing.get_cmd_args(vec!["app".to_string()]).unwrap_or_else(|err| {
            tracing::error!("Failed to parse CLI arguments (from settings GUI): {:?}", err);
            vec![]
        });
        <CliApp as clap::Parser>::try_parse_from(&args).unwrap_or_else(|err| {
            tracing::error!("Failed to parse CLI arguments (from settings GUI): {:?} --> {:?}", args, err);
            CliApp::default()
        })
    }

    pub fn show(app: &mut SDFViewerApp, ctx: &Context) {
        let (mut open, previous, mut editing) = match &mut app.settings_values {
            SDFViewerAppSettings::Configuring { previous, editing } => (true, Some(previous), Some(editing)),
            _ => (false, None, None),
        };

        let prev_open = open;
        let mut change_state = egui::Window::new("Settings")
            .open(&mut open)
            .resizable(true)
            .scroll2([true, true])
            .show(ctx, |ui| {
                // # The main settings widget
                <&mut AppState>::ui(editing.as_mut().unwrap(), ui);

                // # The cancel and apply buttons
                let action = ui.columns(2, |ui| {
                    if ui[0].button("Cancel").clicked() {
                        return Some(SDFViewerAppSettings::Configured { settings: CliApp::clone(previous.as_ref().unwrap()) });
                    }
                    if ui[1].button("Apply").clicked() {
                        let new_settings = Self::editing_to_instance(editing.as_ref().unwrap());
                        return Some(SDFViewerAppSettings::Configured { settings: new_settings });
                    }
                    None
                });

                // The command line arguments that would generate this config (for copying)
                let cmd_args = editing.as_mut().unwrap()
                    .get_cmd_args(vec![env!("CARGO_PKG_NAME").to_string()])
                    .unwrap_or_else(|err| vec![format!("Invalid configuration: {}", err)]);
                ui.horizontal_wrapped(|ui| {
                    ui.label("CLI: ");
                    let mut cmd_args_str = cmd_args.join(" "); // TODO: Proper escaping
                    TextEdit::singleline(&mut cmd_args_str) // Not editable, but copyable
                        .desired_width(ui.available_width())
                        .ui(ui)
                });

                action
            }).and_then(|r| r.inner.flatten());
        if prev_open && !open { // Closing the window is the same as cancelling
            change_state = Some(SDFViewerAppSettings::Configured { settings: CliApp::clone(previous.as_ref().unwrap()) });
        }

        if let Some(new_state) = change_state {
            if app.settings_values.previous() != &new_state.current() {
                // TODO: Apply only the modified settings (know which settings were left unchanged / as default values)
                new_state.current().apply(app)
                // TODO: Auto-refresh the whole SDF.
            }
            app.settings_values = new_state;
        }
    }
}
