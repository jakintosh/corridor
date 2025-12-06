struct Camera {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0) var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) instance_matrix_0: vec4<f32>,
    @location(3) instance_matrix_1: vec4<f32>,
    @location(4) instance_matrix_2: vec4<f32>,
    @location(5) instance_matrix_3: vec4<f32>,
    @location(6) material_color: vec4<f32>,
    @location(7) node_id: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) node_id: u32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let model = mat4x4<f32>(
        in.instance_matrix_0,
        in.instance_matrix_1,
        in.instance_matrix_2,
        in.instance_matrix_3,
    );

    let world_pos = model * vec4<f32>(in.position, 1.0);
    let clip_pos = camera.view_proj * world_pos;

    return VertexOutput(clip_pos, in.node_id);
}

@fragment
fn fs_main(@location(0) node_id: u32) -> @location(0) u32 {
    return node_id;
}

@fragment
fn fs_debug_main(@location(0) node_id: u32) -> @location(0) vec4<f32> {
    let r: f32 = f32(node_id & 0xFFu) / 255.0;
    let g: f32 = f32((node_id >> 8u) & 0xFFu) / 255.0;
    let b: f32 = f32((node_id >> 16u) & 0xFFu) / 255.0;
    return vec4<f32>(r, g, b, 1.0);
}
