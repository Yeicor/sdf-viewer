use std::sync::Arc;

use eframe::{egui, Storage};
use eframe::egui::{Color32, Frame, ProgressBar, ScrollArea, Stroke, Ui, Vec2};
use eframe::egui::collapsing_header::CollapsingState;
use eframe::egui::panel::{Side, TopBottomSide};
use eframe::egui::util::hash;
use tracing::info;

use scene::SDFViewerAppScene;

use crate::metadata::log_version_info;
use crate::sdf::demo::cube::SDFDemoCubeBrick;
use crate::sdf::SDFSurface;

pub mod cli;
pub mod scene;

/// The main application state and logic.
/// As the application is mostly single-threaded, use RefCell for performance when interior mutability is required.
pub struct SDFViewerApp {
    /// If set, indicates the load progress of the SDF in the range [0, 1] and the display text.
    pub progress: Option<(f32, String)>,
    /// The root SDF surface. This is static as it is generated with Box::leak.
    /// This is needed as we may only be rendering a sub-tree of the SDF.
    pub sdf: &'static dyn SDFSurface,
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
            sdf: Box::leak(Box::new(SDFDemoCubeBrick::default())),
        };

        // In order to configure the 3D scene after initialization, we need to create a new scene now.
        // Warning: future rendering must be done from this thread, or nothing will render.
        SDFViewerAppScene::from_glow_context_thread_local(
            &cc.gl, |_scene| {}, slf.sdf);

        slf
    }

    /// Updates the root SDF surface and sets the whole surface as the render target.
    /// The root SDF must be owned at this point.
    pub fn set_root_sdf(&mut self, sdf: impl SDFSurface + 'static) {
        // SAFETY: This is safe as self.sdf must always be a static reference created from Box::leak.
        // The Box::from_raw is only called once, and the sdf field is repopulated just after this.
        // unsafe { Box::from_raw(self.sdf as *mut _); } // TODO: Clean up previously leaked heap memory
        self.sdf = Box::leak(Box::new(sdf)); // Leak heap memory to get a 'static reference
        Self::scene_mut(|scene| scene.sdf = self.sdf);
    }

    fn ui_three_d_scene_widget(&mut self, ui: &mut Ui) {
        let (rect, response) = ui.allocate_exact_size(
            ui.available_size(), egui::Sense::click_and_drag());
        // Synchronize the scene information (from the previous frame, no way to know the future)
        self.progress = Self::scene_mut(|scene| scene.load_progress()).unwrap_or(None);
        // Queue the rendering of the scene
        ui.painter().add(egui::PaintCallback {
            rect,
            callback: Arc::new(move |info, _painter| {
                let response = response.clone();
                Self::scene_mut(|scene| {
                    scene.render(info, &response);
                });
            }),
        });
    }

    fn ui_create_hierarchy(&mut self, ui: &mut Ui, sdf: &'static dyn SDFSurface, rendering_sdf_id: usize) {
        let id = ui.make_persistent_id(format!("sdf-hierarchy-{}", sdf.id()));
        let children = sdf.children();
        let render_child = |ui: &mut Ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(sdf.name());
                ui.add_enabled_ui(sdf.id() != rendering_sdf_id, |ui| {
                    let render_button_resp = ui.button("📷");
                    let render_button_resp = render_button_resp.on_hover_text("Render only this subtree");
                    if render_button_resp.clicked() {
                        info!("Rendering only {}", sdf.name());
                        Self::scene_mut(|scene| {
                            scene.set_sdf(sdf, 128); // Will progressively regenerate the scene in the next frames
                        });
                    }
                });
                let settings_button_resp = ui.button("⚙?");
                let settings_button_resp = settings_button_resp.on_hover_text("Configure the parameters for this SDF");
                if settings_button_resp.clicked() {
                    info!("Opening parameters {}", sdf.name());
                }
            });
        };
        if children.is_empty() {
            render_child(ui);
        } else {
            CollapsingState::load_with_default_open(ui.ctx(), id, true)
                .show_header(ui, render_child)
                .body(move |ui| {
                    for child in children {
                        self.ui_create_hierarchy(ui, child, rendering_sdf_id);
                    }
                });
        }
    }

    pub fn scene_mut<R>(
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
                // Configuration panel for the parameters of the selected SDF (this must be placed first to reserve space, resizable)
                egui::TopBottomPanel::new(TopBottomSide::Bottom, hash("parameters"))
                    .resizable(true)
                    .frame(Frame::default().outer_margin(0.0).inner_margin(0.0))
                    .show_inside(ui, |ui| {
                        ui.heading("Parameters");
                        ScrollArea::both()
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                // TODO: parameters
                                for _ in 0..10 {
                                    ui.label("Lorem Ipsum");
                                }
                            });
                    });
                // The main SDF hierarchy with action buttons
                ui.heading("Hierarchy");
                ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let rendering_sdf_id = Self::scene_mut(move |scene| scene.sdf.id()).unwrap_or(0);
                        self.ui_create_hierarchy(ui, self.sdf, rendering_sdf_id);
                    });
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

