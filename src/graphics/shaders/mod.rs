pub fn cube_shader_source() -> String {
    include_str!("cube.wgsl").to_string()
}

pub fn picking_shader_source() -> &'static str {
    include_str!("picking.wgsl")
}
