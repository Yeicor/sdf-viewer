use eframe::{egui, Frame};
use eframe::egui::{Context, ProgressBar, ScrollArea};
use eframe::egui::panel::{Side, TopBottomSide};
use eframe::egui::util::hash;

use crate::app::SDFViewerApp;

/// The user interface of the application, using egui.
#[derive(Default)]
pub struct SDFViewerAppUI {
    /// If set, indicates the load progress of the SDF in the range [0, 1] and the display text.
    pub progress: Option<(f32, String)>,
}

impl SDFViewerAppUI {
    /// "Renders" (actually just prepares for render) the user interface.
    pub fn render(&mut self, _app: &SDFViewerApp, ctx: &Context, _frame: &mut Frame) {
        // Top panel for the menu bar
        egui::TopBottomPanel::new(TopBottomSide::Top, hash("top"))
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    egui::menu::menu_button(ui, "File", |ui| {
                        egui::menu::menu_button(ui, "Open SDF...", |_ui| {
                            // Unimplemented
                        });
                    });
                });
            });

        // Main side panel for configuration.
        egui::SidePanel::new(Side::Left, hash("left"))
            .show(ctx, |ui| {
                ScrollArea::new([false, true]).show(ui, |ui| {
                    ui.heading("Connection"); // TODO: Accordions with status emojis/counters...
                    ui.heading("Parameters");
                    ui.heading("Hierarchy");
                    ui.heading("Settings");
                    ui.horizontal(|ui| {
                        ui.label("Theme:");
                        egui::widgets::global_dark_light_mode_switch(ui);
                        // Read-only app access possible, to request operations done by the app.
                        // info!("App access: {:?}", _app.scene.borrow().volume.material.color);
                    });
                })
            });

        // Bottom panel, containing the progress bar if applicable.
        egui::TopBottomPanel::new(TopBottomSide::Bottom, hash("bottom"))
            .frame(egui::containers::Frame::default().inner_margin(0.0))
            .min_height(0.0) // Hide when unused
            .show(ctx, |ui| {
                if let Some((progress, text)) = self.progress.as_ref() {
                    ui.add(ProgressBar::new(*progress).text(text.clone()).animate(true));
                }
            });
    }
}

impl SDFViewerAppUI {}