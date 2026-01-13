use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod control;
mod counter;
mod models;
mod render;
mod shaders;
mod vulkan_context;

use app::App;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    _ = event_loop.run_app(&mut app);
}
