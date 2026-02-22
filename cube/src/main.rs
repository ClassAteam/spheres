use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod counter;
mod cube_pass;
mod overlay_renderer;
mod proc_cube_pass;
mod render;
mod renderer_pool;
mod text_renderer;
mod transform;
mod vulkan_context;
mod within_pass_renderer;

use app::App;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    _ = event_loop.run_app(&mut app);
}
