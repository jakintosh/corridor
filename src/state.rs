use crate::geometry::index_count;
use crate::rendering::{render_cube, Buffers, Pipeline};
use crate::ui::EguiIntegration;
use std::time::Instant;
use winit::event::WindowEvent;
use winit::window::Window;

pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
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
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::all(),
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to request adapter - no compatible GPU found.");

        let (device, queue) = adapter
            .request_device(&Default::default())
            .await
            .expect("Failed to request device");

        let surface_caps = surface.get_capabilities(&adapter);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps.formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let pipeline = Pipeline::new(&device, config.format);
        let buffers = Buffers::new(&device, &pipeline.bind_group_layout);
        let ui = EguiIntegration::new(&device, config.format, &window);

        Self {
            surface,
            device,
            queue,
            config,
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
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn update(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let rotation_speed = 1.0; // radians per second
        self.rotation = elapsed * rotation_speed * self.spin_direction;

        self.buffers.update_uniforms(&self.queue, self.rotation);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let surface_output = self.surface.get_current_texture()?;
        let view = surface_output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
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
            &self.device,
            &self.queue,
            &mut encoder,
            &view,
            &*self.window,
            self.config.width,
            self.config.height,
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

        self.queue.submit(std::iter::once(encoder.finish()));
        surface_output.present();

        Ok(())
    }
}
