use crate::model::CircleParams;

pub struct Control {
    pub rotation_angle: f32,
    pub circle_params: CircleParams,
}

impl Control {
    pub fn new() -> Self {
        Control {
            rotation_angle: 0.0,
            circle_params: CircleParams {
                radius: (0.3),
                segments: (128),
                circle_count: (2),
            },
        }
    }

    pub fn rotate_up(&mut self) {
        self.rotation_angle += 0.01;
    }

    pub fn rotate_down(&mut self) {
        self.rotation_angle -= 0.01;
    }
}
