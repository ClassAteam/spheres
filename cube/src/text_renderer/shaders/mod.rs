pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/text_renderer/shaders/quad_vert.glsl",
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/text_renderer/shaders/quad_frag.glsl",
    }
}
