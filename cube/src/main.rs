use glam::{Mat4, Vec3};
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, RenderPassBeginInfo,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::layout::{
    DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorType,
};
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::depth_stencil::{CompareOp, DepthState, DepthStencilState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::layout::PipelineLayoutCreateInfo;
use vulkano::pipeline::{
    DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
    PipelineShaderStageCreateInfo,
};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::shader::ShaderStages;
use vulkano::sync::GpuFuture;
use vulkano::{single_pass_renderpass, sync};
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

mod models;
use crate::models::{INDICES, POSITIONS};
use crate::vs::Data;

use self::models::Position;

struct VulkanBasicContext {
    bctx: Arc<VulkanoContext>,
    cb_alloc: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

struct App {
    basic_context: Arc<VulkanBasicContext>,
    rdx: Option<RenderContext>,
}

struct RenderContext {
    bctx: Arc<VulkanoContext>,
    window_ctx: VulkanoWindows,
    id: WindowId,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    pipeline: Arc<GraphicsPipeline>,
    vertex_buffer: Subbuffer<[Position]>,
    index_buffer: Subbuffer<[u16]>,
    uniform_buffers: Vec<Subbuffer<vs::Data>>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
}

struct RenderContextBuilder {
    basic_cntx: Arc<VulkanoContext>,
    window_ctx: VulkanoWindows,
    id: WindowId,
    render_pass: Option<Arc<RenderPass>>,
    framebuffers: Option<Vec<Arc<Framebuffer>>>,
    pipeline: Option<Arc<GraphicsPipeline>>,
    vertex_buffer: Option<Subbuffer<[Position]>>,
    index_buffer: Option<Subbuffer<[u16]>>,
    uniform_buffers: Option<Vec<Subbuffer<vs::Data>>>,
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
            vertex_buffer: None,
            index_buffer: None,
            uniform_buffers: None,
        }
    }

    pub fn with_vertex_buffers(mut self) -> Self {
        let vertex_buffer = Buffer::from_iter(
            self.basic_cntx.memory_allocator().clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            POSITIONS,
        )
        .unwrap();
        self.vertex_buffer = Some(vertex_buffer);
        self
    }

    pub fn with_index_buffer(mut self) -> Self {
        let index_buffer = Buffer::from_iter(
            self.basic_cntx.memory_allocator().clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            INDICES,
        )
        .unwrap();
        self.index_buffer = Some(index_buffer);
        self
    }

    pub fn with_uniform_buffers(mut self) -> Self {
        let uniform_buffers = (0..self
            .window_ctx
            .get_renderer(self.id)
            .unwrap()
            .swapchain_image_views()
            .iter()
            .count())
            .map(|_| {
                Buffer::new_sized(
                    self.basic_cntx.memory_allocator().clone(),
                    BufferCreateInfo {
                        usage: BufferUsage::UNIFORM_BUFFER,
                        ..Default::default()
                    },
                    AllocationCreateInfo {
                        memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                            | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                        ..Default::default()
                    },
                )
                .unwrap()
            })
            .collect();

        self.uniform_buffers = Some(uniform_buffers);
        self
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
        self.render_pass = Some(pass);
        self
    }

    pub fn with_frame_buffer(mut self) -> Self {
        let images = self
            .window_ctx
            .get_renderer(self.id)
            .unwrap()
            .swapchain_image_views();

        let framebuffers = Self::create_frame_buffers(
            images,
            self.render_pass.as_ref().unwrap().clone(),
            self.basic_cntx.memory_allocator().to_owned(),
        );

        self.framebuffers = Some(framebuffers);
        self
    }

    pub fn create_frame_buffers(
        images: &[Arc<ImageView>],
        render_pass: Arc<RenderPass>,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Vec<Arc<Framebuffer>> {
        let depth_buffer = ImageView::new_default(
            Image::new(
                memory_allocator,
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: Format::D16_UNORM,
                    extent: images[0].image().extent(),
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .unwrap(),
        )
        .unwrap();
        let framebuffers = images
            .iter()
            .map(|image| {
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![image.clone(), depth_buffer.clone()],
                        ..Default::default()
                    },
                )
                .unwrap()
            })
            .collect::<Vec<_>>();

        framebuffers
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
            PipelineLayoutCreateInfo {
                set_layouts: vec![
                    DescriptorSetLayout::new(
                        self.basic_cntx.device().clone(),
                        DescriptorSetLayoutCreateInfo {
                            bindings: [(
                                0,
                                DescriptorSetLayoutBinding {
                                    stages: ShaderStages::VERTEX,
                                    ..DescriptorSetLayoutBinding::descriptor_type(
                                        DescriptorType::UniformBuffer,
                                    )
                                },
                            )]
                            .into(),
                            ..Default::default()
                        },
                    )
                    .unwrap(),
                ],
                ..Default::default()
            },
        )
        .unwrap();

        let subpass = Subpass::from(self.render_pass.as_ref().unwrap().clone(), 0).unwrap();

        let vertex_input_state = [Position::per_vertex()]
            .definition(
                &vs::load(self.basic_cntx.device().clone())
                    .unwrap()
                    .entry_point("main")
                    .unwrap(),
            )
            .unwrap();

        let create_info = GraphicsPipelineCreateInfo {
            stages: stages,
            rasterization_state: Some(RasterizationState::default()),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState::default()),
            multisample_state: Some(MultisampleState::default()),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState {
                    compare_op: CompareOp::Less,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            subpass: Some(subpass.into()),
            dynamic_state: [DynamicState::Viewport].into_iter().collect(),
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
            bctx: self.basic_cntx.clone(),
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
            vertex_buffer: self.vertex_buffer.unwrap().clone(),
            index_buffer: self.index_buffer.unwrap().clone(),
            uniform_buffers: self.uniform_buffers.unwrap().clone(),
            previous_frame_end: Some(sync::now(self.basic_cntx.device().clone()).boxed()),
        }
    }
}

impl RenderContext {
    fn new(event_loop: &ActiveEventLoop, basic_cntx: Arc<VulkanoContext>) -> Self {
        RenderContextBuilder::new(event_loop, basic_cntx)
            .with_render_pass()
            .with_frame_buffer()
            .with_pipeline()
            .with_vertex_buffers()
            .with_index_buffer()
            .with_uniform_buffers()
            .build()
    }

    pub fn draw(
        &mut self,
        cb_alloc: Arc<StandardCommandBufferAllocator>,
        mem_alloc: Arc<StandardMemoryAllocator>,
        desc_alloc: Arc<StandardDescriptorSetAllocator>,
    ) {
        let render_pass = self.render_pass.clone();
        let mut new_framebuffers = None;

        let acquire_result = self.window_ctx.get_renderer_mut(self.id).unwrap().acquire(
            None,
            |swapchain_image_views| {
                let framebuffers: Vec<Arc<Framebuffer>> =
                    RenderContextBuilder::create_frame_buffers(
                        swapchain_image_views,
                        render_pass,
                        mem_alloc,
                    );
                new_framebuffers = Some(framebuffers);
            },
        );

        if let Some(framebuffers) = new_framebuffers {
            self.framebuffers = framebuffers;
        }

        let next_image_index = self.window_ctx.get_renderer(self.id).unwrap().image_index();
        let command_buffer = self.build_cmd_buf(cb_alloc, desc_alloc, next_image_index);

        let acquire_future = match acquire_result {
            Ok(future) => future,
            _ => {
                panic!("something went wrong with acquiring the future")
            }
        };

        let cmd_buf_executed = acquire_future
            .join(self.previous_frame_end.take().unwrap())
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
            .present(cmd_buf_executed, true);

        self.previous_frame_end = Some(sync::now(self.bctx.device().clone()).boxed());
    }

    pub fn build_cmd_buf(
        &mut self,
        cmb_alloc: Arc<StandardCommandBufferAllocator>,
        desc_mem_alloc: Arc<StandardDescriptorSetAllocator>,
        next_idx: u32,
    ) -> Arc<PrimaryAutoCommandBuffer> {
        let uniform_buffer = self.update_uniform();
        let layout = self.pipeline.layout().set_layouts()[0].clone();
        let descriptor_set = DescriptorSet::new(
            desc_mem_alloc,
            layout,
            [WriteDescriptorSet::buffer(0, uniform_buffer)],
            [],
        )
        .unwrap();
        let mut cb = AutoCommandBufferBuilder::primary(
            cmb_alloc,
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
            clear_values: vec![
                Some([0.0, 0.0, 0.0, 1.0].into()), // Color attachment
                Some(1.0.into()),                  // Depth attachment
            ],
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

        cb.bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            self.pipeline.layout().clone(),
            0,
            descriptor_set,
        )
        .unwrap();

        cb.bind_vertex_buffers(0, self.vertex_buffer.clone())
            .unwrap();

        cb.bind_index_buffer(self.index_buffer.clone()).unwrap();

        unsafe { cb.draw_indexed(self.index_buffer.len() as u32, 1, 0, 0, 0) }.unwrap();

        cb.end_render_pass(Default::default()).unwrap();

        let command_buffer = cb.build().unwrap();
        command_buffer
    }

    pub fn update_uniform(&self) -> Subbuffer<Data> {
        // In your render loop:
        let model_matrix = Mat4::from_rotation_y(0.3) * Mat4::from_rotation_x(0.5);

        // Combined with view and projection:
        let view = Mat4::look_at_rh(
            Vec3::new(0.0, 0.0, 5.0), // camera position
            Vec3::ZERO,               // look at origin
            Vec3::Y,                  // up direction
        );

        let aspect_ratio = self
            .window_ctx
            .get_renderer(self.id)
            .unwrap()
            .aspect_ratio();

        let projection = Mat4::perspective_rh(
            45.0_f32.to_radians(), // FOV
            aspect_ratio,
            0.1,   // near plane
            100.0, // far plane
        );

        let mvp = projection * view * model_matrix;

        let uniform_data = vs::Data {
            mvp: mvp.to_cols_array_2d(),
        };

        let buffer = self.uniform_buffers
            [self.window_ctx.get_renderer(self.id).unwrap().image_index() as usize]
            .clone();

        *buffer.write().unwrap() = uniform_data;

        buffer
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

        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            bctx.device().clone(),
            Default::default(),
        ));
        VulkanBasicContext {
            bctx,
            cb_alloc,
            descriptor_set_allocator,
        }
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
                self.rdx.as_mut().unwrap().draw(
                    self.basic_context.cb_alloc.clone(),
                    self.basic_context.bctx.memory_allocator().clone(),
                    self.basic_context.descriptor_set_allocator.clone(),
                );
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
    _ = event_loop.run_app(&mut app);
}
