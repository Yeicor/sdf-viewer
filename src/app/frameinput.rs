use eframe::egui;

/// Translates from egui input to three-d input
pub struct FrameInput<'a> {
    pub screen: three_d::RenderTarget<'a>,
    pub viewport: three_d::Viewport,
    pub scissor_box: three_d::ScissorBox,
}

impl FrameInput<'_> {
    //noinspection DuplicatedCode
    pub fn new(
        context: &three_d::Context,
        info: &egui::PaintCallbackInfo,
        painter: &eframe::egui_glow::Painter,
    ) -> Self {
        use three_d::*;

        // Disable sRGB textures for three-d
        #[cfg(not(target_arch = "wasm32"))]
        #[allow(unsafe_code)]
        unsafe {
            use eframe::glow::HasContext as _;
            context.disable(eframe::glow::FRAMEBUFFER_SRGB);
        }

        // Constructs a screen render target to render the final image to
        let screen = painter.intermediate_fbo().map_or_else(
            || {
                RenderTarget::screen(
                    context,
                    info.viewport.width() as u32,
                    info.viewport.height() as u32,
                )
            },
            |fbo| {
                RenderTarget::from_framebuffer(
                    context,
                    info.viewport.width() as u32,
                    info.viewport.height() as u32,
                    fbo,
                )
            },
        );

        // Set where to paint
        let viewport = info.viewport_in_pixels();
        let viewport = Viewport {
            x: viewport.left_px,
            y: viewport.from_bottom_px,
            width: viewport.width_px as u32,
            height: viewport.height_px as u32,
        };

        // Respect the egui clip region (e.g. if we are inside an `egui::ScrollArea`).
        let clip_rect = info.clip_rect_in_pixels();
        let scissor_box = ScissorBox {
            x: clip_rect.left_px,
            y: clip_rect.from_bottom_px,
            width: clip_rect.width_px as u32,
            height: clip_rect.height_px as u32,
        };
        Self {
            screen,
            scissor_box,
            viewport,
        }
    }
}
