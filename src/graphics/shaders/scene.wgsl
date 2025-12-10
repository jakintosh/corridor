struct Camera {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0) var<uniform> camera: Camera;

struct Lighting {
    sun_direction: vec4<f32>, // xyz = direction, w = intensity
    sun_color: vec4<f32>,
    horizon_color: vec4<f32>, // xyz = horizon color, w = ambient height
}
@group(1) @binding(0) var<uniform> lighting: Lighting;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) instance_matrix_0: vec4<f32>,
    @location(3) instance_matrix_1: vec4<f32>,
    @location(4) instance_matrix_2: vec4<f32>,
    @location(5) instance_matrix_3: vec4<f32>,
    @location(6) material_color: vec4<f32>,
    @location(7) state_flags: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) material_color: vec4<f32>,
    @location(3) state_flags: u32,
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
    let world_normal = normalize((model * vec4<f32>(in.normal, 0.0)).xyz);
    let clip_pos = camera.view_proj * world_pos;

    return VertexOutput(clip_pos, world_pos.xyz, world_normal, in.material_color, in.state_flags);
}

const STATE_HOVERED: u32 = 0x01u;
const STATE_DRAGGING: u32 = 0x02u;

@fragment
fn fs_main(
    @location(0) world_pos: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) material_color: vec4<f32>,
    @location(3) state_flags: u32,
) -> @location(0) vec4<f32> {
    let n = normalize(world_normal);
    let light_dir = normalize(lighting.sun_direction.xyz);
    let ndotl = max(dot(n, -light_dir), 0.0);

    let sun_light = lighting.sun_color.xyz * (lighting.sun_direction.w * ndotl);

    // Height-based ambient blend between horizon and sun color
    let ambient_factor = clamp(world_pos.y / lighting.horizon_color.w, 0.0, 1.0);
    let ambient_color = mix(lighting.horizon_color.xyz, lighting.sun_color.xyz, ambient_factor);

    var base_color = material_color.rgb;

    // Apply visual feedback based on state
    if ((state_flags & STATE_DRAGGING) != 0u) {
        // Dragging: brighten significantly with slight green tint
        base_color = base_color * 1.4 + vec3<f32>(0.0, 0.15, 0.0);
    } else if ((state_flags & STATE_HOVERED) != 0u) {
        // Hovered: slight brighten
        base_color = base_color * 1.2;
    }

    let lit_color = (sun_light + ambient_color) * base_color;
    return vec4<f32>(lit_color, material_color.a);
}
