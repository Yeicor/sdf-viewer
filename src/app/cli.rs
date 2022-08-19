use std::sync::mpsc;

use clap::Parser;
use eframe::egui;
use eframe::egui::{Context, TextEdit};
use eframe::egui::Widget;
use klask::app_state::AppState;

use crate::app::SDFViewerApp;
use crate::sdf::demo::SDFDemo;
use crate::sdf::wasm::load;

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
                load::load_sdf_from_path_or_url(sender_of_updates, watch.url);
            }
        }
        // TODO: More settings
    }
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
                ui.horizontal_wrapped(|ui| {
                    ui.label("WEB: ");
                    let cmd_args_str = cmd_args[1..].iter()
                        .map(|arg| format!("cli{}", arg))
                        .collect::<Vec<_>>()
                        .join("&"); // TODO: Proper escaping
                    let mut text_buf = "?".to_string() + &cmd_args_str;
                    TextEdit::singleline(&mut text_buf) // Not editable, but copyable
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
