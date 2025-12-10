use crate::graphics::scene::{self, Camera, Scene};
use crate::graphics::scene::network::update_network_edges;
use crate::graphics::{
    CameraBuffer, GpuContext, InstanceBuffer, InstanceData, LightingBuffer, LightingControls,
    LightingSettings, MeshBuffers, Pipeline, render_scene,
};
use crate::graphics::{CameraDebugInfo, EguiIntegration, RenderStats, panels};
use crate::model::Network;
use glam::Vec3;
use instant::Instant;
use std::collections::VecDeque;
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
    last_cursor_position: Option<winit::dpi::PhysicalPosition<f64>>,
    last_frame_time: Instant,
    frame_times: VecDeque<f32>,
    frame_count: u64,
}

impl State {
    pub async fn new(window: std::sync::Arc<Window>, network: Option<Network>) -> Self {
        let raw_size = window.inner_size();
        let size = winit::dpi::PhysicalSize::new(raw_size.width.max(1), raw_size.height.max(1));
        let gpu = GpuContext::new(&window).await;

        let pipeline = Pipeline::new(&gpu.device, gpu.config.format);

        // Create scene from network or use demo scene
        let scene = match network {
            Some(network) => scene::network::network_to_scene(&network),
            None => scene::demo::create_demo_scene(),
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
            last_cursor_position: None,
            last_frame_time: Instant::now(),
            frame_times: VecDeque::with_capacity(300),
            frame_count: 0,
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        // Give UI first chance to consume the event
        if self.ui.handle_event(&*self.window, event) {
            return true;
        }

        let mut event_used = false;

        // Track cursor position for picking
        if let WindowEvent::CursorMoved { position, .. } = event {
            self.last_cursor_position = Some(*position);

            // Immediate CPU-based hover detection
            let (ray_origin, ray_dir) = self.camera.screen_to_world_ray(
                position.x as f32,
                position.y as f32,
                self.size.width as f32,
                self.size.height as f32,
            );
            let hovered = self.scene.cpu_pick_ray(ray_origin, ray_dir);
            self.scene.picking.update_hovered_node(hovered);

            if self.scene.picking.is_dragging() {
                let pos = (position.x as f32, position.y as f32);
                self.scene.picking.update_drag(pos);

                // Update dragged node position if node is locked
                if self.scene.picking.is_node_locked() {
                    if let Some(node_id) = self.scene.picking.picked_node {
                        self.update_dragged_node_position(node_id);
                    }
                    event_used = true;
                }
            }
        }

        if let WindowEvent::KeyboardInput { event, .. } = event {
            match event.logical_key.as_ref() {
                _ => {}
            }
        }

        // Handle picking on left click (if not currently dragging for camera)
        if let WindowEvent::MouseInput {
            state: button_state,
            button: MouseButton::Left,
            ..
        } = event
        {
            match button_state {
                ElementState::Pressed => {
                    // Left mouse pressed - start drag
                    if let Some(pos) = self.last_cursor_position {
                        let pos = (pos.x as f32, pos.y as f32);
                        self.scene.picking.start_drag(pos);

                        // Use current hover state to determine drag behavior
                        if let Some(hovered_id) = self.scene.picking.hovered_node {
                            if let Some(node) = self.scene.nodes.get(hovered_id as usize) {
                                if node.selectable {
                                    // Lock this node immediately for dragging
                                    self.lock_node_for_drag(hovered_id);
                                    event_used = true;
                                }
                            }
                        }
                        // If no selectable node hovered, don't set event_used (camera handles it)
                    }
                }
                ElementState::Released => {
                    // Left mouse released - end drag
                    if self.scene.picking.is_dragging() {
                        self.scene.picking.end_drag();
                        event_used = true;
                    }
                }
            }
        }

        // Only allow camera control if we're not dragging a locked node
        let camera_used = if self.scene.picking.is_node_locked() {
            false
        } else {
            self.camera_controller.handle_event(&mut self.camera, event)
        };
        event_used || camera_used
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

    pub fn update(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame_time).as_secs_f32();

        if delta > 0.0 {
            self.frame_times.push_back(delta);
            if self.frame_times.len() > 300 {
                self.frame_times.pop_front();
            }
        }

        self.last_frame_time = now;
        self.frame_count += 1;
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

        // Prepare per-frame buffers once for all passes
        let view_proj = self.camera.view_projection_matrix();
        self.camera_buffer.update(&self.gpu.queue, &view_proj);

        let instance_data: Vec<InstanceData> = self
            .scene
            .nodes
            .iter()
            .enumerate()
            .map(|(idx, node)| {
                let node_id = idx as u32;
                let matrix = self.scene.compute_world_transform(node_id);
                let color = self.scene.materials[node.material_id].color;

                // Compute state flags
                let mut state_flags = 0u32;

                if self.scene.picking.is_dragging() {
                    if let Some(dragged_id) = self.scene.picking.picked_node {
                        // Highlight dragged node AND all its children
                        if node_id == dragged_id || self.is_descendant_of(node_id, dragged_id) {
                            state_flags |= 0x02; // STATE_DRAGGING
                        }
                    }
                } else if self.scene.picking.hovered_node == Some(node_id) {
                    state_flags |= 0x01; // STATE_HOVERED
                }

                InstanceData {
                    matrix,
                    color,
                    state_flags,
                    _padding: [0, 0, 0],
                }
            })
            .collect();
        self.instance_buffer.update(&self.gpu.queue, &instance_data);

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

        // Calculate FPS from frame times
        let current_fps = if let Some(&last_delta) = self.frame_times.back() {
            if last_delta > 0.0 {
                1.0 / last_delta
            } else {
                0.0
            }
        } else {
            0.0
        };

        let avg_fps_1s = {
            let samples: Vec<_> = self.frame_times.iter().rev().take(60).copied().collect();
            if !samples.is_empty() {
                let avg_delta: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
                if avg_delta > 0.0 {
                    1.0 / avg_delta
                } else {
                    0.0
                }
            } else {
                0.0
            }
        };

        let avg_fps_5s = {
            let samples: Vec<_> = self.frame_times.iter().copied().collect();
            if !samples.is_empty() {
                let avg_delta: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
                if avg_delta > 0.0 {
                    1.0 / avg_delta
                } else {
                    0.0
                }
            } else {
                0.0
            }
        };

        // Calculate vertex count
        let vertex_count: usize = self
            .scene
            .nodes
            .iter()
            .map(|node| self.scene.meshes[node.mesh_id].vertices.len())
            .sum();

        let render_stats = RenderStats {
            node_count: self.scene.nodes.len(),
            vertex_count,
            material_count: self.scene.materials.len(),
            current_fps,
            avg_fps_1s,
            avg_fps_5s,
        };

        let prepared_ui = {
            let lighting_controls = &mut self.lighting_controls;
            let hovered_node_id = self.scene.picking.hovered_node;

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
                            ui.separator();
                            panels::hover_info(ui, hovered_node_id);
                            ui.separator();
                            panels::render_stats(ui, &render_stats);
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

    fn update_dragged_node_position(&mut self, node_id: u32) {
        // Cast ray from current cursor position through camera
        if let Some(cursor_pos) = self.last_cursor_position {
            let (origin, direction) = self.camera.screen_to_world_ray(
                cursor_pos.x as f32,
                cursor_pos.y as f32,
                self.size.width as f32,
                self.size.height as f32,
            );

            // Intersect with ground plane (Y=0)
            let ground_plane_point = Vec3::ZERO;
            let ground_plane_normal = Vec3::Y;

            if let Some(ground_pos) = Camera::ray_plane_intersection(
                origin,
                direction,
                ground_plane_point,
                ground_plane_normal,
            ) {
                // Apply the stored offset
                if let Some(offset) = self.scene.picking.get_drag_offset() {
                    let new_position = ground_pos + offset;

                    // Force Y to 0 (ground plane) per user preference
                    let new_position = Vec3::new(new_position.x, 0.0, new_position.z);

                    self.scene.update_node_position(node_id, new_position);

                    // Update all edges connected to this node
                    update_network_edges(&mut self.scene, node_id);
                }
            }
        }
    }

    fn lock_node_for_drag(&mut self, node_id: u32) {
        if let Some(cursor_pos) = self.last_cursor_position {
            let (origin, direction) = self.camera.screen_to_world_ray(
                cursor_pos.x as f32,
                cursor_pos.y as f32,
                self.size.width as f32,
                self.size.height as f32,
            );

            if let Some(click_world_pos) =
                Camera::ray_plane_intersection(origin, direction, Vec3::ZERO, Vec3::Y)
            {
                if let Some(node) = self.scene.nodes.get(node_id as usize) {
                    let offset = node.transform.position - click_world_pos;
                    let offset = Vec3::new(offset.x, 0.0, offset.z);

                    self.scene.picking.update_picked_node(Some(node_id));
                    self.scene.picking.lock_node_with_offset(offset);
                }
            }
        }
    }

    fn is_descendant_of(&self, potential_child: u32, potential_ancestor: u32) -> bool {
        let descendants = self.scene.get_descendants(potential_ancestor);
        descendants.contains(&potential_child)
    }
}
