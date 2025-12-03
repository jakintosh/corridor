use glam::{Mat4, Vec3};
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct Camera {
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub target: Vec3,
    pub aspect_ratio: f32,
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Self {
        Self {
            distance: 12.0,
            yaw: PI / 4.0,
            pitch: PI / 8.0,
            target: Vec3::ZERO, // Anchor at origin (0, 0, 0)
            aspect_ratio,
        }
    }

    pub fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }

    pub fn handle_mouse_drag(&mut self, delta_x: f32, delta_y: f32) {
        // Horizontal orbit - no clamping, loops endlessly
        self.yaw += delta_x;

        // Vertical orbit - clamp between 5 and 85 degrees
        let min_pitch = 5.0_f32.to_radians();
        let max_pitch = 85.0_f32.to_radians();

        self.pitch = (self.pitch + delta_y).clamp(min_pitch, max_pitch);
    }

    pub fn handle_scroll(&mut self, delta: f32) {
        // Positive delta = scroll up = move closer
        // Negative delta = scroll down = pull back
        let zoom_speed = 0.5;
        self.distance = (self.distance - delta * zoom_speed).max(0.1);
    }

    pub fn view_projection_matrix(&self) -> [[f32; 4]; 4] {
        // Calculate camera position from spherical coordinates
        let x = self.target.x + self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.target.y + self.distance * self.pitch.sin();
        let z = self.target.z + self.distance * self.pitch.cos() * self.yaw.cos();
        let eye = Vec3::new(x, y, z);

        let view = Mat4::look_at_lh(eye, self.target, Vec3::Y);
        let proj = Mat4::perspective_lh(45.0_f32.to_radians(), self.aspect_ratio, 0.1, 100.0);

        (proj * view).to_cols_array_2d()
    }
}
