use glam::{Mat4, Vec3};

pub struct TransformState {
    pub model: ModelTransform,
    pub camera: Camera,
    pub projection: ProjectionParams,
}

pub struct ModelTransform {
    pub rotation: Vec3, // x, y, z rotations in radians
    pub translation: Vec3,
    pub scale: Vec3,
}

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
}

pub struct ProjectionParams {
    pub fov: f32, // field of view in radians
    pub near: f32,
    pub far: f32,
}

impl TransformState {
    pub fn new() -> Self {
        Self {
            model: ModelTransform::default(),
            camera: Camera::default(),
            projection: ProjectionParams::default(),
        }
    }

    pub fn compute_mvp(&self, aspect_ratio: f32) -> Mat4 {
        let model = self.model.matrix();
        let view = self.camera.view_matrix();
        let projection = self.projection.projection_matrix(aspect_ratio);
        projection * view * model
    }
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
impl Camera {
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }
}

impl ProjectionParams {
    pub fn projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov, aspect_ratio, self.near, self.far)
    }
}

// Default values matching your current hardcoded setup
impl Default for ModelTransform {
    fn default() -> Self {
        Self {
            rotation: Vec3::new(0.5, 0.3, 0.0),
            translation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(3.5, 2.5, 4.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
        }
    }
}

impl Default for ProjectionParams {
    fn default() -> Self {
        Self {
            fov: 45.0_f32.to_radians(),
            near: 0.1,
            far: 100.0,
        }
    }
}

impl TransformState {
    pub fn rotate_model(&mut self, delta: Vec3) {
        self.model.rotation += delta;
    }

    // pub fn move_camera(&mut self, delta: Vec3) {
    //     self.camera.position += delta;
    // }
}
