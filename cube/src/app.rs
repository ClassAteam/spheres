use std::sync::Arc;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;

use crate::counter::FpsCounter;
use crate::cube_pass::CubePass;
use crate::render::RenderContext;
use crate::vulkan_context::VulkanBasicContext;

use vulkano::sync::GpuFuture;

pub struct App {
    pub basic_context: Arc<VulkanBasicContext>,
    pub rdx: Option<RenderContext>,
    fps_counter: FpsCounter,
    cube: Option<CubePass>,
}

impl App {
    pub fn new() -> Self {
        let context = VulkanBasicContext::new();
        App {
            basic_context: Arc::new(context),
            fps_counter: FpsCounter::new(),
            rdx: None,
            cube: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.rdx = Some(RenderContext::new(
            event_loop,
            self.basic_context.bctx.clone(),
        ));

        let id = self.rdx.as_ref().unwrap().id;
        self.cube = Some(CubePass::new(
            self.rdx
                .as_mut()
                .unwrap()
                .window_ctx
                .get_renderer_mut(id)
                .unwrap(),
            self.basic_context.bctx.as_ref(),
        ));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.fps_counter.update();

                let acquired_future = self.rdx.as_mut().unwrap().acquire();

                let mut cb = AutoCommandBufferBuilder::primary(
                    self.basic_context.cb_alloc.clone(),
                    self.rdx
                        .as_ref()
                        .unwrap()
                        .window_ctx
                        .get_primary_renderer()
                        .as_ref()
                        .unwrap()
                        .graphics_queue()
                        .queue_index(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .unwrap();

                self.cube.as_mut().unwrap().update_uniform_and_create_pass(
                    self.basic_context.descriptor_set_allocator.clone(),
                    self.rdx.as_mut().unwrap(),
                    &mut cb,
                );

                let command_buffer = cb.build().unwrap();

                let queue = self
                    .rdx
                    .as_ref()
                    .unwrap()
                    .window_ctx
                    .get_primary_renderer()
                    .unwrap()
                    .graphics_queue()
                    .clone();

                let after_cube_future =
                    acquired_future.then_execute(queue, command_buffer).unwrap();

                self.rdx
                    .as_mut()
                    .unwrap()
                    .present(Box::new(after_cube_future));
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
                KeyCode::KeyL => self.cube.as_mut().unwrap().rotate_cube_y_left(),
                KeyCode::KeyH => self.cube.as_mut().unwrap().rotate_cube_y_right(),
                KeyCode::KeyJ => self.cube.as_mut().unwrap().rotate_cube_x_up(),
                KeyCode::KeyK => self.cube.as_mut().unwrap().rotate_cube_x_down(),
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
