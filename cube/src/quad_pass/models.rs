use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

#[derive(BufferContents, Vertex, Debug, Clone, Copy)]
#[repr(C)]
pub struct QuadVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
    #[format(R32G32_SFLOAT)]
    pub uv: [f32; 2],
}
