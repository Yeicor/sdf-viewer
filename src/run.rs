use std::rc::Rc;
use std::sync::Arc;

use eframe::egui;
use eframe::egui::{InputState, PointerButton, Pos2};
use eframe::egui::mutex::RwLockReadGuard;
use eframe::egui::panel::Side;
use eframe::egui::util::hash;
use three_d::*;

pub struct MyEguiApp {
    three_d_ctx: three_d::Context,
    camera: Camera,
    camera_ctrl: OrbitControl,
    volume: Model<IsourfaceMaterial>,
    light_ambient: AmbientLight,
    lights_dir: Vec<DirectionalLight>,
    last_mouse_pos: Pos2,
}

impl MyEguiApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        eprintln!("Initializing app...");

        // Retrieve Three-D context from the egui context (shared glow dependency).
        let three_d_ctx = Context::from_gl_context(
            unsafe { Arc::from_raw(Rc::into_raw(cc.gl.clone())) }).unwrap();

        let camera = Camera::new_perspective(
            &three_d_ctx,
            Viewport { x: 0, y: 0, width: 0, height: 0 }, // Updated at runtime
            vec3(0.25, -0.5, -2.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            0.1,
            1000.0,
        ).unwrap();
        let camera_ctrl = OrbitControl::new(*camera.target(), 1.0, 100.0);

        // Source: https://web.cs.ucdavis.edu/~okreylos/PhDStudies/Spring2000/ECS277/DataSets.html
        let mut loaded = Loaded::default();
        loaded.insert_bytes("", include_bytes!("../assets/Skull.vol").to_vec());
        let cpu_volume = loaded.vol("").unwrap();
        let mut volume = Model::new_with_material(
            &three_d_ctx,
            &CpuMesh::cube(),
            IsourfaceMaterial {
                voxels: std::rc::Rc::new(Texture3D::new(&three_d_ctx, &cpu_volume.voxels).unwrap()),
                lighting_model: LightingModel::Blinn,
                size: cpu_volume.size,
                threshold: 0.15,
                color: Color::WHITE,
                roughness: 1.0,
                metallic: 0.0,
            },
        ).unwrap();
        volume.material.color = Color::new(25, 125, 25, 255);
        volume.set_transformation(Mat4::from_nonuniform_scale(
            0.5 * cpu_volume.size.x,
            0.5 * cpu_volume.size.y,
            0.5 * cpu_volume.size.z,
        ));

        let ambient = AmbientLight::new(&three_d_ctx, 0.4, Color::WHITE).unwrap();
        let directional1 =
            DirectionalLight::new(&three_d_ctx, 2.0, Color::WHITE, &vec3(-1.0, -1.0, -1.0)).unwrap();
        let directional2 =
            DirectionalLight::new(&three_d_ctx, 2.0, Color::WHITE, &vec3(1.0, 1.0, 1.0)).unwrap();

        eprintln!("Initialization complete! Starting main loop...");
        Self {
            three_d_ctx,
            camera,
            camera_ctrl,
            volume,
            light_ambient: ambient,
            lights_dir: vec![directional1, directional2],
            last_mouse_pos: Pos2::new(0., 0.),
        }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        // === GUI (only defined here, drawn after update() finishes) ===
        egui::SidePanel::new(Side::Left, hash(""))
            // .frame(Frame {
            //     fill: Color32::from_black_alpha(0),
            //     ..Frame::default()
            // })
            .show(ctx, |ui| {
                ui.heading("Hello World!");
            });

        // === Draw Three-D scene ===
        let viewport_rect = ctx.available_rect();
        let viewport = Viewport {
            x: (viewport_rect.min.x * ctx.pixels_per_point()) as i32,
            y: (viewport_rect.min.y * ctx.pixels_per_point()) as i32,
            width: (viewport_rect.width() * ctx.pixels_per_point()) as u32,
            height: (viewport_rect.height() * ctx.pixels_per_point()) as u32,
        };
        self.camera.set_viewport(viewport).unwrap();
        // Handle inputs
        let mut events = self.translate_input_events(ctx.input());
        self.camera_ctrl.handle_events(&mut self.camera, &mut events).unwrap();
        // Collect lights
        let mut lights = self.lights_dir.iter().map(|e| e as &dyn Light).collect::<Vec<&dyn Light>>();
        lights.push(&self.light_ambient);
        // draw volume to screen
        Screen::write(
            &self.three_d_ctx,
            ClearState::color_and_depth(0.0, 0.1, 0.4, 1.0, 1.0),
            || {
                render_pass(
                    &self.camera,
                    &[&self.volume],
                    lights.as_slice(),
                )?;
                Ok(())
            },
        ).unwrap();
    }
}

impl MyEguiApp {
    pub(crate) fn translate_input_events(&mut self, state: RwLockReadGuard<InputState>) -> Vec<three_d::Event> {
        let mut events = vec![];
        for ev in &state.events {
            // Mostly, just translate events to an old version of egui...
            let possible_translation = match ev {
                egui::Event::PointerButton { pos, button, pressed, modifiers } => {
                    let button = match button {
                        egui::PointerButton::Primary => MouseButton::Left, // Note: assumes primary is left :(
                        egui::PointerButton::Middle => MouseButton::Middle,
                        egui::PointerButton::Secondary => MouseButton::Right,
                    };
                    let position = (pos.x as f64, pos.y as f64);
                    let modifiers = Self::translate_input_modifiers(modifiers);
                    Some(if *pressed {
                        three_d::Event::MousePress { button, position, modifiers, handled: modifiers.alt } // TODO: handled: detect if interacting with egui
                    } else {
                        three_d::Event::MouseRelease { button, position, modifiers, handled: modifiers.alt }
                    })
                }
                egui::Event::PointerMoved(new_pos) => {
                    let delta = *new_pos - self.last_mouse_pos;
                    self.last_mouse_pos = new_pos.clone();
                    let delta = (delta.x as f64, delta.y as f64);
                    let button = if state.pointer.button_down(PointerButton::Primary) {
                        Some(MouseButton::Left)
                    } else if state.pointer.button_down(PointerButton::Secondary) {
                        Some(MouseButton::Right)
                    } else if state.pointer.button_down(PointerButton::Middle) {
                        Some(MouseButton::Middle)
                    } else { None };
                    let position = (new_pos.x as f64, new_pos.y as f64);
                    let modifiers = Self::translate_input_modifiers(&state.modifiers);
                    Some(three_d::Event::MouseMotion { button, delta, position, modifiers, handled: modifiers.alt })
                }
                // egui::Event::MouseWheel { delta, position, modifiers, handled } => {
                //     three_d::Event::MouseWheel { delta, position, modifiers, handled }
                // }
                // egui::Event::MouseEnter => {
                //     three_d::Event::MouseEnter
                // }
                // egui::Event::MouseLeave => {
                //     three_d::Event::MouseLeave
                // }
                // egui::Event::KeyPress { kind, modifiers, handled } => {
                //     three_d::Event::KeyPress { kind, modifiers, handled }
                // }
                // egui::Event::KeyRelease { kind, modifiers, handled } => {
                //     three_d::Event::KeyRelease { kind, modifiers, handled }
                // }
                // egui::Event::ModifiersChange { modifiers } => {
                //     three_d::Event::ModifiersChange { modifiers }
                // }
                // egui::Event::Text(val) => {
                //     three_d::Event::Text(val)
                // }
                _ => {
                    eprintln!("Ignored egui input event: {:?}", ev);
                    None
                }
            };
            if let Some(new_ev) = possible_translation {
                events.push(new_ev);
            }
        }
        events
    }

    fn translate_input_modifiers(modifiers: &egui::Modifiers) -> three_d::Modifiers {
        three_d::Modifiers {
            alt: modifiers.alt,
            ctrl: modifiers.ctrl,
            shift: modifiers.shift,
            command: modifiers.command,
        }
    }
}
