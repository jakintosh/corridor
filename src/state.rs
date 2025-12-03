use crate::graphics::scene::{self, Camera, Scene};
use crate::graphics::{
    CameraBuffer, GpuContext, InstanceBuffer, LightingBuffer, LightingControls, LightingSettings,
    MeshBuffers, Pipeline, render_scene,
};
use crate::graphics::{CameraDebugInfo, EguiIntegration, panels};
use crate::model::Network;
use glam::Quat;
use instant::Instant;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::window::Window;

#[derive(Default)]
struct CameraController {
    mouse_dragging: bool,
    last_mouse_pos: Option<(f32, f32)>,
}

impl CameraController {
    fn handle_event(&mut self, camera: &mut Camera, event: &WindowEvent) -> bool {
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
                true
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.mouse_dragging {
                    if let Some((last_x, last_y)) = self.last_mouse_pos {
                        let delta_x = position.x as f32 - last_x;
                        let delta_y = position.y as f32 - last_y;
                        camera.handle_mouse_drag(delta_x * 0.005, delta_y * 0.005);
                    }
                    self.last_mouse_pos = Some((position.x as f32, position.y as f32));
                    return true;
                }
                false
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_amount = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0,
                };
                camera.handle_scroll(scroll_amount);
                true
            }
            _ => false,
        }
    }
}

pub struct State {
    gpu: GpuContext,
    pub size: winit::dpi::PhysicalSize<u32>,
    pipeline: Pipeline,
    mesh_buffers: Vec<MeshBuffers>,
    instance_buffer: InstanceBuffer,
    camera_buffer: CameraBuffer,
    lighting_buffer: LightingBuffer,
    ui: EguiIntegration,
    scene: Scene,
    camera: Camera,
    lighting_controls: LightingControls,
    camera_controller: CameraController,
    window: std::sync::Arc<Window>,
    start_time: Instant,
}

impl State {
    pub async fn new(window: std::sync::Arc<Window>, graph_path: Option<&str>) -> Self {
        let size = window.inner_size();
        let gpu = GpuContext::new(&window).await;

        let pipeline = Pipeline::new(&gpu.device, gpu.config.format);

        // Create scene from network file or use demo scene
        let scene = if let Some(path) = graph_path {
            load_network_scene(path)
        } else {
            scene::demo::create_demo_scene()
        };

        // Create mesh buffers for all meshes in the scene
        let mesh_buffers: Vec<MeshBuffers> = scene
            .meshes
            .iter()
            .map(|mesh| MeshBuffers::from_mesh(&gpu.device, mesh))
            .collect();

        // Create instance buffer with capacity for all nodes
        let instance_buffer = InstanceBuffer::new(&gpu.device, 1000);

        // Create camera buffer
        let camera_buffer = CameraBuffer::new(&gpu.device, &pipeline.camera_bind_group_layout);

        // Create lighting buffer
        let lighting_buffer =
            LightingBuffer::new(&gpu.device, &pipeline.lighting_bind_group_layout);

        // Create camera
        let aspect_ratio = size.width as f32 / size.height as f32;
        let camera = Camera::new(aspect_ratio);

        let lighting_controls = LightingControls::default();

        let ui = EguiIntegration::new(&gpu.device, gpu.config.format, &window);

        Self {
            gpu,
            size,
            pipeline,
            mesh_buffers,
            instance_buffer,
            camera_buffer,
            lighting_buffer,
            ui,
            scene,
            camera,
            lighting_controls,
            camera_controller: CameraController::default(),
            window,
            start_time: Instant::now(),
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        // Give UI first chance to consume the event
        if self.ui.handle_event(&*self.window, event) {
            return true;
        }

        self.camera_controller.handle_event(&mut self.camera, event)
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.gpu.config.width = new_size.width;
            self.gpu.config.height = new_size.height;
            self.gpu
                .surface
                .configure(&self.gpu.device, &self.gpu.config);

            // Recreate depth texture with new size
            self.gpu.depth_texture =
                GpuContext::create_depth_texture(&self.gpu.device, new_size.width, new_size.height);

            // Update camera aspect ratio
            let aspect_ratio = new_size.width as f32 / new_size.height as f32;
            self.camera.update_aspect_ratio(aspect_ratio);
        }
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let surface_output = self.gpu.surface.get_current_texture()?;
        let view = surface_output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        // Calculate camera position from spherical coordinates (UI readout)
        let cam_x = self.camera.target.x
            + self.camera.distance * self.camera.pitch.cos() * self.camera.yaw.sin();
        let cam_y = self.camera.target.y + self.camera.distance * self.camera.pitch.sin();
        let cam_z = self.camera.target.z
            + self.camera.distance * self.camera.pitch.cos() * self.camera.yaw.cos();

        let camera_debug = CameraDebugInfo {
            position: [cam_x, cam_y, cam_z],
            target: self.camera.target.to_array(),
            yaw: self.camera.yaw,
            pitch: self.camera.pitch,
            distance: self.camera.distance,
            object_count: self.scene.nodes.len(),
        };

        let prepared_ui = {
            let lighting_controls = &mut self.lighting_controls;

            self.ui.begin(
                &*self.window,
                self.gpu.config.width,
                self.gpu.config.height,
                move |ctx| {
                    egui::Window::new("Scene Graph Demo")
                        .default_pos([10.0, 10.0])
                        .show(ctx, |ui| {
                            panels::camera_debug(ui, &camera_debug);
                            ui.separator();
                            panels::lighting(ui, lighting_controls);
                        });
                },
            )
        };

        // Sync lighting controls into engine settings for this frame
        let lighting_settings: LightingSettings = (&self.lighting_controls).into();

        // Render scene
        render_scene(
            &mut encoder,
            &view,
            &self.gpu.depth_texture,
            &self.pipeline.render_pipeline,
            &self.mesh_buffers,
            &self.instance_buffer,
            &self.camera_buffer,
            &self.lighting_buffer,
            &self.gpu.queue,
            &self.scene,
            &self.camera,
            &lighting_settings.to_uniform(),
        );

        // Render egui UI overlay
        self.ui.paint(
            &self.gpu.device,
            &self.gpu.queue,
            &mut encoder,
            &view,
            prepared_ui,
        );

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        surface_output.present();

        Ok(())
    }
}

fn load_network_scene(path: &str) -> Scene {
    let json =
        std::fs::read_to_string(path).expect(&format!("Failed to read network file: {}", path));
    let network: Network = serde_json::from_str(&json).expect("Failed to parse network JSON");
    scene::network::network_to_scene(&network)
}
