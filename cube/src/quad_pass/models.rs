use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

#[derive(BufferContents, Vertex, Debug, Clone, Copy)]
#[repr(C)]
pub struct QuadVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
    #[format(R32G32_SFLOAT)]
    pub uv: [f32; 2],
}

// 4 vertices forming a unit quad (will be transformed by orthographic matrix)
// Counter-clockwise winding order
pub const QUAD_VERTICES: [QuadVertex; 4] = [
    // Bottom-left
    QuadVertex {
        position: [-1.0, -1.0, 0.0],
        uv: [0.0, 1.0], // Bottom-left in texture space
    },
    // Bottom-right
    QuadVertex {
        position: [1.0, -1.0, 0.0],
        uv: [1.0, 1.0], // Bottom-right in texture space
    },
    // Top-right
    QuadVertex {
        position: [1.0, 1.0, 0.0],
        uv: [1.0, 0.0], // Top-right in texture space
    },
    // Top-left
    QuadVertex {
        position: [-1.0, 1.0, 0.0],
        uv: [0.0, 0.0], // Top-left in texture space
    },
];

// Two triangles forming the quad (counter-clockwise winding)
pub const QUAD_INDICES: [u16; 6] = [
    0, 1, 2, // First triangle (bottom-left, bottom-right, top-right)
    2, 3, 0, // Second triangle (top-right, top-left, bottom-left)
];
