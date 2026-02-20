use glam::Mat4;

#[derive(Debug)]
pub struct ProjectionParams {
    pub fov: f32, // field of view in radians
    pub near: f32,
    pub far: f32,
}

impl ProjectionParams {
    pub fn projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov, aspect_ratio, self.near, self.far)
    }
}

impl Default for ProjectionParams {
    fn default() -> Self {
        Self {
            fov: 90.0_f32.to_radians(),
            near: 0.1,
            far: 100.0,
        }
    }
}
