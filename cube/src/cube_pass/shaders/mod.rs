pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/cube_pass/shaders/vert.glsl",
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/cube_pass/shaders/frag.glsl",
    }
}
