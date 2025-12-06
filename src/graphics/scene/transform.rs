use glam::{Mat4, Quat, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    pub fn new(position: [f32; 3], rotation: [f32; 3], scale: [f32; 3]) -> Self {
        Self {
            position: Vec3::from_array(position),
            rotation: Quat::from_euler(glam::EulerRot::ZYX, rotation[2], rotation[1], rotation[0]),
            scale: Vec3::from_array(scale),
        }
    }

    pub fn to_matrix(&self) -> [[f32; 4]; 4] {
        let translation = Mat4::from_translation(self.position);
        let rotation = Mat4::from_quat(self.rotation);
        let scale = Mat4::from_scale(self.scale);

        (translation * rotation * scale).to_cols_array_2d()
    }
}
