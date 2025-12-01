struct Uniforms {
    rotation: f32,
}
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
) -> VertexOutput {
    let angle = uniforms.rotation;
    let cos_a = cos(angle);
    let sin_a = sin(angle);

    // Rotation matrix around Y-axis
    let rotated_x = position.x * cos_a - position.z * sin_a;
    let rotated_z = position.x * sin_a + position.z * cos_a;

    // Perspective projection (simple)
    let projected = vec4<f32>(rotated_x, position.y, rotated_z + 2.0, 1.0);

    return VertexOutput(
        vec4<f32>(
            projected.x / (projected.z * 0.5),
            projected.y / (projected.z * 0.5),
            projected.z / 3.0,
            1.0
        ),
        color,
    );
}

@fragment
fn fs_main(@location(0) color: vec3<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(color, 1.0);
}
