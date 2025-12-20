// Sphere generation parameters
pub struct SphereParams {
    pub radius: f32,
    pub segments: i32, // Longitude divisions
    pub rings: i32,    // Latitude divisions
}

impl Default for SphereParams {
    fn default() -> Self {
        Self {
            radius: 0.8,
            segments: 32,
            rings: 16,
        }
    }
}

// impl SphereParams {
//     pub fn total_vertices(&self) -> u32 {
//         (self.segments * self.rings) as u32
//     }
// }
