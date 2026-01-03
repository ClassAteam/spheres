use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo,
    SubpassContents,
};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::subpass::PipelineSubpassType;
use vulkano::pipeline::graphics::vertex_input::VertexInputState;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::layout::PipelineLayoutCreateInfo;
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::single_pass_renderpass;
use vulkano::sync::GpuFuture;
use vulkano_util::window::{WindowDescriptor, WindowMode};
use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
    window::VulkanoWindows,
};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;

struct VulkanBasicContext {
    bctx: Arc<VulkanoContext>,
    cb_alloc: Arc<StandardCommandBufferAllocator>,
}

struct App {
    basic_context: Arc<VulkanBasicContext>,
    rdx: Option<RenderContext>,
}

struct RenderContext {
    window_ctx: VulkanoWindows,
    id: WindowId,
    // swapchain: Arc<Swapchain>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    pipeline: Arc<GraphicsPipeline>,
    // viewport: Viewport,
    // recreate_swapchain: bool,
}

struct RenderContextBuilder {
    basic_cntx: Arc<VulkanoContext>,
    window_ctx: VulkanoWindows,
    id: WindowId,
    render_pass: Option<Arc<RenderPass>>,
    framebuffers: Option<Vec<Arc<Framebuffer>>>,
    pipeline: Option<Arc<GraphicsPipeline>>,
}

impl RenderContextBuilder {
    pub fn new(event_loop: &ActiveEventLoop, basic_cntx: Arc<VulkanoContext>) -> Self {
        let mut window_ctx = VulkanoWindows::default();
        let window_descr = WindowDescriptor {
            title: "Cube".to_string(),
            mode: WindowMode::BorderlessFullscreen,
            ..Default::default()
        };
        let id = window_ctx.create_window(event_loop, &basic_cntx, &window_descr, |_| {});

        Self {
            window_ctx,
            id,
            basic_cntx,
            render_pass: None,
            framebuffers: None,
            pipeline: None,
        }
    }

    pub fn with_render_pass(mut self) -> Self {
        let pass = single_pass_renderpass!(
            self.basic_cntx.device().clone(),
            attachments: {
                color: {
                    format: self.window_ctx.get_renderer(self.id).unwrap().swapchain_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        )
        .unwrap();
        self.render_pass = Some(pass);
        self
    }

    pub fn with_frame_buffer(mut self) -> Self {
        let framebuffers: Vec<Arc<Framebuffer>> = self
            .window_ctx
            .get_renderer(self.id)
            .as_ref()
            .unwrap()
            .swapchain_image_views()
            .iter()
            .map(|image_view| {
                let frame_buf_info = FramebufferCreateInfo {
                    attachments: vec![image_view.clone()],
                    ..Default::default()
                };
                Framebuffer::new(self.render_pass.as_ref().unwrap().clone(), frame_buf_info)
                    .unwrap()
            })
            .collect();

        self.framebuffers = Some(framebuffers);
        self
    }

    pub fn with_pipeline(mut self) -> Self {
        let stages = [
            PipelineShaderStageCreateInfo::new(
                vs::load(self.basic_cntx.device().clone())
                    .unwrap()
                    .entry_point("main")
                    .unwrap(),
            ),
            PipelineShaderStageCreateInfo::new(
                fs::load(self.basic_cntx.device().clone())
                    .unwrap()
                    .entry_point("main")
                    .unwrap(),
            ),
        ]
        .to_vec()
        .into();

        let layout = PipelineLayout::new(
            self.basic_cntx.device().clone(),
            PipelineLayoutCreateInfo::default(),
        )
        .unwrap();

        let subpass = Subpass::from(self.render_pass.as_ref().unwrap().clone(), 0).unwrap();

        let create_info = GraphicsPipelineCreateInfo {
            stages: stages,
            rasterization_state: Some(RasterizationState::default()),
            vertex_input_state: Some(VertexInputState::default()),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState::default()),
            multisample_state: Some(MultisampleState::default()),
            subpass: Some(subpass.into()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                1,
                ColorBlendAttachmentState::default(),
            )),
            ..GraphicsPipelineCreateInfo::layout(layout)
        };

        self.pipeline = Some(
            GraphicsPipeline::new(self.basic_cntx.device().clone(), None, create_info)
                .unwrap()
                .clone(),
        );
        self
    }

    pub fn build(self) -> RenderContext {
        RenderContext {
            window_ctx: self.window_ctx,
            id: self.id,
            render_pass: self.render_pass.unwrap().clone(),
            framebuffers: self.framebuffers.unwrap().clone(),
            pipeline: self
                .pipeline
                .expect(
                    "you are trying to create a pipeline that are not initialized in the builder",
                )
                .clone(),
        }
    }
}

impl RenderContext {
    fn new(event_loop: &ActiveEventLoop, basic_cntx: Arc<VulkanoContext>) -> Self {
        RenderContextBuilder::new(event_loop, basic_cntx)
            .with_render_pass()
            .with_frame_buffer()
            .with_pipeline()
            .build()
    }

    pub fn draw(&mut self, allocator: Arc<StandardCommandBufferAllocator>) {
        let render_pass = self.render_pass.clone();
        let mut new_framebuffers = None;

        let acquire_result = self.window_ctx.get_renderer_mut(self.id).unwrap().acquire(
            None,
            |swapchain_image_views| {
                let framebuffers: Vec<Arc<Framebuffer>> = swapchain_image_views
                    .iter()
                    .map(|image_view| {
                        Framebuffer::new(
                            render_pass.clone(),
                            FramebufferCreateInfo {
                                attachments: vec![image_view.clone()],
                                ..Default::default()
                            },
                        )
                        .unwrap()
                    })
                    .collect();
                new_framebuffers = Some(framebuffers);
            },
        );

        // Update framebuffers if swapchain was recreated
        if let Some(framebuffers) = new_framebuffers {
            self.framebuffers = framebuffers;
        }

        let next_image_index = self.window_ctx.get_renderer(self.id).unwrap().image_index();
        let command_buffer = self.build_cmd_buf(allocator, next_image_index);

        let acquire_future = match acquire_result {
            Ok(future) => future,
            _ => {
                panic!("something went wrong with acquiring the future")
            }
        };

        let cmd_buf_executed = acquire_future
            .then_execute(
                self.window_ctx
                    .get_renderer(self.id)
                    .unwrap()
                    .graphics_queue()
                    .clone(),
                command_buffer,
            )
            .unwrap()
            .boxed();

        self.window_ctx
            .get_renderer_mut(self.id)
            .unwrap()
            .present(cmd_buf_executed, false);
    }

    pub fn build_cmd_buf(
        &self,
        allocator: Arc<StandardCommandBufferAllocator>,
        next_idx: u32,
    ) -> Arc<PrimaryAutoCommandBuffer> {
        let mut cb = AutoCommandBufferBuilder::primary(
            allocator,
            self.window_ctx
                .get_primary_renderer()
                .as_ref()
                .unwrap()
                .graphics_queue()
                .queue_family_index(),
            vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let render_pass_begin_info = RenderPassBeginInfo {
            clear_values: vec![Some([0.0, 1.0, 0.0, 1.0].into())],
            ..RenderPassBeginInfo::framebuffer(self.framebuffers[next_idx as usize].clone())
        };

        cb.begin_render_pass(render_pass_begin_info, Default::default())
            .unwrap();

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: self
                .window_ctx
                .get_renderer(self.id)
                .unwrap()
                .swapchain_image_size()
                .map(|v| v as f32),
            depth_range: 0.0..=1.0,
        };
        cb.set_viewport(0, [viewport].into_iter().collect())
            .unwrap();

        cb.bind_pipeline_graphics(self.pipeline.clone()).unwrap();

        unsafe {
            cb.draw(
                3, // vertex_count (e.g., 3 for a triangle)
                1, // instance_count
                0, // first_vertex
                0, // first_instance
            )
        }
        .unwrap();

        // 9. End render pass
        cb.end_render_pass(Default::default()).unwrap();

        // 10. Build the command buffer
        let command_buffer = cb.build().unwrap();
        command_buffer
    }
}

impl App {
    fn new() -> Self {
        let context = VulkanBasicContext::new();
        App {
            basic_context: Arc::new(context),
            rdx: None,
        }
    }
}

impl VulkanBasicContext {
    fn new() -> Self {
        let bctx = Arc::new(VulkanoContext::new(VulkanoConfig::default()));
        let cb_alloc = Arc::new(StandardCommandBufferAllocator::new(
            bctx.device().clone(),
            Default::default(),
        ));

        VulkanBasicContext { bctx, cb_alloc }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.rdx = Some(RenderContext::new(
            event_loop,
            self.basic_context.bctx.clone(),
        ));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                //TODO i guess you'll be rendering here
                self.rdx
                    .as_mut()
                    .unwrap()
                    .draw(self.basic_context.cb_alloc.clone());
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Request redraw for continuous rendering
        if let Some(rdx) = &self.rdx {
            rdx.window_ctx
                .get_window(rdx.id)
                .expect("Failed to get window")
                .request_redraw();
        }
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/vert.glsl",
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/frag.glsl",
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app);
}
