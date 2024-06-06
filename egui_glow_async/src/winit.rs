use parking_lot::Mutex;
use std::sync::Arc;

use egui_async_winit as egui_winit;

pub use egui_async_winit;
pub use egui_winit::EventResponse;

use egui::{ViewportId, ViewportOutput};
use egui_async_winit::winit as winit;

use crate::shader_version::ShaderVersion;

/// Use [`egui`] from a [`glow`] app based on [`winit`].
pub struct EguiGlow {
    pub egui_ctx: egui::Context,
    pub egui_winit: Arc<Mutex<egui_async_winit::State>>,
    pub painter: crate::Painter,

    viewport_info: egui::ViewportInfo,

    // output from the last update:
    shapes: Vec<egui::epaint::ClippedShape>,
    pixels_per_point: f32,
    textures_delta: egui::TexturesDelta,
}

impl EguiGlow {
    /// For automatic shader version detection set `shader_version` to `None`.
    pub fn new<TS: winit::ThreadSafety>(
        event_loop: &winit::event_loop::EventLoopWindowTarget<TS>,
        gl: std::sync::Arc<glow::Context>,
        shader_version: Option<ShaderVersion>,
        native_pixels_per_point: Option<f32>,
    ) -> Self {
        let painter = crate::Painter::new(gl, "", shader_version)
            .map_err(|err| {
                log::error!("error occurred in initializing painter:\n{err}");
            })
            .unwrap();

        let egui_ctx = egui::Context::default();

        let egui_winit = egui_winit::State::new(
            egui_ctx.clone(),
            ViewportId::ROOT,
            event_loop,
            native_pixels_per_point,
            Some(painter.max_texture_side()),
        );

        Self {
            egui_ctx,
            egui_winit: Arc::new(Mutex::new(egui_winit)),
            painter,
            viewport_info: Default::default(),
            shapes: Default::default(),
            pixels_per_point: native_pixels_per_point.unwrap_or(1.0),
            textures_delta: Default::default(),
        }
    }

    pub fn on_window_event<TS: winit::ThreadSafety>(
        &mut self,
        window: &winit::window::Window<TS>,
        event: &winit::event::WindowEvent,
    ) -> EventResponse {
        self.egui_winit.lock().on_window_event(window, event)
    }

    /// Call [`Self::paint`] later to paint.
    pub async fn run<TS: winit::ThreadSafety>(&mut self, window: &winit::window::Window<TS>, run_ui: impl FnMut(&egui::Context)) {
        let raw_input = self.egui_winit.lock().take_egui_input(window).await;

        let egui::FullOutput {
            platform_output,
            textures_delta,
            shapes,
            pixels_per_point,
            viewport_output,
        } = self.egui_ctx.run(raw_input, run_ui);

        if viewport_output.len() > 1 {
            log::warn!("Multiple viewports not yet supported by EguiGlow");
        }
        for (_, ViewportOutput { commands, .. }) in viewport_output {
            let mut screenshot_requested = false;
            egui_winit::process_viewport_commands(
                &self.egui_ctx,
                &mut self.viewport_info,
                commands,
                window,
                true,
                &mut screenshot_requested,
            );
            if screenshot_requested {
                log::warn!("Screenshot not yet supported by EguiGlow");
            }
        }

        self.egui_winit.lock()
            .handle_platform_output(window, platform_output);

        self.shapes = shapes;
        self.pixels_per_point = pixels_per_point;
        self.textures_delta.append(textures_delta);
    }

    /// Paint the results of the last call to [`Self::run`].
    pub async fn paint<TS: winit::ThreadSafety>(&mut self, window: &winit::window::Window<TS>) {
        let shapes = std::mem::take(&mut self.shapes);
        let mut textures_delta = std::mem::take(&mut self.textures_delta);

        for (id, image_delta) in textures_delta.set {
            self.painter.set_texture(id, &image_delta);
        }

        let pixels_per_point = self.pixels_per_point;
        let clipped_primitives = self.egui_ctx.tessellate(shapes, pixels_per_point);
        let dimensions: [u32; 2] = window.inner_size().await.into();
        self.painter
            .paint_primitives(dimensions, pixels_per_point, &clipped_primitives);

        for id in textures_delta.free.drain(..) {
            self.painter.free_texture(id);
        }
    }

    /// Call to release the allocated graphics resources.
    pub fn destroy(&mut self) {
        self.painter.destroy();
    }
}
