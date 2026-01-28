pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/quad_pass/shaders/quad_vert.glsl",
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/quad_pass/shaders/quad_frag.glsl",
    }
}
