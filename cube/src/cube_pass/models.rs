use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

#[derive(BufferContents, Vertex, Debug, Clone, Copy)]
#[repr(C)]
pub struct Position {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub color: [f32; 3],
}

// 24 vertices (4 per face, 6 faces) - each face has its own vertices with a unique color
pub const POSITIONS: [Position; 24] = [
    // Back face (indices 0-3) - Red
    Position {
        position: [-1.0, 1.0, -1.0],
        color: [1.0, 0.0, 0.0],
    },
    Position {
        position: [1.0, 1.0, -1.0],
        color: [1.0, 0.0, 0.0],
    },
    Position {
        position: [1.0, -1.0, -1.0],
        color: [1.0, 0.0, 0.0],
    },
    Position {
        position: [-1.0, -1.0, -1.0],
        color: [1.0, 0.0, 0.0],
    },
    // Front face (indices 4-7) - Green
    Position {
        position: [-1.0, 1.0, 1.0],
        color: [0.0, 1.0, 0.0],
    },
    Position {
        position: [1.0, 1.0, 1.0],
        color: [0.0, 1.0, 0.0],
    },
    Position {
        position: [1.0, -1.0, 1.0],
        color: [0.0, 1.0, 0.0],
    },
    Position {
        position: [-1.0, -1.0, 1.0],
        color: [0.0, 1.0, 0.0],
    },
    // Left face (indices 8-11) - Blue
    Position {
        position: [-1.0, 1.0, -1.0],
        color: [0.0, 0.0, 1.0],
    },
    Position {
        position: [-1.0, -1.0, -1.0],
        color: [0.0, 0.0, 1.0],
    },
    Position {
        position: [-1.0, -1.0, 1.0],
        color: [0.0, 0.0, 1.0],
    },
    Position {
        position: [-1.0, 1.0, 1.0],
        color: [0.0, 0.0, 1.0],
    },
    // Right face (indices 12-15) - Yellow
    Position {
        position: [1.0, 1.0, -1.0],
        color: [1.0, 1.0, 0.0],
    },
    Position {
        position: [1.0, 1.0, 1.0],
        color: [1.0, 1.0, 0.0],
    },
    Position {
        position: [1.0, -1.0, 1.0],
        color: [1.0, 1.0, 0.0],
    },
    Position {
        position: [1.0, -1.0, -1.0],
        color: [1.0, 1.0, 0.0],
    },
    // Bottom face (indices 16-19) - Magenta
    Position {
        position: [-1.0, 1.0, -1.0],
        color: [1.0, 0.0, 1.0],
    },
    Position {
        position: [-1.0, 1.0, 1.0],
        color: [1.0, 0.0, 1.0],
    },
    Position {
        position: [1.0, 1.0, 1.0],
        color: [1.0, 0.0, 1.0],
    },
    Position {
        position: [1.0, 1.0, -1.0],
        color: [1.0, 0.0, 1.0],
    },
    // Top face (indices 20-23) - Cyan
    Position {
        position: [-1.0, -1.0, -1.0],
        color: [0.0, 1.0, 1.0],
    },
    Position {
        position: [1.0, -1.0, -1.0],
        color: [0.0, 1.0, 1.0],
    },
    Position {
        position: [1.0, -1.0, 1.0],
        color: [0.0, 1.0, 1.0],
    },
    Position {
        position: [-1.0, -1.0, 1.0],
        color: [0.0, 1.0, 1.0],
    },
];

pub const INDICES: [u16; 36] = [
    // Back face
    0, 1, 2, 2, 3, 0, // Front face
    4, 6, 5, 4, 7, 6, // Left face
    8, 9, 10, 10, 11, 8, // Right face
    12, 13, 14, 14, 15, 12, // Bottom face
    16, 17, 18, 18, 19, 16, // Top face
    20, 21, 22, 22, 23, 20,
];
