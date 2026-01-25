use std::sync::Arc;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;

use crate::config::AppConfig;
use crate::counter::FpsCounter;
use crate::cube_pass::CubePass;
use crate::debug_gui::DebugRenderer;
use crate::render::RenderContext;
use crate::vulkan_context::VulkanBasicContext;

use vulkano::sync::GpuFuture;

pub struct App {
    pub config: AppConfig,
    pub basic_context: Arc<VulkanBasicContext>,
    pub rdx: Option<RenderContext>,
    fps_counter: FpsCounter,
    cube: Option<CubePass>,
    debug_renderer: Option<DebugRenderer>,
}

impl App {
    pub fn new() -> Self {
        let context = VulkanBasicContext::new();
        App {
            config: Default::default(),
            basic_context: Arc::new(context),
            fps_counter: FpsCounter::new(),
            rdx: None,
            cube: None,
            debug_renderer: None,
        }
    }

    pub fn toggle_debug_ui(&mut self) {
        self.config.debug_ui_enabled = !self.config.debug_ui_enabled;
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

        self.debug_renderer = if self.config.debug_ui_enabled {
            Some(DebugRenderer::new(
                event_loop,
                self.rdx
                    .as_mut()
                    .unwrap()
                    .window_ctx
                    .get_renderer_mut(id)
                    .unwrap(),
            ))
        } else {
            None
        }
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

                let mut final_future: Box<dyn GpuFuture> =
                    Box::new(acquired_future.then_execute(queue, command_buffer).unwrap());

                if let Some(debug_renderer) = self.debug_renderer.as_mut() {
                    final_future = debug_renderer.draw_ui(
                        self.rdx.as_mut().unwrap(),
                        &self.fps_counter,
                        self.cube.as_ref().unwrap().get_transform_state(),
                        final_future,
                        self.config.debug_ui_enabled, // Pass the visibility flag
                    );
                }

                self.rdx.as_mut().unwrap().present(Box::new(final_future));
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

                KeyCode::KeyW => self.cube.as_mut().unwrap().translate_cube_x_right(),
                KeyCode::KeyS => self.cube.as_mut().unwrap().translate_cube_x_left(),
                KeyCode::KeyA => self.cube.as_mut().unwrap().translate_cube_y_down(),
                KeyCode::KeyD => self.cube.as_mut().unwrap().translate_cube_y_up(),
                KeyCode::KeyQ => self.cube.as_mut().unwrap().translate_cube_z_down(),
                KeyCode::KeyE => self.cube.as_mut().unwrap().translate_cube_z_up(),
                KeyCode::KeyZ => self.cube.as_mut().unwrap().scale_cube_up(),
                KeyCode::KeyX => self.cube.as_mut().unwrap().scale_cube_down(),
                KeyCode::Digit1 => self.cube.as_mut().unwrap().camera_position_x_up(),
                KeyCode::Digit2 => self.cube.as_mut().unwrap().camera_position_x_down(),
                KeyCode::Digit3 => self.cube.as_mut().unwrap().camera_position_y_up(),
                KeyCode::Digit4 => self.cube.as_mut().unwrap().camera_position_y_down(),
                KeyCode::Digit5 => self.cube.as_mut().unwrap().camera_position_z_up(),
                KeyCode::Digit6 => self.cube.as_mut().unwrap().camera_position_z_down(),
                KeyCode::Digit7 => self.cube.as_mut().unwrap().camera_target_x_up(),
                KeyCode::Digit8 => self.cube.as_mut().unwrap().camera_target_x_down(),
                KeyCode::Digit9 => self.cube.as_mut().unwrap().camera_target_y_up(),
                KeyCode::Digit0 => self.cube.as_mut().unwrap().camera_target_y_down(),
                KeyCode::Minus => self.cube.as_mut().unwrap().camera_target_z_up(),
                KeyCode::Equal => self.cube.as_mut().unwrap().camera_target_z_down(),
                KeyCode::ArrowUp => self.cube.as_mut().unwrap().camera_up_x_up(),
                KeyCode::ArrowDown => self.cube.as_mut().unwrap().camera_up_x_down(),
                KeyCode::ArrowLeft => self.cube.as_mut().unwrap().camera_up_y_up(),
                KeyCode::ArrowRight => self.cube.as_mut().unwrap().camera_up_y_down(),
                KeyCode::PageUp => self.cube.as_mut().unwrap().camera_up_z_up(),
                KeyCode::PageDown => self.cube.as_mut().unwrap().camera_up_z_down(),
                KeyCode::F1 => self.toggle_debug_ui(),
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
