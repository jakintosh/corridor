use glam::Vec3;

use super::LightingUniform;

#[derive(Debug, Clone)]
pub struct LightingSettings {
    pub sun_direction: Vec3,
    pub sun_color: Vec3,
    pub sun_intensity: f32,
    pub horizon_color: Vec3,
    pub ambient_height: f32,
}

impl Default for LightingSettings {
    fn default() -> Self {
        Self::from(&LightingControls::default())
    }
}

impl LightingSettings {
    pub fn to_uniform(&self) -> LightingUniform {
        let dir = self.sun_direction.normalize_or_zero();

        LightingUniform {
            sun_direction: [dir.x, dir.y, dir.z, self.sun_intensity],
            sun_color: [self.sun_color.x, self.sun_color.y, self.sun_color.z, 0.0],
            horizon_color: [
                self.horizon_color.x,
                self.horizon_color.y,
                self.horizon_color.z,
                self.ambient_height.max(0.0001),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct LightingControls {
    pub sun_direction: Vec3,
    pub sun_color: Vec3,
    pub sun_intensity: f32,
    pub horizon_color: Vec3,
    pub ambient_height: f32,
}

impl Default for LightingControls {
    fn default() -> Self {
        Self {
            sun_direction: Vec3::new(-0.4, -1.0, -0.3),
            sun_color: Vec3::new(1.0, 0.7, 0.7),
            sun_intensity: 1.25,
            horizon_color: Vec3::new(0.15, 0.2, 0.55),
            ambient_height: 6.0,
        }
    }
}

impl From<&LightingControls> for LightingSettings {
    fn from(value: &LightingControls) -> Self {
        Self {
            sun_direction: value.sun_direction.normalize_or_zero(),
            sun_color: value.sun_color,
            sun_intensity: value.sun_intensity,
            horizon_color: value.horizon_color,
            ambient_height: value.ambient_height,
        }
    }
}

impl From<&LightingSettings> for LightingControls {
    fn from(value: &LightingSettings) -> Self {
        Self {
            sun_direction: value.sun_direction,
            sun_color: value.sun_color,
            sun_intensity: value.sun_intensity,
            horizon_color: value.horizon_color,
            ambient_height: value.ambient_height,
        }
    }
}
