use glam::Vec3;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;

use crate::counter::FpsCounter;
use crate::debug_gui::DebugRenderer;
use crate::render::RenderContext;
use crate::transform::TransformState;
use crate::vulkan_context::VulkanBasicContext;

pub struct App {
    pub basic_context: Arc<VulkanBasicContext>,
    pub rdx: Option<RenderContext>,
    pub dbg_render: Option<DebugRenderer>,
    transform: TransformState,
    fps_counter: FpsCounter,
}

impl App {
    pub fn new() -> Self {
        let context = VulkanBasicContext::new();
        App {
            basic_context: Arc::new(context),
            transform: TransformState::new(),
            rdx: None,
            fps_counter: FpsCounter::new(),
            dbg_render: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.rdx = Some(RenderContext::new(
            event_loop,
            self.basic_context.bctx.clone(),
        ));

        self.dbg_render = Some(DebugRenderer::new(
            event_loop,
            self.rdx
                .as_ref()
                .unwrap()
                .window_ctx
                .get_renderer(self.rdx.as_ref().unwrap().id)
                .unwrap(),
        ))
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(_) = &mut self.rdx {
            let consumed = self.dbg_render.as_mut().unwrap().update(&event);

            if consumed {
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.fps_counter.update();
                let acquired_future = self.rdx.as_mut().unwrap().acquire();
                let after_cube_future = self.rdx.as_mut().unwrap().draw(
                    acquired_future,
                    self.basic_context.cb_alloc.clone(),
                    self.basic_context.descriptor_set_allocator.clone(),
                    &self.transform,
                );

                self.rdx.as_mut().unwrap().present(after_cube_future);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match key_code {
                KeyCode::KeyL => self.transform.rotate_model(Vec3::new(0.0, -0.01, 0.0)),
                KeyCode::KeyH => self.transform.rotate_model(Vec3::new(0.0, 0.01, 0.0)),
                KeyCode::KeyJ => self.transform.rotate_model(Vec3::new(-0.01, 0.0, 0.0)),
                KeyCode::KeyK => self.transform.rotate_model(Vec3::new(0.01, 0.0, 0.0)),
                KeyCode::Escape => event_loop.exit(),
                _ => (),
            },
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(rdx) = &self.rdx {
            rdx.window_ctx
                .get_window(rdx.id)
                .expect("Failed to get window")
                .request_redraw();
        }
    }
}
