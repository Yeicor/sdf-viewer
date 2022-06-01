use eframe::egui;

use three_d::*;

use crate::input::is_event_handled_by_egui;

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
        // TODO: Mobile controls!
        let pointer_pos = { ctx.input() }.pointer.interact_pos();
        let handled_by_egui = is_event_handled_by_egui(ctx, pointer_pos);
        if !handled_by_egui {
            let state = ctx.input();
            if state.pointer.secondary_down() {
                let should_rotate = self.is_rotating.unwrap_or(!state.modifiers.shift);
                if should_rotate {
                    self.is_rotating = Some(true);
                    // Rotate the camera in an orbit around the target
                    let target = *self.camera.target();
                    let zoom_dist = self.camera.position().distance(target);
                    let delta = state.pointer.delta() * self.sensitivity * 0.05 * zoom_dist;
                    self.camera.rotate_around(&target, delta.x as f32, delta.y as f32).unwrap();
                } else {
                    self.is_rotating = Some(false);
                    // Move the camera target
                    let target = *self.camera.target();
                    let zoom_dist = self.camera.position().distance(target);
                    let delta = state.pointer.delta() * self.sensitivity * 0.01 * zoom_dist;
                    let right_direction = self.camera.right_direction();
                    let up_direction = right_direction.cross(self.camera.view_direction());
                    let delta_camera_space = right_direction * -delta.x + up_direction * delta.y;
                    self.camera.translate(&delta_camera_space).unwrap();
                }
            } else {
                self.is_rotating = None;
            }
            if state.scroll_delta.y != 0. {
                // Zoom the camera
                let target = *self.camera.target();
                let pos = self.camera.position();
                let distance = pos.distance(target);
                let delta = state.scroll_delta.y * self.sensitivity * 0.005 * distance;
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