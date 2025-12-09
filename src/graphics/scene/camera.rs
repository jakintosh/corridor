use glam::{Mat4, Vec3, Vec4};
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

    /// Convert screen coordinates to a world-space ray
    /// Returns (ray_origin, ray_direction)
    pub fn screen_to_world_ray(
        &self,
        screen_x: f32,
        screen_y: f32,
        screen_width: f32,
        screen_height: f32,
    ) -> (Vec3, Vec3) {
        // Convert screen coords to NDC (-1 to 1, Y flipped for screen space)
        let ndc_x = (2.0 * screen_x / screen_width) - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_y / screen_height);

        // Get view-projection matrix and invert it
        let view_proj = Mat4::from_cols_array_2d(&self.view_projection_matrix());
        let inv_view_proj = view_proj.inverse();

        // Unproject near and far points in NDC space to world space
        let near_ndc = Vec4::new(ndc_x, ndc_y, 0.0, 1.0);
        let far_ndc = Vec4::new(ndc_x, ndc_y, 1.0, 1.0);

        let near_world = inv_view_proj * near_ndc;
        let far_world = inv_view_proj * far_ndc;

        // Perspective divide
        let near_world = near_world.truncate() / near_world.w;
        let far_world = far_world.truncate() / far_world.w;

        // Ray from near to far
        let direction = (far_world - near_world).normalize();

        (near_world, direction)
    }

    /// Find intersection of a ray with a plane
    /// Returns None if ray is parallel to plane or intersection is behind ray origin
    pub fn ray_plane_intersection(
        ray_origin: Vec3,
        ray_dir: Vec3,
        plane_point: Vec3,
        plane_normal: Vec3,
    ) -> Option<Vec3> {
        let denom = plane_normal.dot(ray_dir);

        // Check if ray is parallel to plane
        if denom.abs() < 1e-6 {
            return None;
        }

        // Calculate t parameter
        let t = (plane_point - ray_origin).dot(plane_normal) / denom;

        // Check if intersection is behind ray origin
        if t < 0.0 {
            return None;
        }

        Some(ray_origin + ray_dir * t)
    }
}
