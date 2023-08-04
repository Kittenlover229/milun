use egui::Context as EguiContext;
use egui_wgpu::{renderer::ScreenDescriptor, Renderer as EguiRenderer};
use wgpu::{Device, Queue, SurfaceConfiguration, TextureView};
use winit::{event::WindowEvent, window::Window};

/// Manages all the processing done by egui.
pub(crate) struct EguiIntegration {
    pub(crate) input_state: egui_winit::State,
    pub(crate) context: EguiContext,
    pub(crate) egui_renderer: EguiRenderer,
}

impl EguiIntegration {
    pub(crate) fn new(
        window: &winit::window::Window,
        device: &egui_wgpu::wgpu::Device,
        surface_format: egui_wgpu::wgpu::TextureFormat,
    ) -> Self {
        let egui_renderer = EguiRenderer::new(device, surface_format, None, 1);
        let egui_context = EguiContext::default();
        let egui_input_state = egui_winit::State::new(window);

        Self {
            input_state: egui_input_state,
            context: egui_context,
            egui_renderer,
        }
    }
}

impl EguiIntegration {
    pub(crate) fn input(&mut self, event: &WindowEvent) -> bool {
        self.input_state.on_event(&self.context, event).consumed
    }

    pub(crate) fn begin_frame(&mut self, window: &Window) {
        self.context
            .begin_frame(self.input_state.take_egui_input(window));
    }

    pub(crate) fn end_frame(
        &mut self,
        window: &Window,
        device: &Device,
        queue: &Queue,
        config: &SurfaceConfiguration,
        view: &TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let egui_output = self.context.end_frame();
        if egui_output.shapes.is_empty() {
            return;
        }

        let egui_paint_job = self.context.tessellate(egui_output.shapes);

        for (id, image_delta) in egui_output.textures_delta.set {
            self.egui_renderer
                .update_texture(device, queue, id, &image_delta);
        }

        for id in egui_output.textures_delta.free {
            self.egui_renderer.free_texture(&id);
        }

        self.egui_renderer.update_buffers(
            device,
            queue,
            encoder,
            &egui_paint_job,
            &ScreenDescriptor {
                pixels_per_point: 1.,
                size_in_pixels: [config.width, config.height],
            },
        );

        self.input_state
            .handle_platform_output(window, &self.context, egui_output.platform_output);

        let mut egui_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Egui Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        self.egui_renderer.render(
            &mut egui_render_pass,
            &egui_paint_job,
            &ScreenDescriptor {
                pixels_per_point: self.input_state.pixels_per_point(),
                size_in_pixels: [config.width, config.height],
            },
        );
    }
}
