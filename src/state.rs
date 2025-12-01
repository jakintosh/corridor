use crate::geometry::index_count;
use crate::rendering::{render_cube, Buffers, GpuContext, Pipeline};
use crate::ui::EguiIntegration;
use std::time::Instant;
use winit::event::WindowEvent;
use winit::window::Window;

pub struct State {
    gpu: GpuContext,
    pub size: winit::dpi::PhysicalSize<u32>,
    pipeline: Pipeline,
    buffers: Buffers,
    ui: EguiIntegration,
    rotation: f32,
    spin_direction: f32,
    window: std::sync::Arc<Window>,
    start_time: Instant,
}

impl State {
    pub async fn new(window: std::sync::Arc<Window>) -> Self {
        let size = window.inner_size();
        let gpu = GpuContext::new(&window).await;

        let pipeline = Pipeline::new(&gpu.device, gpu.config.format);
        let buffers = Buffers::new(&gpu.device, &pipeline.bind_group_layout);
        let ui = EguiIntegration::new(&gpu.device, gpu.config.format, &window);

        Self {
            gpu,
            size,
            pipeline,
            buffers,
            ui,
            rotation: 0.0,
            spin_direction: 1.0,
            window,
            start_time: Instant::now(),
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        self.ui.handle_event(&*self.window, event)
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.gpu.config.width = new_size.width;
            self.gpu.config.height = new_size.height;
            self.gpu.surface.configure(&self.gpu.device, &self.gpu.config);
        }
    }

    pub fn update(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let rotation_speed = 1.0; // radians per second
        self.rotation = elapsed * rotation_speed * self.spin_direction;

        self.buffers.update_uniforms(&self.gpu.queue, self.rotation);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let surface_output = self.gpu.surface.get_current_texture()?;
        let view = surface_output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        // Render cube
        render_cube(
            &mut encoder,
            &view,
            &self.pipeline.render_pipeline,
            &self.buffers,
            index_count(),
        );

        // Render egui UI
        let spin_direction = &mut self.spin_direction;
        self.ui.render(
            &self.gpu.device,
            &self.gpu.queue,
            &mut encoder,
            &view,
            &*self.window,
            self.gpu.config.width,
            self.gpu.config.height,
            |ctx| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show(ctx, |ui| {
                        ui.label("Controls:");
                        if ui.button("Reverse Direction").clicked() {
                            *spin_direction *= -1.0;
                        }
                    });
            },
        );

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        surface_output.present();

        Ok(())
    }
}
