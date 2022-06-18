use eframe::egui::{PaintCallbackInfo, PointerButton, Response};
use three_d::*;

/// The camera movement code.
pub struct CameraController {
    /// The camera.
    pub camera: Camera,
    /// Translate, Rotate and Scale sensitivity.
    pub sensitivity: f32,
    /// If we started rotating, keep rotating even if we press shift.
    pub is_rotating: Option<bool>,
}

impl CameraController {
    pub fn new(camera: Camera) -> Self {
        Self { camera, sensitivity: 1.0, is_rotating: None }
    }

    /// Updates the viewport according to the egui context.
    /// Also handles the events modifying the camera transform according to the events.
    pub fn update(&mut self, info: &PaintCallbackInfo, egui_resp: &Response) -> Viewport {
        // Update viewport
        let viewport = info.viewport_in_pixels();
        let viewport = Viewport {
            x: viewport.left_px.round() as _,
            y: viewport.from_bottom_px.round() as _,
            width: viewport.width_px.round() as _,
            height: viewport.height_px.round() as _,
        };
        self.camera.set_viewport(viewport).unwrap();
        // Handle inputs
        if egui_resp.hovered() { // If interacting with the widget
            let state = egui_resp.ctx.input();
            let dragged_delta = egui_resp.drag_delta();
            let number_touches = state.multi_touch().map(|touches| touches.num_touches).unwrap_or(0);
            let scroll_y = state.multi_touch().and_then(|touches| {
                if number_touches == 2 {
                    const TOUCH_SENSITIVITY: f32 = 0.01; // Otherwise, it is always zooming
                    let zoom_delta = touches.zoom_delta - 1.;
                    if zoom_delta.abs() > TOUCH_SENSITIVITY {
                        Some(zoom_delta) // positive is zoom in, negative is zoom out
                    } else { None }
                } else { None }
            }).unwrap_or(state.scroll_delta.y);
            if egui_resp.dragged_by(PointerButton::Secondary) || number_touches >= 2 && scroll_y == 0. {
                let should_rotate = self.is_rotating.unwrap_or(!state.modifiers.shift && number_touches < 3);
                if should_rotate {
                    self.is_rotating = Some(true);
                    // Rotate the camera in an orbit around the target
                    let target = *self.camera.target();
                    let zoom_dist = self.camera.position().distance(target);
                    let delta = dragged_delta * self.sensitivity * 0.05 * zoom_dist;
                    self.camera.rotate_around(&target, delta.x as f32, delta.y as f32).unwrap();
                } else {
                    self.is_rotating = Some(false);
                    // Move the camera target
                    let target = *self.camera.target();
                    let zoom_dist = self.camera.position().distance(target);
                    let delta = dragged_delta * self.sensitivity * 0.01 * zoom_dist;
                    let right_direction = self.camera.right_direction();
                    let up_direction = right_direction.cross(self.camera.view_direction());
                    let delta_camera_space = right_direction * -delta.x + up_direction * delta.y;
                    self.camera.translate(&delta_camera_space).unwrap();
                }
            } else {
                self.is_rotating = None;
            }
            if scroll_y != 0. {
                // Zoom the camera
                let target = *self.camera.target();
                let pos = self.camera.position();
                let distance = pos.distance(target);
                let delta = scroll_y.signum() * self.sensitivity * 0.1 * distance;
                let new_distance = (distance - delta).max(0.01).min(1000.0);
                let new_position = target - self.camera.view_direction() * new_distance;
                let up = *self.camera.up();
                self.camera.set_view(new_position, target, up).unwrap();
            }
        }
        // self.camera_ctrl.handle_events(&mut self.camera, &mut events).unwrap();
        viewport
    }
}