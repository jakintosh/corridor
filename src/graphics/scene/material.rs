#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub color: [f32; 4], // RGBA
}

impl Material {
    pub fn new(color: [f32; 4]) -> Self {
        Self { color }
    }

    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Self {
            color: [r, g, b, 1.0],
        }
    }
}
