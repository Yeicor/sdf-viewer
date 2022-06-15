use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use eframe::{egui, glow, Storage};
use eframe::egui::{Frame, ProgressBar, ScrollArea, Ui};
use eframe::egui::panel::{Side, TopBottomSide};
use eframe::egui::util::hash;
use three_d::Context;
use tracing::info;

use scene::SDFViewerAppScene;

use crate::metadata::log_version_info;

pub mod cli;
pub mod scene;

/// The main application state and logic.
/// As the application is mostly single-threaded, use RefCell for performance when interior mutability is required.
pub struct SDFViewerApp {
    /// If set, indicates the load progress of the SDF in the range [0, 1] and the display text.
    pub progress: Option<(f32, String)>,
}

impl SDFViewerApp {
    #[profiling::function]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Test logging and provide useful information
        log_version_info();

        // Default to dark mode if no theme is provided by the OS (or environment variables)
        if (cc.integration_info.prefer_dark_mode == Some(false) ||
            std::env::var("sdf_viewer_light_theme").is_ok()) &&
            std::env::var("sdf_viewer_dark_theme").is_err() { // TODO: Save & restore theme settings
            cc.egui_ctx.set_visuals(egui::Visuals::light());
        } else {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
        }

        info!("Initialization complete! Starting main loop...");
        Self {
            progress: None,
        }
    }

    fn scene_scene(&mut self, ui: &mut Ui) {
        let (rect, response) = ui.allocate_exact_size(
            ui.available_size(), egui::Sense::drag());
        // Synchronize the scene information (from the previous frame, no way to know the future)
        SCENE.with(|scene| {
            if let Some(scene) = scene.borrow().as_ref() {
                self.progress = scene.load_progress();
            }
        });
        // Queue the rendering of the scene
        ui.painter().add(egui::PaintCallback {
            rect,
            callback: Arc::new(move |info, painter| {
                let painter = painter.downcast_mut::<egui_glow::Painter>().unwrap();
                let response = response.clone();
                with_scene_context(painter.gl(), move |scene| {
                    scene.render(info, &response);
                });
            }),
        });
    }
}

impl eframe::App for SDFViewerApp {
    #[profiling::function]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
            .frame(Frame::default().inner_margin(0.0))
            .min_height(0.0) // Hide when unused
            .show(ctx, |ui| {
                if let Some((progress, text)) = self.progress.as_ref() {
                    // HACK: Setting animate to true forces the scene to render continuously,
                    // making the loading process continue a bit each frame.
                    ui.add(ProgressBar::new(*progress).text(text.clone()).animate(true));
                }
            });

        // 3D Scene main content
        egui::CentralPanel::default()
            .frame(Frame::default().inner_margin(0.0))
            .show(ctx, |ui| {
                Frame::canvas(ui.style()).show(ui, |ui| {
                    self.scene_scene(ui);
                });
            });
    }

    #[profiling::function]
    fn save(&mut self, _storage: &mut dyn Storage) {
        // TODO: Store app state, indexed by the loaded SDF?
        // storage.set_string()
    }
}

thread_local! {
    pub static SCENE: RefCell<Option<SDFViewerAppScene>> = RefCell::new(None);
}

/// We get a [`glow::Context`] from `eframe`, but we want a [`scene::Context`] for [`SDFViewerAppScene`].
/// This function is a helper to convert the [`glow::Context`] to the [`scene::Context`] only once
/// in a thread-safe way.
///
/// Sadly we can't just create a [`scene::Context`] in [`MyApp::new`] and pass it
/// to the [`egui::PaintCallback`] because [`scene::Context`] isn't `Send+Sync`, which
/// [`egui::PaintCallback`] is.
fn with_scene_context<R>(
    gl: &Rc<glow::Context>,
    f: impl FnOnce(&mut SDFViewerAppScene) -> R,
) -> R {
    SCENE.with(|scene| {
        let mut scene = scene.borrow_mut();
        let scene =
            scene.get_or_insert_with(|| {
                // HACK: need to convert the GL context from Rc to Arc (UNSAFE: likely double-free on app close)
                let gl = unsafe { std::mem::transmute(gl.clone()) };
                // Retrieve Three-D context from the egui context (thanks to the shared glow dependency).
                let three_d_ctx = Context::from_gl_context(gl).unwrap();
                // Create the Three-D scene (only the first time).
                SDFViewerAppScene::new(three_d_ctx)
            });
        f(scene)
    })
}
