use std::rc::Rc;
use std::sync::Arc;

use eframe::{egui, Theme};
use eframe::egui::{Context, Frame, ProgressBar, ScrollArea, Ui, Vec2};
use eframe::egui::collapsing_header::CollapsingState;
use eframe::egui::panel::{Side, TopBottomSide};
use image::EncodableLayout;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use cli::settings::SettingsWindow;
use scene::SDFViewerAppScene;

use crate::app::cli::CliApp;
use crate::app::frameinput::FrameInput;
use crate::cli::env_get;
use crate::sdf::demo::cube::SDFDemoCube;
use crate::sdf::SDFSurface;
use crate::sdf::wasm::load::spawn_async;

pub mod cli;
pub mod scene;
mod frameinput;

/// The main application state and logic.
/// As the application is mostly single-threaded, use RefCell for performance when interior mutability is required.
pub struct SDFViewerApp {
    /// If set, indicates the load progress of the SDF in the range [0, 1] and the display text.
    pub progress: Option<(f32, String)>,
    /// The root SDF surface. This is static as it is generated with Box::leak.
    /// This is needed as we may only be rendering a sub-tree of the SDF.
    pub sdf: Rc<Box<dyn SDFSurface>>,
    // TODO: A loading (downloading/parsing/compiling wasm) indicator for the user.
    /// The SDF for which we are modifying the parameters, if any.
    pub selected_params_sdf: Option<Rc<Box<dyn SDFSurface>>>,
    /// The application's potentially partially edited settings, displayed in a window.
    pub app_settings: SettingsWindow<CliApp>,
    /// The server's potentially partially edited settings, displayed in a window.
    #[cfg(feature = "server")]
    pub server_settings: SettingsWindow<crate::server::CliServer>,
    /// The mesher's potentially partially edited settings, displayed in a window.
    #[cfg(feature = "meshers")]
    pub mesher_settings: SettingsWindow<crate::sdf::meshers::CliMesher>,
    /// This is set when the mesher is running, and will be set to a Some("...") value when it finishes.
    /// A window with the contents should be displayed when this is set. This resets when the user closes
    /// the window.
    #[cfg(feature = "meshers")]
    pub mesher_result: Arc<Option<Mutex<Option<String>>>>,
    // ===== LOADING =====
    /// The currently loading SDF surface, that will replace the current [`sdf`] when ready.
    /// It will be polled on update.
    pub sdf_loading: Option<Receiver<Box<(dyn SDFSurface + Send + Sync)>>>,
    /// The SDF loading manager: receives values to replace sdf_loading whenever an SDF update is detected.
    pub sdf_loading_mgr: Option<Receiver<Receiver<Box<(dyn SDFSurface + Send + Sync)>>>>,
}

impl SDFViewerApp {
    #[profiling::function]
    pub fn new(cc: &eframe::CreationContext<'_>, cli_args: CliApp) -> Self {
        // Default to dark mode if no theme is provided by the OS (or environment variables)
        if (cc.integration_info.system_theme == Some(Theme::Light) ||
            env_get("light").is_some()) && env_get("dark").is_none() { // TODO: Save & restore theme settings
            cc.egui_ctx.set_visuals(egui::Visuals::light());
        } else {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
        }

        info!("Initialization complete! Starting main loop...");
        let mut slf = Self {
            progress: None,
            sdf: Rc::new(Box::<SDFDemoCube>::default()),
            sdf_loading: None,
            sdf_loading_mgr: None,
            selected_params_sdf: None,
            app_settings: SettingsWindow::Configured { settings: cli_args },
            #[cfg(feature = "server")]
            server_settings: SettingsWindow::Configured { settings: crate::server::CliServer::default() },
            #[cfg(feature = "meshers")]
            mesher_settings: SettingsWindow::Configured { settings: crate::sdf::meshers::CliMesher::default() },
            #[cfg(feature = "meshers")]
            mesher_result: Arc::new(None),
        };

        // In order to configure the 3D scene after initialization, we need to create a new scene now.
        // Warning: future rendering must be done from this thread, or nothing will render.
        SDFViewerAppScene::from_glow_context_thread_local(
            Arc::clone(cc.gl.as_ref().unwrap()), |_scene| {}, Rc::clone(&slf.sdf));

        // Now that we initialized the scene, apply all the initial CLI arguments and save the settings.
        slf.app_settings.current().apply(&mut slf);

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
    pub fn set_root_sdf_loading(&mut self, promise: Receiver<Box<(dyn SDFSurface + Send + Sync)>>) {
        self.sdf_loading = Some(promise);
    }

    /// Automatically queues updates for the root SDF using a promise that returns a promise of the updated SDF.
    /// When the promise is ready, [`set_root_sdf`](#method.set_root_sdf) will be called internally automatically.
    pub fn set_root_sdf_loading_manager(&mut self, promise: Receiver<Receiver<Box<(dyn SDFSurface + Send + Sync)>>>) {
        self.sdf_loading_mgr = Some(promise);
    }

    /// Called on every update to check if we are ready to render the SDF that was loading.
    fn update_poll_loading_sdf(&mut self, ctx: &Context) {
        // Poll the SDF loading manager promise if it is set
        self.sdf_loading_mgr = if let Some(mut receiver) = self.sdf_loading_mgr.take() {
            match receiver.try_recv() {
                Ok(new_root_sdf_loading) => {
                    self.set_root_sdf_loading(new_root_sdf_loading);
                    Some(receiver) // Keep connected for future updates
                }
                Err(TryRecvError::Empty) => {
                    ctx.request_repaint(); // Request a repaint to keep polling the promise.
                    // TODO: Less frequent repaints?
                    Some(receiver)
                }
                Err(TryRecvError::Disconnected) => {
                    warn!("sdf_loading_mgr disconnected, won't receive future updates");
                    None
                }
            }
        } else { None };
        // Poll the SDF loading promise if it is set
        self.sdf_loading = if let Some(mut receiver) = self.sdf_loading.take() {
            match receiver.try_recv() {
                Ok(new_root_sdf) => {
                    self.set_root_sdf(new_root_sdf);
                    None // Disconnect after the first update (more updates should generate another receiver to display progress)
                }
                Err(TryRecvError::Empty) => {
                    ctx.request_repaint(); // Request a repaint to keep polling the promise.
                    // TODO: Less frequent repaints?
                    Some(receiver)
                }
                Err(TryRecvError::Disconnected) => {
                    warn!("sdf_loading disconnected, the update was lost");
                    None
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
        ui.painter().add(egui::PaintCallback {
            rect,
            callback: Arc::new(eframe::egui_glow::CallbackFn::new(move |info, painter| {
                let response = response.clone();
                Self::scene_mut(|scene| {
                    let frame_input = FrameInput::new(&scene.ctx, &info, painter);
                    scene.render(frame_input, &response);
                });
            })),
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
        egui::TopBottomPanel::new(TopBottomSide::Top, egui::Id::new("top"))
            .show(ctx, |ui| {
                #[cfg(target_os = "android")]
                {
                    ui.add_space(48f32); // HACK: Add some space to avoid the status bar
                }
                ScrollArea::new([true, true]).show(ui, |ui| {
                    egui::menu::bar(ui, |ui| {
                        self.app_settings.show_window_button(ui, "⚙ Settings");
                        #[cfg(feature = "server")]
                        self.server_settings.show_window_button(ui, "🌐 Server");
                        #[cfg(feature = "meshers")]
                        self.mesher_settings.show_window_button(ui, "💾 Mesher");
                        #[cfg(all(target_arch = "wasm32", not(feature = "server")))]
                        ui.add_enabled_ui(false, |ui| ui.menu_button("🌐 Server (native-only)", |_| {}));
                        #[cfg(all(not(target_arch = "wasm32"), not(feature = "server")))]
                        ui.add_enabled_ui(false, |ui| ui.menu_button("🌐 Server (not enabled)", |_| {}));
                        #[cfg(not(feature = "meshers"))]
                        ui.add_enabled_ui(false, |ui| ui.menu_button("💾 Mesher (not enabled)", |_| {}));
                        // Add an spacer to right-align some options
                        ui.allocate_space(Vec2::new(ui.available_width() - 26.0, 1.0));
                        egui::widgets::global_dark_light_mode_switch(ui);
                    });
                });
            });
    }

    fn ui_settings_windows(&mut self, ctx: &Context) {
        if let Some(applier) = self.app_settings.show(
            ctx, "⚙ Settings", vec!["app".to_string()],
            true, false) {
            applier.apply(self);
        }
        #[cfg(feature = "server")]
        if let Some(server) = self.server_settings.show(
            ctx, "🌐 Server (execute only once!)", vec!["server".to_string()],
            false, true) {
            // TODO: stop previous server if running
            spawn_async(async move { server.run().await }, false)
        }
        #[cfg(feature = "meshers")]
        if let Some(mesher) = self.mesher_settings.show(
            ctx, "💾 Mesher", vec!["mesher".to_string()],
            false, true) {
            self.run_mesher(mesher);
        }
    }

    fn ui_left_panel(&mut self, ctx: &Context) {
        // Main side panel for configuration.
        egui::SidePanel::new(Side::Left, egui::Id::new("left"))
            .show(ctx, |ui| {
                // Configuration panel for the parameters of the selected SDF (this must be placed first to reserve space, resizable)
                // FIXME: SDF Hierarchy rendered over this panel...
                if let Some(ref selected_sdf) = self.selected_params_sdf {
                    egui::TopBottomPanel::new(TopBottomSide::Bottom, egui::Id::new("parameters"))
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
        egui::TopBottomPanel::new(TopBottomSide::Bottom, egui::Id::new("bottom"))
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

    #[cfg(feature = "meshers")]
    fn run_mesher(&mut self, mut mesher: crate::sdf::meshers::CliMesher) {
        // Overwrite the input to our input
        if let Some(inp) = self.app_settings.previous().sdf_provider.url() {
            mesher.input = inp.to_string();
        } else {
            error!("Can't find URL for SDF to render (don't use non-wasm demo source)");
            return;
        }
        // For notifying that we are running...
        self.mesher_result = Arc::new(Some(Mutex::new(None)));
        let mesher_result_ref = Arc::clone(&self.mesher_result);
        // Run in a new thread/async block to avoid blocking the UI while exporting
        spawn_async(async move {
            let mut in_memory_model = vec![];
            let output_file_clone = mesher.output_file.to_str().unwrap_or("").to_string();
            if output_file_clone.is_empty() || output_file_clone.eq("-") {
                if let Err(err) = mesher.run_custom_out(&mut in_memory_model).await {
                    let msg = format!("Failed to export model: {err}");
                    error!("{}", msg);
                    in_memory_model = msg.into_bytes();
                }
            } else {
                #[cfg(not(target_arch = "wasm32"))]
                if let Err(err) = mesher.run_cli().await {
                    let msg = format!("Failed to export model: {err}");
                    error!("{}", msg);
                    in_memory_model = msg.into_bytes();
                } else {
                    in_memory_model = "Done!".to_string().into_bytes();
                }
                #[cfg(target_arch = "wasm32")]
                if let Err(err) = mesher.run_custom_out(&mut in_memory_model).await {
                    let msg = format!("Failed to export model: {}", err);
                    error!("{}", msg);
                    in_memory_model = msg.into_bytes();
                } else {
                    // Saving to a file on wasm32 is a special case, which buffers the data and then forces a download
                    if let Err(err) = js_sys::eval(js_download_file_code(
                        &output_file_clone, in_memory_model.as_bytes()).as_str()) {
                        error!("Failed to export model using JS code: {:?}", err);
                    }
                    in_memory_model = "Done!".to_string().into_bytes();
                }
            }
            // PLY is valid ASCII text! Future model formats may not keep this property (use file outputs for them)
            let in_memory_model_text = String::from_utf8_lossy(in_memory_model.as_bytes());
            // Notify the user that the export is finished, and show a dialog with the model if requested.
            let mut guard = Option::as_ref(&mesher_result_ref).unwrap().lock().await;
            guard.replace(in_memory_model_text.to_string());
        }, false);
    }

    pub fn ui_exported_model_window(&mut self, ctx: &Context) {
        // Optimization to ignore mutex normally!
        let forget = if let Some(res) = self.mesher_result.as_ref() {
            // Progress reporting assumes no other concurrent processes are running (will override them)
            self.progress = Some((0.0, "Exporting model...".to_string()));
            if let Ok(mut guard) = res.try_lock() { // Try_lock should be safe here (called every frame)
                if let Some(model) = guard.as_mut() {
                    self.progress = Some((1.0, "Exported model!".to_string()));
                    let mut open = true;
                    egui::Window::new("Exported model")
                        .open(&mut open)
                        .resizable(true)
                        .scroll([true, true])
                        .show(ctx, |ui| {
                            ui.text_edit_multiline(model);
                        });
                    !open
                } else { false }
            } else { false }
        } else { false };
        if forget { // Just closed the window, forget model and mutex
            self.progress = None;
            self.mesher_result = Arc::new(None);
        }
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
        self.ui_settings_windows(ctx);
        self.ui_exported_model_window(ctx);
        self.ui_left_panel(ctx);
        self.ui_bottom_panel(ctx);
        self.ui_central_panel(ctx);
        // ctx.request_repaint(); // Uncomment to always render at maximum framerate instead of lazy renders
    }
}

#[cfg(target_arch = "wasm32")]
fn js_download_file_code(name: &str, contents: &[u8]) -> String {
    // TODO: Convert this to js_sys + web_sys code (more performance?)
    return format!(r#"
var bytes = new Uint8Array({:?}); // pass your byte response to this constructor
var blob=new Blob([bytes], {{type: "application/pdf"}});// change resultByte to bytes
var link=document.createElement('a');
link.href=window.URL.createObjectURL(blob);
link.download="{}";
link.click();
"#, contents, name);
}
