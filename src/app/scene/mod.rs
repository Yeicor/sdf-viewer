use std::rc::Rc;
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
    pub sdf_viewer: SDFViewer,
    /// When we last updated the GPU texture of the SDF
    pub sdf_viewer_last_commit: Option<Instant>,
    // === CAMERA ===
    /// The 3D perspective camera
    pub camera: CameraController,
    // === ENVIRONMENT ===
    /// The full list of lights of the scene
    pub lights: Vec<Box<dyn Light>>,
    /// The full list of objects of the scene (including the SDF)
    pub objects: Vec<Box<dyn Object>>,
}

impl SDFViewerAppScene {
    pub fn new(ctx: Context) -> Self {
        // Create the camera
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

        // Create the SDF loader and viewer
        // TODO: SDF infrastructure (webserver and file drag&drop)
        let sdf = Box::new(SDFDemo {});
        let sdf_viewer = SDFViewer::from_bb(&ctx, &sdf.bounding_box(), Some(64));
        sdf_viewer.volume.borrow_mut().material.color = Color::new_opaque(25, 225, 25);

        // Load the scene
        // let tmp_mesh = Mesh::new(&ctx, &CpuMesh::cone(20)).unwrap();
        // let mut tmp_material = PbrMaterial::default();
        // tmp_material.albedo = Color::new_opaque(25, 125, 225);
        // let tmp_object = Gm::new(
        //     tmp_mesh, PhysicalMaterial::new(&ctx, &tmp_material).unwrap());

        // Create the lights
        let ambient = AmbientLight::new(&ctx, 0.4, Color::WHITE).unwrap();
        let directional1 =
            DirectionalLight::new(&ctx, 2.0, Color::WHITE, &vec3(-1.0, -1.0, -1.0)).unwrap();
        let directional2 =
            DirectionalLight::new(&ctx, 2.0, Color::WHITE, &vec3(1.0, 1.0, 1.0)).unwrap();

        let sdf_viewer_volume = Rc::clone(&sdf_viewer.volume);
        Self {
            ctx,
            camera: CameraController::new(camera),
            sdf,
            sdf_viewer,
            sdf_viewer_last_commit: None,
            lights: vec![Box::new(ambient), Box::new(directional1), Box::new(directional2)],
            objects: vec![Box::new(sdf_viewer_volume)/*, Box::new(tmp_object)*/],
        }
    }
}

impl SDFViewerAppScene {
    pub fn render(&mut self, _app: &SDFViewerApp, egui_ctx: &egui::Context, _frame: &mut Frame) {
        // Update camera
        let viewport = self.camera.update(egui_ctx);

        // Load more of the SDF to the GPU in realtime (if needed)
        let load_start_cpu = Instant::now();
        let cpu_updates = self.sdf_viewer.update(&self.sdf, Duration::from_millis(30), false);
        if cpu_updates > 0 {
            if self.sdf_viewer_last_commit.map(|i| i.elapsed().as_millis() > 1000).unwrap_or(true) {
                let load_start_gpu = Instant::now();
                self.sdf_viewer.commit();
                let now = Instant::now();
                self.sdf_viewer_last_commit = Some(now);
                info!("Loaded SDF chunk ({} updates) in {:?} (CPU) + {:?} (GPU)",
                    cpu_updates, load_start_gpu - load_start_cpu, now - load_start_gpu);
            } else {
                info!("Loaded SDF chunk ({} updates) in {:?} (CPU) + skipped (GPU)",
                    cpu_updates, Instant::now() - load_start_cpu);
            }
        } else if self.sdf_viewer_last_commit.is_some() {
            self.sdf_viewer.commit();
            self.sdf_viewer_last_commit = None;
        }

        // Prepare the screen for drawing (get the render target)
        // TODO: Sub- and super-sampling (eframe's pixels_per_point?)
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
        let lights = self.lights.iter().map(|e| &**e).collect::<Vec<_>>();
        let objects = self.objects.iter().map(|e| &**e).collect::<Vec<_>>();
        screen.render_partially(ScissorBox::from(viewport), &self.camera.camera,
                                objects.as_slice(), lights.as_slice()).unwrap();
    }
}
