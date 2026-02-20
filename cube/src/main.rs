use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod config;
mod counter;
mod cube_pass;
mod debug_gui;
mod proc_cube_pass;
mod render;
mod text_renderer;
mod transform;
mod vulkan_context;

use app::App;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    _ = event_loop.run_app(&mut app);
}
