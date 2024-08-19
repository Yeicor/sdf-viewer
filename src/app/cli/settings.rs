use eframe::egui::{Context, Response, TextEdit, Ui, Widget, WidgetText};
use eframe::egui;
use klask::app_state::AppState;
use klask::settings::Localization;
use once_cell::sync::OnceCell;

pub enum SettingsWindow<P> {
    /// The settings window is closed.
    Configured { settings: P },
    /// The settings window is open, but we still remember old values in case editing is cancelled.
    Configuring { previous: P, editing: AppState<'static> },
}

impl<P: clap::Parser + Clone + Default + PartialEq> SettingsWindow<P> {
    pub fn previous(&self) -> &P {
        match self {
            SettingsWindow::Configured { settings } => settings,
            SettingsWindow::Configuring { previous, .. } => previous,
        }
    }

    pub fn current(&self) -> P {
        match self {
            SettingsWindow::Configured { settings } => settings.clone(),
            SettingsWindow::Configuring { editing, .. } => Self::editing_to_instance(editing),
        }
    }

    fn editing_to_instance(editing: &AppState) -> P {
        let args = editing.get_cmd_args(vec!["app".to_string()]).unwrap_or_else(|err| {
            tracing::error!("Failed to parse CLI arguments (from settings GUI): {}", err);
            vec![]
        });
        P::try_parse_from(&args).unwrap_or_else(|err| {
            tracing::error!("Failed to parse CLI arguments (from settings GUI): {:?} --> {}", args, err);
            P::default()
        })
    }

    /// Shows the settings window button that toggles its visibility.
    pub fn show_window_button(&mut self, ui: &mut Ui, window_name: impl Into<WidgetText>) {
        // If settings are not already open, enable the settings button.
        let (enabled, values) = match &self {
            Self::Configured { settings: values } => (true, values.clone()),
            Self::Configuring { previous, .. } => (false, previous.clone()),
        };
        ui.add_enabled_ui(enabled, |ui| {
            let settings_button = egui::menu::menu_button(ui, window_name, |_ui| {});
            if settings_button.response.clicked() {
                static LOC_SETTINGS: OnceCell<Localization> = OnceCell::new();
                let loc_settings = LOC_SETTINGS.get_or_init(Localization::default);
                *self = Self::Configuring {
                    previous: values.clone(),
                    // TODO(klask): load initial values from the previous settings
                    editing: AppState::new(&P::command(), loc_settings),
                };
            }
        });
    }

    /// Show the widget (or do nothing if closed). It will return a value when a change needs to be applied.
    pub fn show(&mut self, ctx: &Context, window_name: impl Into<WidgetText>, mut cmd_name: Vec<String>,
                show_web_link: bool, update_if_unchanged: bool) -> Option<P> {
        let (mut open, previous, mut editing) = match self {
            SettingsWindow::Configuring { previous, editing } => (true, Some(previous), Some(editing)),
            _ => (false, None, None),
        };

        let prev_open = open;
        let mut cancelled = false;
        let mut change_state = egui::Window::new(window_name)
            .open(&mut open)
            .resizable(true)
            .scroll([true, true])
            .show(ctx, |ui| {
                let editing = match editing.as_mut() {
                    Some(_editing) => _editing,
                    None => return None,
                };

                // # The main settings widget
                <&mut AppState>::ui(editing, ui);

                // # The cancel and apply buttons
                let action = ui.columns(2, |ui| {
                    if ui[0].button("Cancel").clicked() {
                        cancelled = true;
                        return Some(SettingsWindow::Configured { settings: P::clone(&**previous.as_ref().unwrap()) });
                    }
                    if ui[1].button("Apply").clicked() {
                        let new_settings = Self::editing_to_instance(editing);
                        return Some(SettingsWindow::Configured { settings: new_settings });
                    }
                    None
                });

                // The command line arguments that would generate this config (for copying)
                cmd_name.insert(0, env!("CARGO_PKG_NAME").to_string());
                let cmd_args = editing
                    .get_cmd_args(cmd_name)
                    .unwrap_or_else(|err| vec![format!("Invalid configuration: {err}")]);
                ui.horizontal_wrapped(|ui| {
                    ui.label("CLI: ");
                    let mut cmd_args_str = cmd_args.join(" "); // TODO: Proper escaping
                    TextEdit::singleline(&mut cmd_args_str) // Not editable, but copyable
                        .desired_width(ui.available_width())
                        .ui(ui)
                });
                if show_web_link {
                    ui.horizontal_wrapped(|ui| {
                        ui.label("WEB: ");
                        let cmd_args_str = cmd_args.iter().skip(2)
                            .map(|arg| format!("cli{arg}"))
                            .collect::<Vec<_>>()
                            .join("&"); // TODO: Proper escaping
                        let mut text_buf = "?".to_string() + &cmd_args_str;
                        TextEdit::singleline(&mut text_buf) // Not editable, but copyable
                            .desired_width(ui.available_width())
                            .ui(ui)
                    });
                }

                action
            }).and_then(|r| r.inner.flatten());
        cancelled |= prev_open && !open;
        if cancelled { // Closing the window is the same as cancelling
            change_state = Some(SettingsWindow::Configured { settings: P::clone(previous.as_ref()?) });
        }

        if let Some(new_state) = change_state {
            let previous = self.previous().clone();
            *self = new_state;
            // TODO: Apply only the modified settings (know which settings were left unchanged / as default values)
            let current = self.current();
            if !cancelled && (update_if_unchanged || previous != current) { Some(current) } else { None }
        } else { None }
    }
}
