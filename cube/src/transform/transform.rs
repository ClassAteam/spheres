use glam::{Mat4, Vec3, Vec4};

use super::camera::Camera;
use super::model::ModelTransform;
use super::projection::ProjectionParams;

#[derive(Debug)]
pub struct TransformState {
    pub model: ModelTransform,
    pub camera: Camera,
    pub projection: ProjectionParams,
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

impl TransformState {
    pub fn rotate_model(&mut self, delta: Vec3) {
        self.model.rotation += delta;
    }
    pub fn translate_model(&mut self, delta: Vec3) {
        self.model.translation += delta;
    }
    pub fn scale_model(&mut self, delta: Vec3) {
        self.model.scale += delta;
    }
    pub fn camera_position(&mut self, delta: Vec3) {
        self.camera.position += delta;
    }
    pub fn camera_target(&mut self, delta: Vec3) {
        self.camera.target += delta;
    }
    pub fn camera_up(&mut self, delta: Vec3) {
        self.camera.up += delta;
    }

    pub fn transform_vertex(&self, position: [f32; 3], aspect_ratio: f32) -> TransformedVertex {
        let mvp = self.compute_mvp(aspect_ratio);
        let pos = Vec4::new(position[0], position[1], position[2], 1.0);
        let clip_space = mvp * pos;

        // Perspective divide to get NDC (Normalized Device Coordinates)
        let ndc = if clip_space.w != 0.0 {
            Vec3::new(
                clip_space.x / clip_space.w,
                clip_space.y / clip_space.w,
                clip_space.z / clip_space.w,
            )
        } else {
            Vec3::ZERO
        };

        TransformedVertex {
            clip_space: [clip_space.x, clip_space.y, clip_space.z, clip_space.w],
            ndc: [ndc.x, ndc.y, ndc.z],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TransformedVertex {
    pub clip_space: [f32; 4], // x, y, z, w before perspective divide
    pub ndc: [f32; 3],        // Normalized Device Coordinates (after perspective divide)
}
