use crate::model::SphereParams;

pub struct Control {
    pub rotation_x: f32,
    pub rotation_y: f32, 
    pub rotation_z: f32,
    pub sphere_params: SphereParams,
}

impl Control {
    pub fn new() -> Self {
        Control {
            rotation_x: 0.0,
            rotation_y: 0.0,
            rotation_z: 0.0,
            sphere_params: SphereParams::default(),
        }
    }

    pub fn rotate_up(&mut self) {
        self.rotation_x += 0.02;
    }

    pub fn rotate_down(&mut self) {
        self.rotation_x -= 0.02;
    }
    
    pub fn rotate_left(&mut self) {
        self.rotation_y -= 0.02;
    }
    
    pub fn rotate_right(&mut self) {
        self.rotation_y += 0.02;
    }
}
