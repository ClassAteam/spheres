// Circle generation parameters
pub struct CircleParams {
    pub radius: f32,
    pub segments: i32,
    pub circle_count: i32,
}

impl Default for CircleParams {
    fn default() -> Self {
        Self {
            radius: 0.3,
            segments: 128,
            circle_count: 2,
        }
    }
}

impl CircleParams {
    pub fn total_vertices(&self) -> u32 {
        (self.segments * self.circle_count) as u32
    }
}
