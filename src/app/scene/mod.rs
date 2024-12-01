use std::cell::RefCell;
use std::cmp::max;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use eframe::egui::Response;
use eframe::glow;
use instant::Instant;
use three_d::*;
use tracing::info;

use camera::CameraController;

use crate::app::frameinput::FrameInput;
use crate::app::scene::sdf::SDFViewer;
use crate::sdf::SDFSurface;

pub mod camera;
pub mod sdf;

thread_local! {
    /// We get a [`glow::Context`] from `eframe`, but we want a [`scene::Context`] for [`SDFViewerAppScene`].
    /// This function is a helper to convert the [`glow::Context`] to the [`scene::Context`] only once
    /// in a thread-safe way.
    ///
    /// Sadly we can't just create a [`scene::Context`] in [`MyApp::new`] and pass it
    /// to the [`egui::PaintCallback`] because [`scene::Context`] isn't `Send+Sync`, which
    /// [`egui::PaintCallback`] is.
    pub static SCENE: RefCell<Option<SDFViewerAppScene>> = const { RefCell::new(None) };
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
        gl: Arc<glow::Context>,
        f: impl FnOnce(&mut SDFViewerAppScene) -> R,
        sdf: Rc<Box<dyn SDFSurface>>,
    ) -> R {
        SCENE.with(|scene| {
            let mut scene = scene.borrow_mut();
            let scene =
                scene.get_or_insert_with(|| {
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
        SCENE.with(|scene| scene.borrow_mut().as_mut().map(f))
    }

    pub fn new(ctx: Context, sdf: Rc<Box<dyn SDFSurface>>) -> Self {
        // Create the camera
        let camera = Camera::new_perspective(
            Viewport { x: 0, y: 0, width: 0, height: 0 }, // Updated at runtime
            vec3(2.5, 3.0, 5.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            0.1,
            1000.0,
        );

        // 3D objects and lights
        let objects: Vec<Box<dyn Object>> = vec![];
        let mut lights: Vec<Box<dyn Light>> = vec![];

        // Create the SDF loader and viewer
        let sdf_viewer = SDFViewer::from_bb(&ctx, &sdf.bounding_box(), 32, 2);
        // sdf_viewer.volume.borrow_mut().material.color = Color::new_opaque(25, 225, 25);
        // objects.push(Box::new(Rc::clone(&sdf_viewer.volume)));

        lights.push(Box::new(AmbientLight::new(&ctx, 0.1, Srgba::WHITE)));

        // TODO: Custom user-defined objects (gltf) with transforms
        // TODO: Default grid object for scale
        // Load an example/test cube on the scene
        // let mut tmp_mesh = Mesh::new(&ctx, &CpuMesh::cube());
        // tmp_mesh.set_transformation(Mat4::from_translation(vec3(1.0, 0.0, 0.0)));
        // let mut tmp_material = CpuMaterial::default();
        // tmp_material.albedo = Color::new(25, 125, 225, 200);
        // let mut tmp_material = PhysicalMaterial::new_transparent(&ctx, &tmp_material);
        // tmp_material.render_states.cull = Cull::Back;
        // tmp_material.render_states.blend = Blend::TRANSPARENCY;
        // let tmp_object = Gm::new(tmp_mesh, tmp_material);
        // objects.push(Box::new(tmp_object));

        // Create more lights
        lights.push(Box::new(DirectionalLight::new(
            &ctx, 0.9, Srgba::WHITE, &vec3(-1.0, -1.0, -1.0))));

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
    pub fn set_sdf(
        &mut self,
        sdf: Rc<Box<dyn SDFSurface>>,
        max_voxels_side: Option<usize>, // None means keep the current value
        loading_passes: Option<usize>, // None means keep the current value
    ) {
        let bb = sdf.bounding_box();
        self.sdf = sdf;
        let max_voxels_side_val = max_voxels_side.unwrap_or_else(|| {
            max(
                max(self.sdf_viewer.tex0.width, self.sdf_viewer.tex0.height),
                self.sdf_viewer.tex0.depth,
            ) as usize
        });
        let loading_passes_val = loading_passes.unwrap_or(self.sdf_viewer.loading_mgr.passes);
        self.sdf_viewer = SDFViewer::from_bb(
            &self.ctx,
            &bb,
            max_voxels_side_val,
            loading_passes_val,
        );
    }

    pub fn render(&mut self, frame_input: FrameInput<'_>, egui_resp: &Response) -> Option<glow::Framebuffer> {
        // Update camera viewport and scissor box
        self.camera.update(&frame_input, egui_resp);

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
            egui_resp.ctx.request_repaint(); // Make sure we keep loading the SDF by repainting
        } else if self.sdf_viewer_last_commit.is_some() {
            let now = Instant::now();
            self.sdf_viewer.commit();
            info!("Loaded last SDF chunk in {:?} (GPU)", now - load_start_cpu);
            self.sdf_viewer_last_commit = None;
            egui_resp.ctx.request_repaint(); // Make sure we keep loading the SDF by repainting
        }

        // Get the screen render target to be able to render something on the screen
        let target = frame_input.screen;

        // Clear the depth of the "screen" render target
        target.clear_partially(frame_input.scissor_box, ClearState::depth(1.0));

        // Render each of the objects to the "screen"
        let objects_vec = self.objects.iter().map(|e| &**e).collect::<Vec<_>>();
        let objects_slice = objects_vec.as_slice();
        let lights_vec = self.lights.iter().map(|e| &**e).collect::<Vec<_>>();
        let lights_slice = lights_vec.as_slice();
        self.sdf_viewer.volume.render(&self.camera.camera, lights_slice); // FIXME: Merge this render with the next call
        target.render_partially(frame_input.scissor_box, &self.camera.camera, objects_slice, lights_slice);

        // Take back the screen fbo, we may continue to use it.
        target.into_framebuffer()
    }

    /// Reports the progress of the SDF loading
    pub fn load_progress(&self) -> Option<(f32, String)> {
        let remaining = self.sdf_viewer.loading_mgr.len();
        if self.sdf_viewer_last_commit.is_some() {
            let done_iterations = self.sdf_viewer.loading_mgr.total_iterations();
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
