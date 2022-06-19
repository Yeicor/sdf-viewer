use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use eframe::egui::{Color32, PaintCallbackInfo, Response};
use eframe::glow;
use instant::Instant;
use three_d::*;
use tracing::info;

use camera::CameraController;

use crate::app::scene::sdf::SDFViewer;
use crate::sdf::SDFSurface;

pub mod sdf;
pub mod camera;

thread_local! {
    /// We get a [`glow::Context`] from `eframe`, but we want a [`scene::Context`] for [`SDFViewerAppScene`].
    /// This function is a helper to convert the [`glow::Context`] to the [`scene::Context`] only once
    /// in a thread-safe way.
    ///
    /// Sadly we can't just create a [`scene::Context`] in [`MyApp::new`] and pass it
    /// to the [`egui::PaintCallback`] because [`scene::Context`] isn't `Send+Sync`, which
    /// [`egui::PaintCallback`] is.
    pub static SCENE: RefCell<Option<SDFViewerAppScene>> = RefCell::new(None);
}

/// Renders the main 3D scene, containing the SDF object
pub struct SDFViewerAppScene {
    // === CONTEXT ===
    /// The 3D rendering context of the library we use to render the scene
    pub ctx: Context,
    // === SDF ===
    /// The CPU-side definition of the SDF object to render (infinite precision)
    pub sdf: &'static dyn SDFSurface,
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
    /// This initializes (the first time) a new [`SDFViewerAppScene`] from the given [`glow::Context`],
    /// and runs the given function with a mutable reference to the scene.
    pub fn from_glow_context_thread_local<R>(
        gl: &Rc<glow::Context>,
        f: impl FnOnce(&mut SDFViewerAppScene) -> R,
        sdf: &'static dyn SDFSurface,
    ) -> R {
        SCENE.with(|scene| {
            let mut scene = scene.borrow_mut();
            let scene =
                scene.get_or_insert_with(|| {
                    // HACK: need to convert the GL context from Rc to Arc (UNSAFE: likely double-free on app close)
                    let gl = unsafe { std::mem::transmute(gl.clone()) }; // FIXME: this unsafe block
                    // Retrieve Three-D context from the egui context (thanks to the shared glow dependency).
                    let three_d_ctx = Context::from_gl_context(gl).unwrap();
                    // Create the Three-D scene (only the first time).
                    SDFViewerAppScene::new(three_d_ctx, sdf)
                });
            f(scene)
        })
    }

    /// Runs the given function with a mutable reference to the scene, ONLY if it was previously initialized.
    pub fn read_context_thread_local<R>(
        f: impl FnOnce(&mut Self) -> R,
    ) -> Option<R> {
        SCENE.with(|scene| scene.borrow_mut().as_mut().map(|x| {
            f(x)
        }))
    }

    pub fn new(ctx: Context, sdf: &'static dyn SDFSurface) -> Self {
        // Create the camera
        let camera = Camera::new_perspective(
            &ctx,
            Viewport { x: 0, y: 0, width: 0, height: 0 }, // Updated at runtime
            vec3(2.5, 3.0, 5.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            0.1,
            1000.0,
        ).unwrap();

        // 3D objects and lights
        let mut objects: Vec<Box<dyn Object>> = vec![];
        let mut lights: Vec<Box<dyn Light>> = vec![];

        // Create the SDF loader and viewer
        // TODO: SDF infrastructure (webserver and file drag&drop)
        // let sdf = Box::new(SDFDemoCubeBrick::default());
        let sdf_viewer = SDFViewer::from_bb(&ctx, &sdf.bounding_box(), Some(128));
        // sdf_viewer.volume.borrow_mut().material.color = Color::new_opaque(25, 225, 25);

        // Load the skybox (embedded in the binary)
        if cfg!(feature = "skybox") { // TODO: Speed-up skybox load times
            let mut skybox_image = image::load_from_memory_with_format(
                include_bytes!("../../../assets/skybox.jpg"),
                image::ImageFormat::Jpeg,
            ).unwrap();
            skybox_image = skybox_image.adjust_contrast(-15.0); // %
            skybox_image = skybox_image.brighten(-50); // u8
            let skybox_texture = CpuTexture {
                data: TextureData::RgbU8(skybox_image.as_rgb8().unwrap().as_raw()
                    .chunks_exact(3).map(|e| [e[0], e[1], e[2]]).collect::<_>()),
                width: skybox_image.width(),
                height: skybox_image.height(),
                ..CpuTexture::default()
            };
            let skybox = Skybox::new_from_equirectangular(
                &ctx, &skybox_texture).unwrap();
            let ambient_light = AmbientLight::new_with_environment(
                &ctx, 1.0, Color::WHITE, skybox.texture()).unwrap();
            objects.push(Box::new(skybox));
            lights.push(Box::new(ambient_light));
        } else {
            lights.push(Box::new(AmbientLight::new(&ctx, 0.25, Color::WHITE).unwrap()));
        }

        // Load the scene TODO: custom user-defined objects (gltf) with transforms
        // let tmp_mesh = Mesh::new(&ctx, &CpuMesh::cone(20)).unwrap();
        // let mut tmp_material = PbrMaterial::default();
        // tmp_material.albedo = Color::new_opaque(25, 125, 225);
        // let tmp_object = Gm::new(
        //     tmp_mesh, PhysicalMaterial::new(&ctx, &tmp_material).unwrap());

        // Create more lights
        lights.push(Box::new(DirectionalLight::new(
            &ctx, 2.0, Color::WHITE, &vec3(-1.0, -1.0, -1.0)).unwrap()));
        lights.push(Box::new(DirectionalLight::new(
            &ctx, 0.5, Color::WHITE, &vec3(1.0, 1.0, 1.0)).unwrap()));

        Self {
            ctx,
            camera: CameraController::new(camera),
            sdf,
            sdf_viewer,
            sdf_viewer_last_commit: None,
            lights,
            objects,
        }
    }

    /// Updates the SDF to render (and clears all required caches).
    pub fn set_sdf(&mut self, sdf: &'static dyn SDFSurface, max_voxels_side: usize) {
        self.sdf = sdf;
        self.sdf_viewer = SDFViewer::from_bb(&self.ctx, &sdf.bounding_box(), Some(max_voxels_side));
    }

    pub fn render(&mut self, info: &PaintCallbackInfo, egui_resp: &Response) {
        // Update camera
        let viewport = self.camera.update(info, egui_resp);

        // Load more of the SDF to the GPU in realtime (if needed)
        let load_start_cpu = Instant::now();
        let cpu_updates = self.sdf_viewer.update(self.sdf, Duration::from_millis(30));
        if cpu_updates > 0 {
            // Update the GPU texture sparingly (to mitigate stuttering on high-detail rendering loads)
            if self.sdf_viewer_last_commit.map(|i| i.elapsed().as_millis() > 500).unwrap_or(true) {
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
        let full_screen_rect = egui_resp.ctx.input().screen_rect.size();
        let screen = RenderTarget::screen(
            &self.ctx, (full_screen_rect.x * egui_resp.ctx.pixels_per_point()) as u32,
            (full_screen_rect.y * egui_resp.ctx.pixels_per_point()) as u32);

        // FIXME: Clear not working on web platforms (egui tooltip still visible). Related: https://github.com/emilk/egui/issues/1744
        let bg_color = if egui_resp.ctx.style().visuals.dark_mode { Color32::BLACK } else { Color32::WHITE };
        let scissor_box = ScissorBox::from(viewport);
        screen.clear_partially(scissor_box, ClearState::color_and_depth(
            bg_color.r() as f32 / 255., bg_color.g() as f32 / 255.,
            bg_color.b() as f32 / 255., 1.0, 1.0)).unwrap();

        // Now render the main scene
        // Note: there is no need to clear the scene (already done by egui with the correct color)
        let lights = self.lights.iter().map(|e| &**e).collect::<Vec<_>>();
        let mut objects = self.objects.iter().map(|e| &**e).collect::<Vec<_>>();
        objects.push(&self.sdf_viewer.volume); // "Add" the volume always to update automatically
        screen.render_partially(scissor_box, &self.camera.camera,
                                objects.as_slice(), lights.as_slice()).unwrap();
    }

    /// Reports the progress of the SDF loading
    pub fn load_progress(&self) -> Option<(f32, String)> {
        let remaining = self.sdf_viewer.loading_mgr.len();
        if self.sdf_viewer_last_commit.is_some() {
            if remaining > 0 {
                let progress = self.sdf_viewer.loading_mgr.iterations() as f32 /
                    ((self.sdf_viewer.loading_mgr.iterations() + remaining) as f32);
                Some((progress, format!("Loading SDF... {} passes left",
                                        self.sdf_viewer.loading_mgr.passes_left())))
            } else {
                Some((1.0, "Loading SDF done! (ignore this message, it is a bug if you see it: \
                try resizing the screen)".to_string()))
            }
        } else {
            None
        }
    }
}
