use super::buffers::{CameraBuffer, InstanceBuffer, InstanceData, MeshBuffers};
use super::renderer::draw_batched_instances;
use crate::graphics::geometry::Vertex;
use crate::graphics::scene::Scene;
use crate::graphics::shaders;
use std::sync::mpsc;

pub struct PickingPass {
    pipeline: wgpu::RenderPipeline,
    debug_pipeline: Option<DebugOverlayPipeline>,
    shader: wgpu::ShaderModule,
    picking_texture: wgpu::Texture,
    picking_view: wgpu::TextureView,
    readback_buffer: wgpu::Buffer,
    pending_pick: Option<PendingPick>,
    size: (u32, u32),
}

struct DebugOverlayPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    color_format: wgpu::TextureFormat,
}

struct PendingPick {
    pixel_coords: (u32, u32),
    frame_submitted: bool,
    map_requested: bool,
    receiver: Option<std::sync::mpsc::Receiver<Result<(), wgpu::BufferAsyncError>>>,
}

impl PickingPass {
    pub fn new(
        device: &wgpu::Device,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
    ) -> Self {
        // Clamp to a valid texture size (WebGPU disallows zero dimensions)
        let width = width.max(1);
        let height = height.max(1);

        // Create picking texture (R32Uint format)
        let picking_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Picking Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Uint,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let picking_view = picking_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create readback buffer (aligned to row stride; we'll read first 4 bytes)
        let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Picking Readback Buffer"),
            size: wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Create picking shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Picking Shader"),
            source: wgpu::ShaderSource::Wgsl(shaders::picking_shader_source().into()),
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Picking Pipeline Layout"),
            bind_group_layouts: &[camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Picking Pipeline"),
            layout: Some(&pipeline_layout),
            multiview: None,
            cache: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Vertex::desc(), InstanceData::picking_desc()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::R32Uint,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
        });

        Self {
            pipeline,
            debug_pipeline: None,
            shader,
            picking_texture,
            picking_view,
            readback_buffer,
            pending_pick: None,
            size: (width, height),
        }
    }

    pub fn request_pick(&mut self, window_x: u32, window_y: u32, scale_factor: f64) {
        // On native, winit's WindowEvent coords are already physical; on web, apply the scale factor.
        let factor = if cfg!(target_arch = "wasm32") {
            scale_factor
        } else {
            1.0
        };
        let physical_x = (window_x as f64 * factor) as u32;
        let physical_y = (window_y as f64 * factor) as u32;

        let texture_y = physical_y;

        // Bounds check
        if physical_x >= self.size.0 || texture_y >= self.size.1 {
            return;
        }

        self.pending_pick = Some(PendingPick {
            pixel_coords: (physical_x, texture_y),
            frame_submitted: false,
            map_requested: false,
            receiver: None,
        });
    }

    pub fn should_execute(&self) -> bool {
        self.pending_pick
            .as_ref()
            .map_or(false, |p| !p.frame_submitted)
    }

    pub fn execute_pick(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        depth_view: &wgpu::TextureView,
        mesh_buffers: &[MeshBuffers],
        instance_buffer: &InstanceBuffer,
        camera_buffer: &CameraBuffer,
        scene: &Scene,
    ) {
        let pending = match &self.pending_pick {
            Some(p) if !p.frame_submitted => p,
            _ => return,
        };

        let (pixel_x, pixel_y) = pending.pixel_coords;

        // Begin render pass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Picking Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.picking_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: u32::MAX as f64,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &camera_buffer.bind_group, &[]);
        render_pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

        draw_batched_instances(&mut render_pass, mesh_buffers, scene);

        drop(render_pass);

        // Copy the target pixel into the readback buffer for CPU access
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.picking_texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: pixel_x,
                    y: pixel_y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.readback_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT),
                    rows_per_image: Some(1),
                },
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        // Mark as submitted
        if let Some(pending) = self.pending_pick.as_mut() {
            pending.frame_submitted = true;
        }
    }

    pub fn poll_result(&mut self, device: &wgpu::Device) -> Option<u32> {
        let pending = self.pending_pick.as_mut()?;
        if !pending.frame_submitted {
            return None;
        }

        if !pending.map_requested {
            let buffer_slice = self.readback_buffer.slice(..);
            let (sender, receiver) = mpsc::channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                let _ = sender.send(result);
            });
            pending.map_requested = true;
            pending.receiver = Some(receiver);
            return None;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = device.poll(wgpu::PollType::Poll);
        }
        #[cfg(target_arch = "wasm32")]
        let _ = device;

        let receiver = pending.receiver.as_ref()?;
        match receiver.try_recv() {
            Ok(Ok(())) => {
                let buffer_slice = self.readback_buffer.slice(..);
                let data = buffer_slice.get_mapped_range();
                let value = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                drop(data);
                self.readback_buffer.unmap();
                self.pending_pick = None;
                Some(value)
            }
            Ok(Err(_)) => {
                self.readback_buffer.unmap();
                self.pending_pick = None;
                None
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.pending_pick = None;
                None
            }
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        // Cancel any pending picks
        self.pending_pick = None;

        let width = width.max(1);
        let height = height.max(1);

        // Recreate picking texture with new size
        self.picking_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Picking Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Uint,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        self.picking_view = self
            .picking_texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.size = (width, height);
    }

    pub fn render_debug_overlay(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        color_format: wgpu::TextureFormat,
    ) {
        self.ensure_debug_pipeline(device, color_format);

        let Some(debug) = &self.debug_pipeline else {
            return;
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Picking Debug Overlay Bind Group"),
            layout: &debug.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&self.picking_view),
            }],
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Picking Debug Overlay"),
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

        render_pass.set_viewport(0.0, 0.0, self.size.0 as f32, self.size.1 as f32, 0.0, 1.0);
        render_pass.set_pipeline(&debug.pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }

    fn ensure_debug_pipeline(&mut self, device: &wgpu::Device, color_format: wgpu::TextureFormat) {
        if let Some(debug) = &self.debug_pipeline {
            if debug.color_format == color_format {
                return;
            }
        }

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Picking Debug Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Uint,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Picking Debug Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Picking Debug Pipeline"),
            layout: Some(&pipeline_layout),
            multiview: None,
            cache: None,
            vertex: wgpu::VertexState {
                module: &self.shader,
                entry_point: Some("vs_overlay"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &self.shader,
                entry_point: Some("fs_overlay"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
        });

        self.debug_pipeline = Some(DebugOverlayPipeline {
            pipeline,
            bind_group_layout,
            color_format,
        });
    }
}
