use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct Position {
    #[format(R32G32B32_SFLOAT)]
    position: [f32; 3],
}

pub const POSITIONS: [Position; 3] = [
    Position {
        position: [0.0, 0.0, 0.0],
    },
    Position {
        position: [-1.0, -1.0, -1.0],
    },
    Position {
        position: [1.0, -1.0, 1.0],
    },
];
