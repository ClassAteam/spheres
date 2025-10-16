pub struct Control {
    pub rotation_angle: f32,
}

impl Control {
    pub fn new() -> Self {
        Control {
            rotation_angle: 0.0,
        }
    }

    pub fn rotate_up(&mut self) {
        self.rotation_angle += 0.01;
    }

    pub fn rotate_down(&mut self) {
        self.rotation_angle -= 0.01;
    }
}
