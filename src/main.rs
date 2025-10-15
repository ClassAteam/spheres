use std::error::Error;
use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

mod model;

mod render_context;
mod window_context;
use render_context::RenderContext;

struct App {
    rcx: Option<RenderContext>,
}

impl App {
    fn new(event_loop: &EventLoop<()>) -> Self {
        App {
            rcx: Some(RenderContext::new(event_loop)),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None))),
                )
                .unwrap(),
        );

        self.rcx.as_mut().unwrap().resumed(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(ref mut render_context) = self.rcx {
            match event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }

                WindowEvent::Resized(_) => {
                    render_context.window_invalidated();
                }

                WindowEvent::RedrawRequested => {
                    render_context.draw();
                }
                
                WindowEvent::KeyboardInput { 
                    event: KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                    ..
                } => {
                    event_loop.exit();
                }
                _ => {}
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.rcx.as_mut().unwrap().request_redraw();
    }
}

fn main() -> Result<(), impl Error> {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(&event_loop);
    event_loop.run_app(&mut app)
}
