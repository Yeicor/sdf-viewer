use std::sync::Arc;

use eframe::{egui, Storage};
use eframe::egui::{Color32, Frame, ProgressBar, ScrollArea, Stroke, Ui, Vec2};
use eframe::egui::collapsing_header::CollapsingState;
use eframe::egui::panel::{Side, TopBottomSide};
use eframe::egui::util::hash;
use tracing::info;

use scene::SDFViewerAppScene;

use crate::metadata::log_version_info;
use crate::sdf::SDFSurface;

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
        let slf = Self {
            progress: None,
        };

        // In order to configure the 3D scene after initialization, we need to create a new scene now.
        // Warning: future rendering must be done from this thread, or a new scene will be created.
        SDFViewerAppScene::from_glow_context_thread_local(&cc.gl, |_scene| {});

        slf
    }

    fn ui_three_d_scene_widget(&mut self, ui: &mut Ui) {
        let (rect, response) = ui.allocate_exact_size(
            ui.available_size(), egui::Sense::click_and_drag());
        // Synchronize the scene information (from the previous frame, no way to know the future)
        self.progress = self.scene_mut(|scene| scene.load_progress()).unwrap_or(None);
        // Queue the rendering of the scene
        ui.painter().add(egui::PaintCallback {
            rect,
            callback: Arc::new(move |info, painter| {
                let painter = painter.downcast_mut::<egui_glow::Painter>().unwrap();
                let response = response.clone();
                SDFViewerAppScene::from_glow_context_thread_local(painter.gl(), move |scene| {
                    scene.render(info, &response);
                });
            }),
        });
    }

    fn ui_create_hierarchy(sdf: &dyn SDFSurface, ui: &mut Ui) {
        let id = ui.make_persistent_id(format!("sdf-hierarchy-{}", sdf.id()));
        let children = sdf.children();
        let render_child = |ui: &mut Ui| {
            ui.label(sdf.name());
        };
        if children.is_empty() {
            render_child(ui);
        } else {
            CollapsingState::load_with_default_open(ui.ctx(), id, true)
                .show_header(ui, render_child)
                .body(move |ui| {
                    for child in children {
                        Self::ui_create_hierarchy(child.as_ref(), ui);
                    }
                });
        }
    }

    pub fn scene_mut<R>(
        &mut self,
        f: impl FnOnce(&mut SDFViewerAppScene) -> R,
    ) -> Option<R> {
        SDFViewerAppScene::read_context_thread_local(f)
    }
}

impl eframe::App for SDFViewerApp {
    #[profiling::function]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel for the menu bar
        egui::TopBottomPanel::new(TopBottomSide::Top, hash("top"))
            .show(ctx, |ui| {
                ScrollArea::new([true, true]).show(ui, |ui| {
                    egui::menu::bar(ui, |ui| {
                        egui::menu::menu_button(ui, "File", |ui| {
                            egui::menu::menu_button(ui, "Open SDF...", |_ui| {
                                // TODO: Open and swap the new SDF manually inserted (url/file)
                            });
                        });
                        // Add an spacer to right-align some options
                        ui.allocate_space(Vec2::new(ui.available_width() - 26.0, 1.0));
                        egui::widgets::global_dark_light_mode_switch(ui);
                    });
                });
            });

        // Main side panel for configuration.
        egui::SidePanel::new(Side::Left, hash("left"))
            .show(ctx, |ui| {
                ScrollArea::new([true, true]).show(ui, |ui| {
                    self.scene_mut(move |scene| {
                        Self::ui_create_hierarchy(scene.sdf.as_ref(), ui);
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
            .frame(Frame::none().inner_margin(0.0))
            .show(ctx, |ui| {
                Frame::canvas(ui.style())
                    // FIXME: 0 alpha is the only way the contents render, but it also renders
                    //  removed widgets, and 1 alpha is the only way the removed widgets don't render.
                    .fill(Color32::from_rgba_unmultiplied(0, 0, 0, 0))
                    .inner_margin(0.0)
                    .outer_margin(0.0)
                    .stroke(Stroke::none())
                    .show(ui, |ui| {
                        self.ui_three_d_scene_widget(ui);
                    });
            });
    }

    #[profiling::function]
    fn save(&mut self, _storage: &mut dyn Storage) {
        // TODO: Store app state, indexed by the loaded SDF?
        // storage.set_string()
    }
}

