struct Camera {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0) var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) instance_matrix_0: vec4<f32>,
    @location(2) instance_matrix_1: vec4<f32>,
    @location(3) instance_matrix_2: vec4<f32>,
    @location(4) instance_matrix_3: vec4<f32>,
    @location(5) material_color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
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

    return VertexOutput(clip_pos, in.material_color);
}

@fragment
fn fs_main(@location(0) color: vec4<f32>) -> @location(0) vec4<f32> {
    return color;
}
