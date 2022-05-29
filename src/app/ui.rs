use eframe::{egui, Frame};
use eframe::egui::{Context, ScrollArea};
use eframe::egui::panel::Side;
use eframe::egui::util::hash;


use crate::app::SDFViewerApp;

/// The user interface of the application, using egui.
#[derive(Default)]
pub struct SDFViewerAppUI {}

impl SDFViewerAppUI {
    /// "Renders" (actually just prepares for render) the user interface.
    pub fn render(&mut self, _app: &SDFViewerApp, ctx: &Context, _frame: &mut Frame) {
        egui::SidePanel::new(Side::Left, hash(""))
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
    }
}

impl SDFViewerAppUI {}