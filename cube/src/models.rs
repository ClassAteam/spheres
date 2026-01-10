use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct Position {
    #[format(R32G32B32_SFLOAT)]
    position: [f32; 3],
}

// 8 unique vertex positions (cube corners)
pub const POSITIONS: [Position; 8] = [
    Position {
        position: [-1.0, -1.0, -1.0],
    }, // 0: back-bottom-left
    Position {
        position: [1.0, -1.0, -1.0],
    }, // 1: back-bottom-right
    Position {
        position: [1.0, 1.0, -1.0],
    }, // 2: back-top-right
    Position {
        position: [-1.0, 1.0, -1.0],
    }, // 3: back-top-left
    Position {
        position: [-1.0, -1.0, 1.0],
    }, // 4: front-bottom-left
    Position {
        position: [1.0, -1.0, 1.0],
    }, // 5: front-bottom-right
    Position {
        position: [1.0, 1.0, 1.0],
    }, // 6: front-top-right
    Position {
        position: [-1.0, 1.0, 1.0],
    }, // 7: front-top-left
];

// 36 indices forming 12 triangles (6 faces × 2 triangles per face)
// Counter-clockwise winding when viewed from outside
pub const INDICES: [u16; 36] = [
    // Back face (z = -1)
    0, 1, 2, 2, 3, 0, // Front face (z = 1)
    4, 6, 5, 4, 7, 6, // Left face (x = -1)
    4, 0, 3, 3, 7, 4, // Right face (x = 1)
    1, 5, 6, 6, 2, 1, // Bottom face (y = -1)
    4, 5, 1, 1, 0, 4, // Top face (y = 1)
    3, 2, 6, 6, 7, 3,
];

// **Visualization of vertex positions:**
// ```
//         7 ----------- 6
//        /|            /|
//       / |           / |
//      /  |          /  |
//     4 ----------- 5   |
//     |   |         |   |
//     |   3 --------|-- 2
//     |  /          |  /
//     | /           | /
//     |/            |/
//     0 ----------- 1

// Back face (z=-1): 0,1,2,3
// Front face (z=1): 4,5,6,7
