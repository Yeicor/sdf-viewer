use std::rc::Rc;
use std::sync::Arc;

use eframe::{egui, Storage};
use eframe::egui::panel::Side;
use eframe::egui::ScrollArea;
use eframe::egui::util::hash;
use three_d::*;
use tracing::info;

use crate::input::InputTranslator;
use crate::metadata::log_version_info;

pub mod cli;

pub struct SDFViewerApp {
    input_translator: InputTranslator,
    three_d_ctx: three_d::Context,
    camera: Camera,
    camera_ctrl: OrbitControl,
    volume: Model<IsourfaceMaterial>,
    light_ambient: AmbientLight,
    lights_dir: Vec<DirectionalLight>,
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

        // Retrieve Three-D context from the egui context (shared glow dependency).
        let three_d_ctx = Context::from_gl_context(
            unsafe { Arc::from_raw(Rc::into_raw(cc.gl.clone())) }).unwrap();

        let camera = Camera::new_perspective(
            &three_d_ctx,
            Viewport { x: 0, y: 0, width: 0, height: 0 }, // Updated at runtime
            vec3(0.25, -0.5, -2.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            0.1,
            1000.0,
        ).unwrap();
        let camera_ctrl = OrbitControl::new(*camera.target(), 1.0, 100.0);

        // Source: https://web.cs.ucdavis.edu/~okreylos/PhDStudies/Spring2000/ECS277/DataSets.html
        // TODO: SDF infrastructure (webserver and file drag&drop)
        let mut loaded = Loaded::default();
        loaded.insert_bytes("", include_bytes!("../../assets/Skull.vol").to_vec());
        let cpu_volume = loaded.vol("").unwrap();
        let mut volume = Model::new_with_material(
            &three_d_ctx,
            &CpuMesh::cube(),
            IsourfaceMaterial {
                // FIXME: Do NOT clip cube's inside triangles (or render inverted cube) to render the surface while inside
                // TODO: Variable cube size same as SDF's bounding box
                // FIXME: HACK: Use gl_FragDepth to interact with other objects of the scene
                // FIXME: Cube seams visible from far away?
                voxels: std::rc::Rc::new(Texture3D::new(&three_d_ctx, &cpu_volume.voxels).unwrap()),
                lighting_model: LightingModel::Blinn,
                size: cpu_volume.size,
                threshold: 0.15,
                color: Color::WHITE,
                roughness: 1.0,
                metallic: 0.0,
            },
        ).unwrap();
        volume.material.color = Color::new(25, 125, 25, 255);
        volume.set_transformation(Mat4::from_nonuniform_scale(
            0.5 * cpu_volume.size.x,
            0.5 * cpu_volume.size.y,
            0.5 * cpu_volume.size.z,
        ));

        let ambient = AmbientLight::new(&three_d_ctx, 0.4, Color::WHITE).unwrap();
        let directional1 =
            DirectionalLight::new(&three_d_ctx, 2.0, Color::WHITE, &vec3(-1.0, -1.0, -1.0)).unwrap();
        let directional2 =
            DirectionalLight::new(&three_d_ctx, 2.0, Color::WHITE, &vec3(1.0, 1.0, 1.0)).unwrap();

        info!("Initialization complete! Starting main loop...");
        Self {
            input_translator: InputTranslator::default(),
            three_d_ctx,
            camera,
            camera_ctrl,
            volume,
            light_ambient: ambient,
            lights_dir: vec![directional1, directional2],
        }
    }
}

impl eframe::App for SDFViewerApp {
    #[profiling::function]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // === GUI (only defined here, drawn after update() finishes) ===
        egui::SidePanel::new(Side::Left, hash(""))
            // .frame(Frame {
            //     fill: Color32::from_black_alpha(0),
            //     ..Frame::default()
            // })
            .show(ctx, |ui| {
                ScrollArea::new([false, true]).show(ui, |ui| {
                    ui.heading("Connection"); // TODO: Accordions with status emojis/counters...
                    ui.heading("Parameters");
                    ui.heading("Hierarchy");
                    ui.heading("Settings");
                    ui.horizontal(|ui| {
                        ui.label("Theme:");
                        egui::widgets::global_dark_light_mode_switch(ui);
                    });
                })
            });

        // === Draw Three-D scene ===
        let viewport_rect = ctx.available_rect();
        let viewport = Viewport {
            x: (viewport_rect.min.x * ctx.pixels_per_point()) as i32,
            y: (viewport_rect.min.y * ctx.pixels_per_point()) as i32,
            width: (viewport_rect.width() * ctx.pixels_per_point()) as u32,
            height: (viewport_rect.height() * ctx.pixels_per_point()) as u32,
        };
        self.camera.set_viewport(viewport).unwrap();
        // Handle inputs
        let mut events = self.input_translator.translate_input_events(ctx);
        // TODO: HACK: Swap left and right click for the camera controls
        self.camera_ctrl.handle_events(&mut self.camera, &mut events).unwrap();
        // Collect lights
        let mut lights = self.lights_dir.iter().map(|e| e as &dyn Light).collect::<Vec<&dyn Light>>();
        lights.push(&self.light_ambient);
        // Draw the scene to screen
        // TODO: Sub- and super-sampling (eframe's pixels_per_point?)!
        let full_screen_rect = ctx.input().screen_rect.size();
        let screen = RenderTarget::screen(
            &self.three_d_ctx, (full_screen_rect.x * ctx.pixels_per_point()) as u32,
            (full_screen_rect.y * ctx.pixels_per_point()) as u32);
        // Clear with the same background color as the UI
        let bg_color = ctx.style().visuals.window_fill();
        // FIXME: Clear not working on web platforms (egui tooltip still visible)
        screen.clear(ClearState::color_and_depth(
            bg_color.r() as f32 / 255., bg_color.g() as f32 / 255.,
            bg_color.b() as f32 / 255., 1.0, 1.0)).unwrap();
        // Now render the main scene
        screen.render_partially(ScissorBox::from(viewport), &self.camera,
                                &[&self.volume], lights.as_slice()).unwrap();
    }

    #[profiling::function]
    fn save(&mut self, _storage: &mut dyn Storage) {
        // TODO: Store app state, indexed by the loaded SDF?
        // storage.set_string()
    }
}
