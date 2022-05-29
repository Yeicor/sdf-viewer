use eframe::egui;
use eframe::egui::{InputState, PointerButton, Pos2, Rect, Vec2};
use eframe::egui::mutex::RwLockReadGuard;
use tracing::{info, warn};

#[derive(Debug, Clone, Default)]
pub(crate) struct InputTranslator {
    last_mouse_pos: Pos2,
    scroll_abs: Pos2,
    mouse_left: bool,
}

impl InputTranslator {
    #[profiling::function]
    pub(crate) fn translate_input_events(&mut self, ctx: &egui::Context) -> Vec<three_d::Event> {
        let mut events = vec![];
        let rect = ctx.available_rect();
        let state = ctx.input();
        for ev in &state.events {
            // Mostly, just translate events to an old version of egui...
            let possible_translation = match ev {
                egui::Event::PointerButton { pos, button, pressed, modifiers } => {
                    let button = match button {
                        egui::PointerButton::Primary => three_d::MouseButton::Left, // Note: assumes primary is left :(
                        egui::PointerButton::Middle => three_d::MouseButton::Middle,
                        egui::PointerButton::Secondary => three_d::MouseButton::Right,
                    };
                    let position = (pos.x as f64, pos.y as f64);
                    let modifiers = Self::translate_modifiers(modifiers);
                    let handled = Self::is_event_handled_by_egui(ctx, &state, rect, self.last_mouse_pos);
                    Some(if *pressed {
                        three_d::Event::MousePress { button, position, modifiers, handled } // TODO: handled: detect if interacting with egui
                    } else {
                        three_d::Event::MouseRelease { button, position, modifiers, handled }
                    })
                }
                egui::Event::PointerMoved(new_pos) => {
                    if self.mouse_left {
                        events.push(three_d::Event::MouseEnter);
                        self.mouse_left = false
                    }
                    let delta = *new_pos - self.last_mouse_pos;
                    self.last_mouse_pos = new_pos.clone();
                    let delta = (delta.x as f64, delta.y as f64);
                    let button = if state.pointer.button_down(PointerButton::Primary) {
                        Some(three_d::MouseButton::Left)
                    } else if state.pointer.button_down(PointerButton::Secondary) {
                        Some(three_d::MouseButton::Right)
                    } else if state.pointer.button_down(PointerButton::Middle) {
                        Some(three_d::MouseButton::Middle)
                    } else { None };
                    let position = (new_pos.x as f64, new_pos.y as f64);
                    let modifiers = Self::translate_modifiers(&state.modifiers);
                    let handled = Self::is_event_handled_by_egui(ctx, &state, rect, self.last_mouse_pos);
                    Some(three_d::Event::MouseMotion { button, delta, position, modifiers, handled })
                }
                egui::Event::Scroll(delta) => {
                    // Cross-platform scrolling (each OS/arch has its own force for scrolling)
                    let scroll_fn = |val: f32| if val.abs() < 1e-6 { 0.0 } else { val.signum() * 3. };
                    let delta = (scroll_fn(delta.x) as f64, scroll_fn(delta.y) as f64);
                    self.scroll_abs += Vec2::new(delta.0 as f32, delta.1 as f32);
                    // info!("scroll: {:?}, delta: {:?}", self.scroll_abs, delta);
                    let position = (self.scroll_abs.x as f64, self.scroll_abs.y as f64);
                    let modifiers = Self::translate_modifiers(&state.modifiers);
                    let handled = Self::is_event_handled_by_egui(ctx, &state, rect, self.last_mouse_pos);
                    // FIXME: Emulate two-finger scroll on web + android.
                    Some(three_d::Event::MouseWheel { delta, position, modifiers, handled })
                }
                // three_d::Event::MouseEnter is emulated (see PointerMoved)
                egui::Event::PointerGone => {
                    self.mouse_left = true;
                    Some(three_d::Event::MouseLeave)
                }
                egui::Event::Key { key, pressed, modifiers } => {
                    let kind = Self::translate_key(key);
                    let modifiers = Self::translate_modifiers(modifiers);
                    let handled = Self::is_event_handled_by_egui(ctx, &state, rect, self.last_mouse_pos);
                    if *pressed {
                        Some(three_d::Event::KeyPress { kind, modifiers, handled })
                    } else {
                        Some(three_d::Event::KeyRelease { kind, modifiers, handled })
                    }
                }
                // egui::Event::ModifiersChange { modifiers } => { // Not available?
                //     three_d::Event::ModifiersChange { modifiers }
                // }
                egui::Event::Text(val) => {
                    Some(three_d::Event::Text(val.clone()))
                }
                _ => {
                    warn!("Ignored egui input event: {:?}", ev);
                    None
                }
            };
            if let Some(new_ev) = possible_translation {
                events.push(new_ev);
            }
        }
        events
    }

    fn is_event_handled_by_egui(_ctx: &egui::Context, state: &RwLockReadGuard<InputState>, rect: Rect, pos: Pos2) -> bool {
        let currently_inside_egui_area = !rect.contains(pos);
        // Also handle when dragging starts outside the render area
        let currently_dragging_from_egui_area = if let Some(press_origin) = state.pointer.press_origin() {
            !rect.contains(press_origin)
        } else { false };
        // TODO: Also check that no windows (egui::Window) are under the mouse cursor inside the available rect
        // TODO: and other edge cases
        currently_inside_egui_area || currently_dragging_from_egui_area
    }

    fn translate_modifiers(modifiers: &egui::Modifiers) -> three_d::Modifiers {
        three_d::Modifiers {
            alt: modifiers.alt,
            ctrl: modifiers.ctrl,
            shift: modifiers.shift,
            command: modifiers.command,
        }
    }

    fn translate_key(key: &egui::Key) -> three_d::Key {
        match key { // Keep in sync with egui (will fail to build otherwise)
            egui::Key::ArrowDown => three_d::Key::ArrowDown,
            egui::Key::ArrowLeft => three_d::Key::ArrowLeft,
            egui::Key::ArrowRight => three_d::Key::ArrowRight,
            egui::Key::ArrowUp => three_d::Key::ArrowUp,

            egui::Key::Escape => three_d::Key::Escape,
            egui::Key::Tab => three_d::Key::Tab,
            egui::Key::Backspace => three_d::Key::Backspace,
            egui::Key::Enter => three_d::Key::Enter,
            egui::Key::Space => three_d::Key::Space,

            egui::Key::Insert => three_d::Key::Insert,
            egui::Key::Delete => three_d::Key::Delete,
            egui::Key::Home => three_d::Key::Home,
            egui::Key::End => three_d::Key::End,
            egui::Key::PageUp => three_d::Key::PageUp,
            egui::Key::PageDown => three_d::Key::PageDown,

            egui::Key::Num0 => three_d::Key::Num0,
            egui::Key::Num1 => three_d::Key::Num1,
            egui::Key::Num2 => three_d::Key::Num2,
            egui::Key::Num3 => three_d::Key::Num3,
            egui::Key::Num4 => three_d::Key::Num4,
            egui::Key::Num5 => three_d::Key::Num5,
            egui::Key::Num6 => three_d::Key::Num6,
            egui::Key::Num7 => three_d::Key::Num7,
            egui::Key::Num8 => three_d::Key::Num8,
            egui::Key::Num9 => three_d::Key::Num9,

            egui::Key::A => three_d::Key::A,
            egui::Key::B => three_d::Key::B,
            egui::Key::C => three_d::Key::C,
            egui::Key::D => three_d::Key::D,
            egui::Key::E => three_d::Key::E,
            egui::Key::F => three_d::Key::F,
            egui::Key::G => three_d::Key::G,
            egui::Key::H => three_d::Key::H,
            egui::Key::I => three_d::Key::I,
            egui::Key::J => three_d::Key::J,
            egui::Key::K => three_d::Key::K,
            egui::Key::L => three_d::Key::L,
            egui::Key::M => three_d::Key::M,
            egui::Key::N => three_d::Key::N,
            egui::Key::O => three_d::Key::O,
            egui::Key::P => three_d::Key::P,
            egui::Key::Q => three_d::Key::Q,
            egui::Key::R => three_d::Key::R,
            egui::Key::S => three_d::Key::S,
            egui::Key::T => three_d::Key::T,
            egui::Key::U => three_d::Key::U,
            egui::Key::V => three_d::Key::V,
            egui::Key::W => three_d::Key::W,
            egui::Key::X => three_d::Key::X,
            egui::Key::Y => three_d::Key::Y,
            egui::Key::Z => three_d::Key::Z,
        }
    }
}
