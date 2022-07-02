use std::rc::Rc;
use std::sync::Arc;

use eframe::egui;
use eframe::egui::{Context, Frame, ProgressBar, ScrollArea, Ui, Vec2};
use eframe::egui::collapsing_header::CollapsingState;
use eframe::egui::panel::{Side, TopBottomSide};
use eframe::egui::util::hash;
use klask::LocalizationSettings;
use once_cell::sync::OnceCell;
use tracing::{info, warn};

use cli::SDFViewerAppSettings;
use scene::SDFViewerAppScene;

use crate::app::cli::CliApp;
use crate::cli::env_get;
use crate::sdf::demo::cube::SDFDemoCube;
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
    pub sdf: Rc<Box<dyn SDFSurface>>,
    /// The currently loading SDF surface, that will replace the current [`sdf`] when ready.
    /// It will be polled on update.
    pub sdf_loading: Option<poll_promise::Promise<Box<(dyn SDFSurface + Send + Sync)>>>,
    // TODO: A loading (downloading/parsing/compiling wasm) indicator for the user.
    /// The SDF for which we are modifying the parameters, if any.
    pub selected_params_sdf: Option<Rc<Box<dyn SDFSurface>>>,
    /// The application's potentially partially edited settings, displayed in a window.
    pub settings_values: SDFViewerAppSettings,
}

impl SDFViewerApp {
    #[profiling::function]
    pub fn new(cc: &eframe::CreationContext<'_>, cli_args: CliApp) -> Self {
        // Default to dark mode if no theme is provided by the OS (or environment variables)
        if (cc.integration_info.prefer_dark_mode == Some(false) ||
            env_get("light").is_some()) && env_get("dark").is_none() { // TODO: Save & restore theme settings
            cc.egui_ctx.set_visuals(egui::Visuals::light());
        } else {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
        }

        info!("Initialization complete! Starting main loop...");
        let mut slf = Self {
            progress: None,
            sdf: Rc::new(Box::new(SDFDemoCube::default())),
            sdf_loading: None,
            selected_params_sdf: None,
            settings_values: SDFViewerAppSettings::Configured { settings: cli_args },
        };

        // In order to configure the 3D scene after initialization, we need to create a new scene now.
        // Warning: future rendering must be done from this thread, or nothing will render.
        SDFViewerAppScene::from_glow_context_thread_local(
            &cc.gl, |_scene| {}, Rc::clone(&slf.sdf));

        // Now that we initialized the scene, apply all the initial CLI arguments and save the settings.
        slf.settings_values.current().apply(&mut slf);

        slf
    }

    /// Updates the root SDF surface and sets the whole surface as the render target.
    /// The root SDF must be owned at this point.
    pub fn set_root_sdf(&mut self, sdf: Box<dyn SDFSurface>) {
        self.sdf = Rc::new(sdf); // Reference counted ownership as we need to share it with the scene renderer.
        Self::scene_mut(|scene| scene.set_sdf(Rc::clone(&self.sdf), 64, 2));
    }

    /// Updates the root SDF using a promise that will be polled on update.
    /// When the promise is ready, [`set_root_sdf`](#method.set_root_sdf) will be called internally automatically.
    pub fn set_root_sdf_loading(&mut self, promise: poll_promise::Promise<Box<(dyn SDFSurface + Send + Sync)>>) {
        self.sdf_loading = Some(promise);
    }

    /// Called on every update to check if we are ready to render the SDF that was loading.
    fn update_poll_loading_sdf(&mut self, ctx: &Context) {
        // Poll the SDF loading promise if it is set
        self.sdf_loading = if let Some(promise) = self.sdf_loading.take() {
            match promise.try_take() {
                Ok(new_root_sdf) => {
                    self.set_root_sdf(new_root_sdf);
                    None
                }
                Err(promise_again) => {
                    ctx.request_repaint(); // Request a repaint to keep polling the promise.
                    Some(promise_again)
                }
            }
        } else { None };
    }

    pub fn scene_mut<R>(
        f: impl FnOnce(&mut SDFViewerAppScene) -> R,
    ) -> Option<R> {
        SDFViewerAppScene::read_context_thread_local(f)
    }

    fn ui_three_d_scene_widget(&mut self, ui: &mut Ui) {
        let (rect, response) = ui.allocate_exact_size(
            ui.available_size(), egui::Sense::click_and_drag());
        // Synchronize the scene information (from the previous frame, no way to know the future)
        self.progress = Self::scene_mut(|scene| scene.load_progress()).unwrap_or(None);
        // Queue the rendering of the scene
        let ui_ctx = ui.ctx().clone();
        ui.painter().add(egui::PaintCallback {
            rect,
            callback: Arc::new(move |info, _painter| {
                // OpenGL API at _painter.downcast_mut::<egui_glow::Painter>().unwrap().gl()
                let response = response.clone();
                Self::scene_mut(|scene| {
                    scene.render(&ui_ctx, info, &response);
                });
            }),
        });
    }

    fn ui_create_hierarchy(&mut self, ui: &mut Ui, sdf: Rc<Box<dyn SDFSurface>>, rendering_sdf_id: u32) {
        let id = ui.make_persistent_id(format!("sdf-hierarchy-{}", sdf.id()));
        let mut render_child = |ui: &mut Ui| {
            ui.horizontal_wrapped(|ui| {
                // Pre-compute (own) some values for the rendering of the SDF (to then move the SDF)
                ui.label(sdf.name());
                let rendering_this_sdf = sdf.id() == rendering_sdf_id;
                ui.add_enabled_ui(!rendering_this_sdf, |ui| {
                    let render_button_resp = ui.button("📷");
                    let render_button_resp = render_button_resp.on_hover_text("Render only this subtree");
                    if render_button_resp.clicked() {
                        info!("Rendering only {}", sdf.name());
                        Self::scene_mut(|scene| {
                            // Will progressively regenerate the scene in the next frames
                            scene.set_sdf(Rc::clone(&sdf), 64, 2);
                        });
                    }
                });
                let params = sdf.parameters();
                if !params.is_empty() {
                    let editing_params = self.selected_params_sdf.as_ref()
                        .map(|sdf2| sdf2.id() == sdf.id()).unwrap_or(false);
                    let mut editing_params_now = editing_params;
                    let settings_button_resp = ui.toggle_value(&mut editing_params_now, format!("⚙ {}", params.len()));
                    if editing_params_now {
                        settings_button_resp.on_hover_text("Stop editing parameters".to_string());
                        self.selected_params_sdf = Some(Rc::clone(&sdf));
                    } else {
                        settings_button_resp.on_hover_text(format!("Edit {} parameters", params.len()));
                        if editing_params {
                            self.selected_params_sdf = None
                        }
                    }
                }
            });
        };
        if sdf.children().is_empty() {
            render_child(ui);
        } else {
            CollapsingState::load_with_default_open(ui.ctx(), id, true)
                .show_header(ui, render_child)
                .body(move |ui| {
                    for child in sdf.children() {
                        self.ui_create_hierarchy(ui, Rc::new(child), rendering_sdf_id);
                    }
                });
        }
    }

    fn ui_menu_bar(&mut self, ctx: &Context) {
        // Top panel for the menu bar
        egui::TopBottomPanel::new(TopBottomSide::Top, hash("top"))
            .show(ctx, |ui| {
                ScrollArea::new([true, true]).show(ui, |ui| {
                    egui::menu::bar(ui, |ui| {
                        egui::menu::menu_button(ui, "📄 Open SDF", |_ui| {
                            // TODO: Open and swap the new SDF manually inserted (url/file)
                        });
                        // If settings are not already open, enable the settings button.
                        let (enabled, values) = match &self.settings_values {
                            SDFViewerAppSettings::Configured { settings: values } => (true, values.clone()),
                            SDFViewerAppSettings::Configuring { previous, .. } => (false, previous.clone()),
                        };
                        ui.add_enabled_ui(enabled, |ui| {
                            let settings_button = egui::menu::menu_button(ui, "⚙ Settings", |_ui| {});
                            if settings_button.response.clicked() {
                                use clap::CommandFactory;
                                static LOC_SETTINGS: OnceCell<LocalizationSettings> = OnceCell::new();
                                self.settings_values = SDFViewerAppSettings::Configuring {
                                    previous: values.clone(),
                                    editing: klask::app_state::AppState::new(
                                        &CliApp::command(),
                                        LOC_SETTINGS.get_or_init(|| LocalizationSettings::default())),
                                };
                            }
                        });
                        // Add an spacer to right-align some options
                        ui.allocate_space(Vec2::new(ui.available_width() - 26.0, 1.0));
                        egui::widgets::global_dark_light_mode_switch(ui);
                    });
                });
            });
    }

    fn ui_left_panel(&mut self, ctx: &Context) {
        // Main side panel for configuration.
        egui::SidePanel::new(Side::Left, hash("left"))
            .show(ctx, |ui| {
                // Configuration panel for the parameters of the selected SDF (this must be placed first to reserve space, resizable)
                // FIXME: SDF Hierarchy rendered over this panel...
                if let Some(ref selected_sdf) = self.selected_params_sdf {
                    egui::TopBottomPanel::new(TopBottomSide::Bottom, hash("parameters"))
                        .resizable(true)
                        .default_height(200.0)
                        .frame(Frame::default().outer_margin(0.0).inner_margin(0.0))
                        .show_inside(ui, |ui| {
                            ui.heading(format!("Parameters for {}", selected_sdf.name()));
                            ScrollArea::both()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    for mut param in selected_sdf.parameters() {
                                        if param.gui(ui) { // If the value was modified
                                            match selected_sdf.set_parameter(param.id, &param.value) {
                                                Ok(()) => {} // Implementation should report the change in the next sdf.changed() call
                                                Err(e) => warn!("Failed to set parameter: {}", e), // TODO: User-facing error handling
                                            }
                                        }
                                    }
                                });
                        });
                }
                // The main SDF hierarchy with action buttons
                ui.horizontal_wrapped(|ui| {
                    ui.heading("Hierarchy");
                    if self.sdf_loading.is_some() {
                        ui.spinner().on_hover_text(
                            "Currently downloading/compiling a new version of the SDF code");
                    }
                });
                ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let rendering_sdf_id = Self::scene_mut(move |scene| scene.sdf.id()).unwrap_or(0);
                        self.ui_create_hierarchy(ui, Rc::clone(&self.sdf), rendering_sdf_id);
                    });
            });
    }

    fn ui_bottom_panel(&mut self, ctx: &Context) {
        // Bottom panel, containing the progress bar if applicable.
        egui::TopBottomPanel::new(TopBottomSide::Bottom, hash("bottom"))
            .frame(Frame::default().inner_margin(0.0))
            .min_height(0.0) // Hide when unused
            .show(ctx, |ui| {
                if let Some((progress, text)) = self.progress.as_ref() {
                    ui.add(ProgressBar::new(*progress).text(text.clone()).animate(true));
                }
            });
    }

    fn ui_central_panel(&mut self, ctx: &Context) {
        // 3D Scene main content
        egui::CentralPanel::default()
            .frame(Frame::none().inner_margin(0.0))
            .show(ctx, |ui| {
                Frame::canvas(ui.style())
                    .show(ui, |ui| {
                        self.ui_three_d_scene_widget(ui);
                    });
            });
    }
}

impl eframe::App for &mut SDFViewerApp {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        SDFViewerApp::update(self, ctx, frame)
    }
}

impl eframe::App for SDFViewerApp {
    #[profiling::function]
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.update_poll_loading_sdf(ctx);
        self.ui_menu_bar(ctx);
        SDFViewerAppSettings::show(self, ctx);
        self.ui_left_panel(ctx);
        self.ui_bottom_panel(ctx);
        self.ui_central_panel(ctx);
        // ctx.request_repaint(); // Uncomment to always render at maximum framerate instead of lazy renders
    }
}