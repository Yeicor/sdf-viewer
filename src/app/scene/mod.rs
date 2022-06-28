use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use eframe::egui::{PaintCallbackInfo, Response};
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
    pub sdf: Rc<Box<dyn SDFSurface>>,
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
        sdf: Rc<Box<dyn SDFSurface>>,
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

    pub fn new(ctx: Context, sdf: Rc<Box<dyn SDFSurface>>) -> Self {
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
        let sdf_viewer = SDFViewer::from_bb(&ctx, &sdf.bounding_box(), Some(128), 3);
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
    pub fn set_sdf(&mut self, sdf: Rc<Box<dyn SDFSurface>>, max_voxels_side: usize, loading_passes: usize) {
        let bb = sdf.bounding_box();
        self.sdf = sdf;
        self.sdf_viewer = SDFViewer::from_bb(&self.ctx, &bb, Some(max_voxels_side), loading_passes);
    }

    pub fn render(&mut self, info: &PaintCallbackInfo, egui_resp: &Response) {
        // Update camera viewport and scissor box
        let viewport = self.camera.update(info, egui_resp);
        let scissor_box = ScissorBox::from(viewport);

        // Load more of the SDF to the GPU in real time (if needed)
        let load_start_cpu = Instant::now();
        let cpu_updates = self.sdf_viewer.update(&self.sdf, Duration::from_millis(30));
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

        // Instead of the normal path of RenderTarget::render(), we use the direct rendering methods
        // as egui uses a custom RenderTarget?
        // Note: there is no need to clear the scene (already done by egui with the correct color).
        // However, we need to clear the depth buffer as egui does not do this by default.
        unsafe { self.ctx.clear(context::DEPTH_BUFFER_BIT); } // OpenGL calls are always unsafe
        self.ctx.set_scissor(scissor_box);
        let lights = self.lights.iter().map(|e| &**e).collect::<Vec<_>>();
        for obj in &self.objects {
            obj.render(&self.camera.camera, lights.as_slice()).unwrap();
        }
        self.sdf_viewer.volume.render(&self.camera.camera, lights.as_slice()).unwrap();
    }

    /// Reports the progress of the SDF loading
    pub fn load_progress(&self) -> Option<(f32, String)> {
        let remaining = self.sdf_viewer.loading_mgr.len();
        if self.sdf_viewer_last_commit.is_some() {
            let done_iterations = self.sdf_viewer.loading_mgr.iterations();
            let total_iterations = done_iterations + remaining;
            let progress = done_iterations as f32 / ((total_iterations) as f32);
            Some((progress, format!("Loading SDF {:.2}% ({} levels of detail left, evaluations: {} / {})",
                                    progress * 100.0, self.sdf_viewer.loading_mgr.passes_left(),
                                    done_iterations, total_iterations)))
        } else {
            None
        }
    }
}
