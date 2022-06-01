use eframe::egui;
use eframe::egui::{Pos2};



pub fn is_event_handled_by_egui(ctx: &egui::Context, pos: Option<Pos2>) -> bool {
    let scene_rect = ctx.available_rect();
    let currently_inside_egui_area = pos.map(|pos| !scene_rect.contains(pos)).unwrap_or(false);
    // Also handle when dragging starts outside the render area
    let state = ctx.input();
    let currently_dragging_from_egui_area = state.pointer.press_origin().map(|pos| !scene_rect.contains(pos)).unwrap_or(false);
    // TODO: Also check that no windows (egui::Window) are under the mouse cursor inside the available rect
    // TODO: and other edge cases
    currently_inside_egui_area || currently_dragging_from_egui_area
}

