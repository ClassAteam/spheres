use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::sync::GpuFuture;
use vulkano::{single_pass_renderpass, sync};
use vulkano_util::context::VulkanoContext;
use vulkano_util::window::{VulkanoWindows, WindowDescriptor, WindowMode};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use super::context::RenderContext;
use super::pipeline::create_graphics_pipeline;
use crate::models::{Position, INDICES, POSITIONS};
use crate::shaders::vs;

pub struct RenderContextBuilder {
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
        self.pipeline = Some(create_graphics_pipeline(
            self.basic_cntx.device().clone(),
            self.render_pass.as_ref().unwrap().clone(),
        ));
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
