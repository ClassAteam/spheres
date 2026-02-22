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

use crate::counter::FpsCounter;
use crate::cube_pass::CubePass;
use crate::proc_cube_pass::ProcCubePass;
use crate::render::RenderContext;
use crate::renderer_pool::RendererPool;
use crate::vulkan_context::VulkanBasicContext;

use vulkano::sync::GpuFuture;

pub struct App {
    pub basic_context: Arc<VulkanBasicContext>,
    pub rdx: Option<RenderContext>,
    render_pass: Option<Arc<RenderPass>>,
    fps_counter: FpsCounter,
    renderer_pool: RendererPool,
}

impl App {
    pub fn new() -> Self {
        let context = VulkanBasicContext::new();
        App {
            basic_context: Arc::new(context),
            fps_counter: FpsCounter::new(),
            rdx: None,
            render_pass: None,
            renderer_pool: RendererPool::new(),
        }
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

        self.renderer_pool.active().draw_within_pass(
            descriptor_set_allocator.clone(),
            extent,
            &mut cb,
        );

        cb.end_render_pass(Default::default()).unwrap();

        let command_buffer = cb.build().unwrap();

        let queue = &renderer.as_ref().unwrap().graphics_queue();
        let cube_frame_ready_future = acquire_future
            .then_execute(queue.clone(), command_buffer)
            .unwrap();

        self.rdx
            .as_mut()
            .unwrap()
            .present(Box::new(cube_frame_ready_future));
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.rdx = Some(RenderContext::new(
            event_loop,
            self.basic_context.bctx.clone(),
        ));

        self.create_render_pass();

        self.renderer_pool.add_renderer(Box::new(CubePass::new(
            self.basic_context.bctx.as_ref(),
            self.render_pass.as_ref().unwrap().clone(),
        )));
        self.renderer_pool.add_renderer(Box::new(ProcCubePass::new(
            self.basic_context.bctx.as_ref(),
            self.render_pass.as_ref().unwrap().clone(),
        )));
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

            ref e => {
                // Delegate event to active renderer
                let handled = if !self.renderer_pool.is_empty() {
                    self.renderer_pool.active().handle_window_event(e)
                } else {
                    false
                };

                // If not handled by renderer, check app-level events
                if !handled {
                    if let WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } = e
                    {
                        event_loop.exit();
                    }
                }
            }
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
