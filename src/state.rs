use crate::rendering::{render_scene, CameraBuffer, GpuContext, InstanceBuffer, MeshBuffers, Pipeline};
use crate::scene::{self, Camera, Scene};
use crate::ui::EguiIntegration;
use glam::Quat;
use std::time::Instant;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::window::Window;

pub struct State {
    gpu: GpuContext,
    pub size: winit::dpi::PhysicalSize<u32>,
    pipeline: Pipeline,
    mesh_buffers: Vec<MeshBuffers>,
    instance_buffer: InstanceBuffer,
    camera_buffer: CameraBuffer,
    ui: EguiIntegration,
    scene: Scene,
    camera: Camera,
    mouse_dragging: bool,
    last_mouse_pos: Option<(f32, f32)>,
    window: std::sync::Arc<Window>,
    start_time: Instant,
}

impl State {
    pub async fn new(window: std::sync::Arc<Window>) -> Self {
        let size = window.inner_size();
        let gpu = GpuContext::new(&window).await;

        let pipeline = Pipeline::new(&gpu.device, gpu.config.format);

        // Create demo scene
        let scene = scene::demo::create_demo_scene();

        // Create mesh buffers for all meshes in the scene
        let mesh_buffers: Vec<MeshBuffers> = scene
            .meshes
            .iter()
            .map(|mesh| MeshBuffers::from_mesh(&gpu.device, mesh))
            .collect();

        // Create instance buffer with capacity for all nodes
        let instance_buffer = InstanceBuffer::new(&gpu.device, 100);

        // Create camera buffer
        let camera_buffer = CameraBuffer::new(&gpu.device, &pipeline.bind_group_layout);

        // Create camera
        let aspect_ratio = size.width as f32 / size.height as f32;
        let camera = Camera::new(aspect_ratio);

        let ui = EguiIntegration::new(&gpu.device, gpu.config.format, &window);

        Self {
            gpu,
            size,
            pipeline,
            mesh_buffers,
            instance_buffer,
            camera_buffer,
            ui,
            scene,
            camera,
            mouse_dragging: false,
            last_mouse_pos: None,
            window,
            start_time: Instant::now(),
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        // Handle mouse input for camera controls
        match event {
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.mouse_dragging = *state == ElementState::Pressed;
                if !self.mouse_dragging {
                    self.last_mouse_pos = None;
                }
                return true;
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.mouse_dragging {
                    if let Some((last_x, last_y)) = self.last_mouse_pos {
                        let delta_x = position.x as f32 - last_x;
                        let delta_y = position.y as f32 - last_y;
                        self.camera.handle_mouse_drag(delta_x * 0.005, delta_y * 0.005);
                    }
                    self.last_mouse_pos = Some((position.x as f32, position.y as f32));
                    return true;
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_amount = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0,
                };
                self.camera.handle_scroll(scroll_amount);
                return true;
            }
            _ => {}
        }

        // Pass remaining events to UI
        self.ui.handle_event(&*self.window, event)
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.gpu.config.width = new_size.width;
            self.gpu.config.height = new_size.height;
            self.gpu.surface.configure(&self.gpu.device, &self.gpu.config);

            // Recreate depth texture with new size
            self.gpu.depth_texture = GpuContext::create_depth_texture(
                &self.gpu.device,
                new_size.width,
                new_size.height,
            );

            // Update camera aspect ratio
            let aspect_ratio = new_size.width as f32 / new_size.height as f32;
            self.camera.update_aspect_ratio(aspect_ratio);
        }
    }

    pub fn update(&mut self) {
        let time = self.start_time.elapsed().as_secs_f32();

        // Animate specific nodes (indices 1, 2, 3 are the animated cubes)
        if self.scene.nodes.len() > 1 {
            self.scene.nodes[1].transform.rotation = Quat::from_rotation_y(time * 1.0);
        }
        if self.scene.nodes.len() > 2 {
            self.scene.nodes[2].transform.rotation = Quat::from_rotation_x(time * 1.5);
        }
        if self.scene.nodes.len() > 3 {
            self.scene.nodes[3].transform.rotation =
                Quat::from_euler(glam::EulerRot::ZYX, time * 0.5, time * 1.3, time * 0.7);
        }
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

        // Render scene
        render_scene(
            &mut encoder,
            &view,
            &self.gpu.depth_texture,
            &self.pipeline.render_pipeline,
            &self.mesh_buffers,
            &self.instance_buffer,
            &self.camera_buffer,
            &self.gpu.queue,
            &self.scene,
            &self.camera,
        );

        // Render egui UI
        let camera = &self.camera;

        // Calculate camera position from spherical coordinates
        let cam_x = camera.target.x + camera.distance * camera.pitch.cos() * camera.yaw.sin();
        let cam_y = camera.target.y + camera.distance * camera.pitch.sin();
        let cam_z = camera.target.z + camera.distance * camera.pitch.cos() * camera.yaw.cos();

        self.ui.render(
            &self.gpu.device,
            &self.gpu.queue,
            &mut encoder,
            &view,
            &*self.window,
            self.gpu.config.width,
            self.gpu.config.height,
            |ctx| {
                egui::Window::new("Scene Graph Demo")
                    .default_pos([10.0, 10.0])
                    .show(ctx, |ui| {
                        ui.label("Left-click and drag to orbit camera");
                        ui.label("Scroll wheel to zoom in/out");
                        ui.separator();
                        ui.monospace(format!("pos({:.2}, {:.2}, {:.2})", cam_x, cam_y, cam_z));
                        ui.monospace(format!("look({:.2}, {:.2}, {:.2})",
                            camera.target.x, camera.target.y, camera.target.z));
                        ui.monospace(format!("yaw:{:.2} pitch:{:.2} dist:{:.2}",
                            camera.yaw, camera.pitch, camera.distance));
                        ui.monospace(format!("objects: {}", self.scene.nodes.len()));
                    });
            },
        );

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        surface_output.present();

        Ok(())
    }
}
