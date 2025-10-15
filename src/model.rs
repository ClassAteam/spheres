use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct Position {
    #[format(R32G32B32_SFLOAT)]
    position: [f32; 3],
}

pub const TEST_TRIANGLE: [Position; 3] = [
    Position {
        position: [-0.5, 0.5, 0.0],
    },
    Position {
        position: [0.5, 0.5, 0.0],
    },
    Position {
        position: [0.0, -0.5, 0.0],
    },
];
