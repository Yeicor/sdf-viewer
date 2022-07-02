use clap::Parser;
use eframe::egui;
use eframe::egui::Context;
use eframe::egui::Widget;
use ehttp::Request;
use klask::app_state::AppState;

use crate::app::SDFViewerApp;
use crate::sdf::demo::SDFDemo;
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

pub enum SDFViewerAppSettings {
    /// The settings window is closed.
    Configured { settings: CliApp },
    /// The settings window is open, but we still remember old values in case editing is cancelled.
    Configuring { previous: CliApp, editing: klask::app_state::AppState<'static> },
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
        let args = editing.get_cmd_args(vec!["app".to_string()])
            .or_else(|err| {
                tracing::error!("Failed to parse CLI arguments (from settings GUI): {:?}", err);
                Ok::<_, ()>(vec![])
            }).unwrap();
        <CliApp as clap::Parser>::try_parse_from(&args)
            .or_else(|err| {
                tracing::error!("Failed to parse CLI arguments (from settings GUI): {:?} --> {:?}", args, err);
                Ok::<_, ()>(CliApp::default())
            }).unwrap()
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
                ui.columns(2, |ui| {
                    if ui[0].button("Cancel").clicked() {
                        return Some(SDFViewerAppSettings::Configured { settings: CliApp::clone(previous.as_ref().unwrap()) });
                    }
                    if ui[1].button("Apply").clicked() {
                        let new_settings = Self::editing_to_instance(editing.as_ref().unwrap());
                        return Some(SDFViewerAppSettings::Configured { settings: new_settings });
                    }
                    None
                })

                // TODO: The command line arguments that would generate this config (for copying)
            }).and_then(|r| r.inner.flatten());
        if prev_open && !open { // Closing the window is the same as cancelling
            change_state = Some(SDFViewerAppSettings::Configured { settings: CliApp::clone(previous.as_ref().unwrap()) });
        }

        if let Some(new_state) = change_state {
            if app.settings_values.previous() != &new_state.current() {
                new_state.current().apply(app)
                // TODO: Auto-refresh the whole SDF.
            }
            app.settings_values = new_state;
        }
    }
}
