use super::Buffers;

pub fn render_cube(
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    render_pipeline: &wgpu::RenderPipeline,
    buffers: &Buffers,
    index_count: u32,
) {
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
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
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    render_pass.set_pipeline(render_pipeline);
    render_pass.set_bind_group(0, &buffers.bind_group, &[]);
    render_pass.set_vertex_buffer(0, buffers.vertex_buffer.slice(..));
    render_pass.set_index_buffer(buffers.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    render_pass.draw_indexed(0..index_count, 0, 0..1);
}
