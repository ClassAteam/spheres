use glam::{Mat4, Vec3};

#[derive(Debug)]
pub struct ModelTransform {
    pub rotation: Vec3, // x, y, z rotations in radians
    pub translation: Vec3,
    pub scale: Vec3,
}
impl ModelTransform {
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_translation(self.translation)
            * Mat4::from_rotation_y(self.rotation.y)
            * Mat4::from_rotation_x(self.rotation.x)
            * Mat4::from_rotation_z(self.rotation.z)
            * Mat4::from_scale(self.scale)
    }
}

impl Default for ModelTransform {
    fn default() -> Self {
        Self {
            rotation: Vec3::new(0.0, 0.0, 0.0),
            translation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(0.5, 0.5, 0.5),
        }
    }
}
