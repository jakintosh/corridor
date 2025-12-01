use egui_wgpu::ScreenDescriptor;
use winit::{event::WindowEvent, window::Window};

pub struct EguiIntegration {
    renderer: egui_wgpu::Renderer,
    context: egui::Context,
    state: egui_winit::State,
}

impl EguiIntegration {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        window: &Window,
    ) -> Self {
        let context = egui::Context::default();
        let renderer = egui_wgpu::Renderer::new(device, surface_format, Default::default());
        let state = egui_winit::State::new(
            context.clone(),
            egui::ViewportId::ROOT,
            window,
            None,
            None,
            None,
        );

        Self {
            renderer,
            context,
            state,
        }
    }

    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.state.on_window_event(window, event).consumed
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        window: &Window,
        config_width: u32,
        config_height: u32,
        ui_fn: impl FnOnce(&egui::Context),
    ) {
        let raw_input = self.state.take_egui_input(window);
        self.context.begin_pass(raw_input);

        ui_fn(&self.context);

        let egui_output = self.context.end_pass();
        self.state
            .handle_platform_output(window, egui_output.platform_output);
        let paint_jobs = self
            .context
            .tessellate(egui_output.shapes, self.context.pixels_per_point());

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [config_width, config_height],
            pixels_per_point: self.context.pixels_per_point(),
        };

        for (id, image_delta) in &egui_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }

        for id in &egui_output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        self.renderer.update_buffers(
            device,
            queue,
            encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        self.renderer.render(
            &mut render_pass.forget_lifetime(),
            &paint_jobs,
            &screen_descriptor,
        );
    }
}
