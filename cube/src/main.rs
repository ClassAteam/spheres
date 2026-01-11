use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod context;
mod control;
mod models;
mod render;
mod shaders;

use app::App;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    _ = event_loop.run_app(&mut app);
}
