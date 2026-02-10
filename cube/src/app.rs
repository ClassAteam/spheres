use std::sync::Arc;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo};
use vulkano::format::Format;
use vulkano::image::ImageUsage;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::single_pass_renderpass;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;

use crate::config::AppConfig;
use crate::counter::FpsCounter;
use crate::cube_pass::CubePass;
use crate::debug_gui::DebugRenderer;
use crate::quad_pass::QuadPass;
use crate::render::RenderContext;
use crate::vulkan_context::VulkanBasicContext;

use vulkano::sync::GpuFuture;

pub struct App {
    pub config: AppConfig,
    pub basic_context: Arc<VulkanBasicContext>,
    pub rdx: Option<RenderContext>,
    render_pass: Option<Arc<RenderPass>>,
    fps_counter: FpsCounter,
    cube: Option<CubePass>,
    quad: Option<QuadPass>,
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
            render_pass: None,
            cube: None,
            quad: None,
            debug_renderer: None,
        }
    }

    pub fn toggle_debug_ui(&mut self) {
        self.config.debug_ui_enabled = !self.config.debug_ui_enabled;
    }

    fn create_render_pass(&mut self) {
        let id = self.rdx.as_ref().unwrap().id;
        let mut renderer = self.rdx.as_mut().unwrap().window_ctx.get_renderer_mut(id);
        let pass = single_pass_renderpass!(
            self.basic_context.bctx.device().clone(),
            attachments: {
                color: {
                    format: renderer.as_mut().unwrap().swapchain_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                depth_stencil: {
                    format: Format::D16_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {depth_stencil},
            },
        )
        .unwrap();

        renderer.as_mut().unwrap().add_additional_image_view(
            0,
            Format::D16_UNORM,
            ImageUsage::DEPTH_STENCIL_ATTACHMENT,
        );

        self.render_pass = Some(pass);
    }

    pub fn start_new_path(&mut self) {
        let acquire_future = self.rdx.as_mut().unwrap().acquire();

        let id = self.rdx.as_ref().unwrap().id;
        let mut renderer = self.rdx.as_mut().unwrap().window_ctx.get_renderer_mut(id);
        let descriptor_set_allocator = &self.basic_context.descriptor_set_allocator;
        let image = renderer.as_ref().unwrap().swapchain_image_view();
        let depth_view = &renderer.as_mut().unwrap().get_additional_image_view(0);

        let extent = renderer
            .as_ref()
            .unwrap()
            .swapchain_image_size()
            .map(|v| v as f32);
        let framebuffer = Framebuffer::new(
            self.render_pass.as_ref().unwrap().clone(),
            FramebufferCreateInfo {
                attachments: vec![image, depth_view.clone()],
                ..Default::default()
            },
        )
        .unwrap();

        let render_pass_begin_info = RenderPassBeginInfo {
            clear_values: vec![
                Some([0.0, 0.0, 0.0, 1.0].into()), // Color attachment
                Some(1.0.into()),                  // Depth attachment
            ],
            ..RenderPassBeginInfo::framebuffer(framebuffer)
        };

        let mut cb = AutoCommandBufferBuilder::primary(
            self.basic_context.cb_alloc.clone(),
            renderer.as_ref().unwrap().graphics_queue().queue_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        cb.begin_render_pass(render_pass_begin_info, Default::default())
            .unwrap();

        self.cube.as_mut().unwrap().draw_within_pass(
            renderer.as_ref().unwrap().aspect_ratio(),
            descriptor_set_allocator.clone(),
            extent,
            &mut cb,
        );

        self.quad.as_mut().unwrap().draw_within_pass(
            descriptor_set_allocator.clone(),
            extent,
            &mut cb,
        );

        cb.end_render_pass(Default::default()).unwrap();

        let command_buffer = cb.build().unwrap();

        let queue = &renderer.as_ref().unwrap().graphics_queue();
        let cube_frame_ready_future = acquire_future.then_execute(queue.clone(), command_buffer);

        let debug_pass_ready_future = self.debug_renderer.as_mut().unwrap().draw_ui(
            self.rdx.as_ref().unwrap(),
            &self.fps_counter,
            self.cube.as_ref().unwrap().get_transform_state(),
            cube_frame_ready_future.unwrap().boxed(),
            true,
        );

        self.rdx
            .as_mut()
            .unwrap()
            .present(Box::new(debug_pass_ready_future));
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.rdx = Some(RenderContext::new(
            event_loop,
            self.basic_context.bctx.clone(),
        ));

        self.create_render_pass();

        let id = self.rdx.as_ref().unwrap().id;

        self.cube = Some(CubePass::new(
            self.basic_context.bctx.as_ref(),
            self.render_pass.as_ref().unwrap().clone(),
        ));

        self.quad = Some(QuadPass::new(
            self.basic_context.bctx.as_ref(),
            self.render_pass.as_ref().unwrap().clone(),
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/glyph_atlas.ppm"),
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
                self.start_new_path();
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
                KeyCode::KeyL => self.cube.as_mut().unwrap().yaw_left(),
                KeyCode::KeyH => self.cube.as_mut().unwrap().yaw_right(),
                KeyCode::KeyJ => self.cube.as_mut().unwrap().pitch_up(),
                KeyCode::KeyK => self.cube.as_mut().unwrap().pitch_down(),

                KeyCode::KeyW => self.cube.as_mut().unwrap().move_right(),
                KeyCode::KeyS => self.cube.as_mut().unwrap().move_left(),
                KeyCode::KeyA => self.cube.as_mut().unwrap().move_down(),
                KeyCode::KeyD => self.cube.as_mut().unwrap().move_up(),
                KeyCode::KeyQ => self.cube.as_mut().unwrap().move_back(),
                KeyCode::KeyE => self.cube.as_mut().unwrap().move_forward(),
                KeyCode::KeyZ => self.cube.as_mut().unwrap().scale_up(),
                KeyCode::KeyX => self.cube.as_mut().unwrap().scale_down(),
                KeyCode::Digit1 => self.cube.as_mut().unwrap().camera_move_right(),
                KeyCode::Digit2 => self.cube.as_mut().unwrap().camera_move_left(),
                KeyCode::Digit3 => self.cube.as_mut().unwrap().camera_move_up(),
                KeyCode::Digit4 => self.cube.as_mut().unwrap().camera_move_down(),
                KeyCode::Digit5 => self.cube.as_mut().unwrap().camera_move_back(),
                KeyCode::Digit6 => self.cube.as_mut().unwrap().camera_move_forward(),
                KeyCode::Digit7 => self.cube.as_mut().unwrap().camera_look_right(),
                KeyCode::Digit8 => self.cube.as_mut().unwrap().camera_look_left(),
                KeyCode::Digit9 => self.cube.as_mut().unwrap().camera_look_up(),
                KeyCode::Digit0 => self.cube.as_mut().unwrap().camera_look_down(),
                KeyCode::Minus => self.cube.as_mut().unwrap().camera_look_back(),
                KeyCode::Equal => self.cube.as_mut().unwrap().camera_look_forward(),
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
