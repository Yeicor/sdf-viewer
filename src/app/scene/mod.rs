use std::time::Duration;

use eframe::{egui, Frame};
use instant::Instant;
use three_d::*;
use tracing::info;

use camera::CameraController;

use crate::app::scene::sdf::SDFViewer;
use crate::app::SDFViewerApp;
use crate::sdf::demo::SDFDemo;
use crate::sdf::SDFSurface;

pub mod sdf;
pub mod camera;
// TODO: Custom skybox/background/loading-external-gltf-to-compare module

/// Renders the main 3D scene, containing the SDF object
pub struct SDFViewerAppScene {
    // === CONTEXT ===
    /// The 3D rendering context of the library we use to render the scene
    pub ctx: Context,
    // === SDF ===
    /// The CPU-side definition of the SDF object to render (infinite precision)
    pub sdf: Box<dyn SDFSurface>,
    /// The controller that helps manage and synchronize the GPU material with the CPU SDF.
    pub surface: SDFViewer,
    // === CAMERA ===
    /// The 3D perspective camera
    pub camera: CameraController,
    // === ENVIRONMENT ===
    /// The ambient light of the scene (hits everything, in all directions)
    pub light_ambient: AmbientLight,
    /// The directional lights of the scene
    pub lights_dir: Vec<DirectionalLight>,
}

impl SDFViewerAppScene {
    pub fn new(ctx: Context) -> Self {
        let camera = Camera::new_perspective(
            &ctx,
            Viewport { x: 0, y: 0, width: 0, height: 0 }, // Updated at runtime
            vec3(1.0, 3.0, -5.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            0.1,
            1000.0,
        ).unwrap();

        // Source: https://web.cs.ucdavis.edu/~okreylos/PhDStudies/Spring2000/ECS277/DataSets.html
        // TODO: SDF infrastructure (webserver and file drag&drop)
        let sdf = Box::new(SDFDemo {});
        let mut sdf_renderer = SDFViewer::from_bb(&ctx, &sdf.bounding_box(), Some(25));
        let load_start_cpu = Instant::now();
        sdf_renderer.update(&sdf, Duration::from_secs(3600), false);
        let load_start_gpu = Instant::now();
        sdf_renderer.commit();
        info!("Loaded SDF in {:?} (CPU) + {:?} (GPU)", load_start_cpu.elapsed(), load_start_gpu.elapsed());
        sdf_renderer.volume.material.color = Color::GREEN;
        // sdf_renderer.volume.set_transformation(Mat4::from_translation(Vector3::new(-0.5, -0.5, -0.5)));
        // TODO: Scale transform!
        // TODO(optional): Test rotation transform

        let ambient = AmbientLight::new(&ctx, 0.4, Color::WHITE).unwrap();
        let directional1 =
            DirectionalLight::new(&ctx, 2.0, Color::WHITE, &vec3(-1.0, -1.0, -1.0)).unwrap();
        let directional2 =
            DirectionalLight::new(&ctx, 2.0, Color::WHITE, &vec3(1.0, 1.0, 1.0)).unwrap();

        Self {
            ctx,
            camera: CameraController::new(camera),
            sdf,
            surface: sdf_renderer,
            light_ambient: ambient,
            lights_dir: vec![directional1, directional2],
        }
    }
}

impl SDFViewerAppScene {
    pub fn render(&mut self, _app: &SDFViewerApp, egui_ctx: &egui::Context, _frame: &mut Frame) {
        // === Draw Three-D scene ===
        let viewport = self.camera.update(egui_ctx);
        // Collect lights
        let mut lights = self.lights_dir.iter().map(|e| e as &dyn Light).collect::<Vec<&dyn Light>>();
        lights.push(&self.light_ambient);
        // Draw the scene to screen
        // TODO: Sub- and super-sampling (eframe's pixels_per_point?)!
        let full_screen_rect = egui_ctx.input().screen_rect.size();
        let screen = RenderTarget::screen(
            &self.ctx, (full_screen_rect.x * egui_ctx.pixels_per_point()) as u32,
            (full_screen_rect.y * egui_ctx.pixels_per_point()) as u32);
        // Clear with the same background color as the UI
        let bg_color = egui_ctx.style().visuals.window_fill();
        // FIXME: Clear not working on web platforms (egui tooltip still visible)
        screen.clear(ClearState::color_and_depth(
            bg_color.r() as f32 / 255., bg_color.g() as f32 / 255.,
            bg_color.b() as f32 / 255., 1.0, 1.0)).unwrap();
        // Now render the main scene
        screen.render_partially(ScissorBox::from(viewport), &self.camera.camera,
                                &[&self.surface.volume], lights.as_slice()).unwrap();
    }
}
