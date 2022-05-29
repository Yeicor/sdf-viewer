use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use eframe::{egui, Storage};
use three_d::Context;
use tracing::info;

use scene::SDFViewerAppScene;
use ui::SDFViewerAppUI;


use crate::metadata::log_version_info;

pub mod cli;
pub mod ui;
pub mod scene;

/// The main application state and logic.
/// As the application is mostly single-threaded, use RefCell for performance when interior mutability is required.
pub struct SDFViewerApp {
    /// The UI renderer
    pub ui: RefCell<SDFViewerAppUI>,
    /// The 3D scene renderer
    pub scene: RefCell<SDFViewerAppScene>,
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

        // HACK: need to convert the GL context from Rc to Arc (UNSAFE: likely double-free on app close)
        let gl_context = unsafe { Arc::from_raw(Rc::into_raw(cc.gl.clone())) };
        // Retrieve Three-D context from the egui context (thanks to the shared glow dependency).
        let three_d_ctx = Context::from_gl_context(gl_context).unwrap();

        info!("Initialization complete! Starting main loop...");
        Self {
            ui: RefCell::new(SDFViewerAppUI::default()),
            scene: RefCell::new(SDFViewerAppScene::new(three_d_ctx)),
        }
    }
}

impl eframe::App for SDFViewerApp {
    #[profiling::function]
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // borrow_mut is safe because we only ever access the ui and scene from this main thread
        self.ui.borrow_mut().render(self, ctx, frame);
        self.scene.borrow_mut().render(self, ctx, frame);
    }

    #[profiling::function]
    fn save(&mut self, _storage: &mut dyn Storage) {
        // TODO: Store app state, indexed by the loaded SDF?
        // storage.set_string()
    }
}
