use eframe::egui;
use three_d::*;

/// The camera movement code.
pub struct CameraController {
    /// The camera.
    pub camera: Camera,
}

impl CameraController {
    pub fn new(camera: Camera) -> Self {
        Self { camera }
    }

    /// Updates the viewport according to the egui context.
    /// Also handles the events modifying the camera transform according to the events.
    pub fn update(&mut self, ctx: &egui::Context) -> Viewport {
        // Update viewport
        let viewport_rect = ctx.available_rect();
        let viewport = Viewport {
            x: (viewport_rect.min.x * ctx.pixels_per_point()) as i32,
            y: (viewport_rect.min.y * ctx.pixels_per_point()) as i32,
            width: (viewport_rect.width() * ctx.pixels_per_point()) as u32,
            height: (viewport_rect.height() * ctx.pixels_per_point()) as u32,
        };
        self.camera.set_viewport(viewport).unwrap();
        // Handle inputs
        let _state = ctx.input();
        // TODO: HACK: Swap left and right click for the camera controls
        // TODO: Allow camera movements
        // self.camera_ctrl.handle_events(&mut self.camera, &mut events).unwrap();
        viewport
    }
}