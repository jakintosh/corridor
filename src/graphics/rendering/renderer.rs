use super::buffers::{CameraBuffer, InstanceBuffer, LightingBuffer, LightingUniform, MeshBuffers};
use crate::graphics::scene::Scene;

pub fn render_scene(
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    depth_view: &wgpu::TextureView,
    render_pipeline: &wgpu::RenderPipeline,
    mesh_buffers: &[MeshBuffers],
    instance_buffer: &InstanceBuffer,
    camera_buffer: &CameraBuffer,
    lighting_buffer: &LightingBuffer,
    queue: &wgpu::Queue,
    scene: &Scene,
    lighting: &LightingUniform,
) {
    // Update lighting uniform
    lighting_buffer.update(queue, lighting);

    // Begin render pass
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Scene Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.1,
                    g: 0.1,
                    b: 0.1,
                    a: 1.0,
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

    render_pass.set_pipeline(render_pipeline);
    render_pass.set_bind_group(0, &camera_buffer.bind_group, &[]);
    render_pass.set_bind_group(1, &lighting_buffer.bind_group, &[]);
    render_pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

    draw_batched_instances(&mut render_pass, mesh_buffers, scene);
}

pub(crate) fn draw_batched_instances<'a>(
    render_pass: &mut wgpu::RenderPass<'a>,
    mesh_buffers: &'a [MeshBuffers],
    scene: &'a Scene,
) {
    let mut current_mesh: Option<usize> = None;
    let mut instance_start = 0;
    let mut instance_count = 0;

    for (i, node) in scene.nodes.iter().enumerate() {
        if current_mesh != Some(node.mesh_id) {
            // Draw previous batch if any
            if let Some(mesh_id) = current_mesh {
                let mesh_buf = &mesh_buffers[mesh_id];
                render_pass.set_vertex_buffer(0, mesh_buf.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(mesh_buf.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(
                    0..mesh_buf.index_count,
                    0,
                    instance_start..instance_start + instance_count,
                );
            }

            // Start new batch
            current_mesh = Some(node.mesh_id);
            instance_start = i as u32;
            instance_count = 1;
        } else {
            instance_count += 1;
        }
    }

    // Draw final batch
    if let Some(mesh_id) = current_mesh {
        let mesh_buf = &mesh_buffers[mesh_id];
        render_pass.set_vertex_buffer(0, mesh_buf.vertex_buffer.slice(..));
        render_pass.set_index_buffer(mesh_buf.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(
            0..mesh_buf.index_count,
            0,
            instance_start..instance_start + instance_count,
        );
    }
}
