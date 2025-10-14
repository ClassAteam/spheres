use std::sync::Arc;
use vulkano::{
    Validated, VulkanError,
    buffer::{Buffer, BufferUsage, Subbuffer, sys::BufferCreateInfo},
    command_buffer::{
        CommandBufferUsage, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
        allocator::StandardCommandBufferAllocator, auto::AutoCommandBufferBuilder,
    },
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
        physical::{PhysicalDevice, PhysicalDeviceType},
    },
    image::{ImageUsage, view::ImageView},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    library::VulkanLibrary,
    memory::allocator::{
        AllocationCreateInfo, FreeListAllocator, GenericMemoryAllocator, MemoryTypeFilter,
        StandardMemoryAllocator,
    },
    pipeline::{
        DynamicState, GraphicsPipeline, PipelineCreateFlags, PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
        },
        layout::PipelineLayout,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::{
        Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo, acquire_next_image,
    },
    sync::future::GpuFuture,
};

use winit::{event_loop::EventLoop, window::Window};

use crate::model::{Position, TEST_TRIANGLE};

pub struct RenderContext {
    queue: Arc<Queue>,
    instance: Arc<Instance>,
    device: Arc<Device>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    vertex_buffer: Subbuffer<[Position]>,
    window_context: Option<WindowDependentContext>,
}

struct WindowDependentContext {
    window: Arc<Window>,
    swapchain: Arc<Swapchain>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    pipeline: Arc<GraphicsPipeline>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    viewport: Viewport,
    recreate_swapchain: bool,
}

impl WindowDependentContext {
    fn new(instance: Arc<Instance>, window: Arc<Window>, device: Arc<Device>) -> Self {
        let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();
        let window_size = window.inner_size();

        let (swapchain, images) = {
            let surface_capabilities = device
                .physical_device()
                .surface_capabilities(&surface, Default::default())
                .unwrap();
            let (image_format, _) = device
                .physical_device()
                .surface_formats(&surface, Default::default())
                .unwrap()[0];

            Swapchain::new(
                device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count.max(2),
                    image_format,
                    image_extent: window_size.into(),
                    image_usage: ImageUsage::COLOR_ATTACHMENT,
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .into_iter()
                        .next()
                        .unwrap(),
                    ..Default::default()
                },
            )
            .unwrap()
        };

        let render_pass = RenderContext::create_render_pass(device.clone(), swapchain.clone());
        let vs = vs::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let fs = fs::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: window_size.into(),
            depth_range: 0.0..=1.0,
        };

        let framebuffers = images
            .iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone()).unwrap();

                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: [view.clone()].to_vec(),
                        ..Default::default()
                    },
                )
                .unwrap()
            })
            .collect::<Vec<_>>();

        use vulkano::sync;
        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        let pipeline = {
            let vertex_input_state = [Position::per_vertex()].definition(&vs).unwrap();

            let stages = [
                PipelineShaderStageCreateInfo::new(vs.clone()),
                PipelineShaderStageCreateInfo::new(fs.clone()),
            ]
            .to_vec()
            .into();

            let layout = PipelineLayout::new(device.clone(), Default::default()).unwrap();
            let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

            GraphicsPipeline::new(
                device.clone(),
                None,
                GraphicsPipelineCreateInfo {
                    flags: PipelineCreateFlags::default(),
                    stages,
                    vertex_input_state: Some(vertex_input_state),
                    input_assembly_state: Some(InputAssemblyState {
                        topology: PrimitiveTopology::TriangleList,
                        ..Default::default()
                    }),
                    rasterization_state: Some(RasterizationState::default()),
                    subpass: Some((subpass).into()),
                    base_pipeline: None,
                    viewport_state: Some(ViewportState::default()),
                    tessellation_state: None,
                    depth_stencil_state: None,
                    multisample_state: Some(MultisampleState::default()),
                    color_blend_state: Some(ColorBlendState::with_attachment_states(
                        1,
                        ColorBlendAttachmentState::default(),
                    )),
                    dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                    fragment_shading_rate_state: None,
                    discard_rectangle_state: None,
                    ..GraphicsPipelineCreateInfo::layout(layout)
                },
            )
            .unwrap()
        };

        Self {
            window,
            swapchain,
            render_pass,
            framebuffers,
            pipeline,
            previous_frame_end,
            viewport,
            recreate_swapchain: false,
        }
    }

    fn redraw(
        &mut self,
        device: Arc<Device>,
        cba: Arc<StandardCommandBufferAllocator>,
        queue: Arc<Queue>,
        vertex_buffer: Subbuffer<[Position]>,
    ) {
        let window_size = self.window.inner_size();

        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if self.recreate_swapchain {
            let (new_swapchain, new_images) = self
                .swapchain
                .recreate(SwapchainCreateInfo {
                    image_extent: window_size.into(),
                    ..self.swapchain.create_info()
                })
                .expect("failed to recreate swapchain");

            self.swapchain = new_swapchain;

            self.render_pass =
                RenderContext::create_render_pass(device.clone(), self.swapchain.clone());

            self.viewport.extent = window_size.into();

            self.recreate_swapchain = false;

            self.framebuffers = new_images
                .iter()
                .map(|image| {
                    let view = ImageView::new_default(image.clone()).unwrap();

                    Framebuffer::new(
                        self.render_pass.clone(),
                        FramebufferCreateInfo {
                            attachments: [view.clone()].to_vec(),
                            ..Default::default()
                        },
                    )
                    .unwrap()
                })
                .collect::<Vec<_>>();
        }

        let (image_index, _suboptimal, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None).map_err(Validated::unwrap) {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => {
                    return;
                }
                Err(e) => panic!("failed to acquire next image: {e}"),
            };

        let mut builder = AutoCommandBufferBuilder::primary(
            cba.clone(),
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],

                    ..RenderPassBeginInfo::framebuffer(
                        self.framebuffers[image_index as usize].clone(),
                    )
                },
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .unwrap()
            .set_viewport(0, [self.viewport.clone()].into_iter().collect())
            .unwrap();

        builder
            .bind_pipeline_graphics(self.pipeline.clone())
            .unwrap()
            .bind_vertex_buffers(0, vertex_buffer.clone())
            .unwrap();

        unsafe { builder.draw(vertex_buffer.len() as u32, 1, 0, 0) }.unwrap();

        builder.end_render_pass(Default::default()).unwrap();

        let command_buffer = builder.build().unwrap();

        let future = self
            .previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        match future.map_err(Validated::unwrap) {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            }
            Err(e) => {
                panic!("failed to flush future: {e}");
            }
        }
    }
}

impl RenderContext {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let library = VulkanLibrary::new().unwrap();
        let required_extensions = Surface::required_extensions(event_loop).unwrap();
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: required_extensions,
                ..Default::default()
            },
        )
        .unwrap();

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) =
            Self::pick_physical_device_and_queue(instance.clone(), &device_extensions, event_loop);

        println!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        let (device, queue) = Self::create_device_and_one_queue(
            physical_device,
            queue_family_index,
            device_extensions,
        );

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let vertex_buffer = Self::create_vertex_buffer(memory_allocator);

        Self {
            instance,
            device,
            queue,
            command_buffer_allocator,
            vertex_buffer,
            window_context: None,
        }
    }

    fn create_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Arc<RenderPass> {
        let render_pass = vulkano::single_pass_renderpass!(
            device,
            attachments: {
                color: {
                    format: swapchain.image_format(),
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
        render_pass
    }

    fn create_device_and_one_queue(
        physical_device: Arc<PhysicalDevice>,
        queue_family_index: u32,
        device_extensions: DeviceExtensions,
    ) -> (Arc<Device>, Arc<Queue>) {
        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: [QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }]
                .to_vec(),
                ..Default::default()
            },
        )
        .unwrap();

        let queue = queues.next().unwrap();

        (device, queue)
    }

    fn pick_physical_device_and_queue(
        instance: Arc<Instance>,
        extensions: &DeviceExtensions,
        event_loop: &EventLoop<()>,
    ) -> (Arc<PhysicalDevice>, u32) {
        instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| p.supported_extensions().contains(extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && p.presentation_support(i as u32, event_loop).unwrap()
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .unwrap()
    }

    fn create_vertex_buffer(
        allocator: Arc<GenericMemoryAllocator<FreeListAllocator>>,
    ) -> Subbuffer<[Position]> {
        let vertex_buffer = Buffer::from_iter(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            TEST_TRIANGLE,
        )
        .unwrap();
        vertex_buffer
    }

    pub fn resumed(&mut self, window: Arc<Window>) {
        self.window_context = Some(WindowDependentContext::new(
            self.instance.clone(),
            window,
            self.device.clone(),
        ));
    }

    pub fn window_invalidated(&mut self) {
        if let Some(ref mut window_context) = self.window_context {
            window_context.recreate_swapchain = true;
        }
    }

    pub fn draw(&mut self) {
        if let Some(ref mut window_context) = self.window_context {
            window_context.redraw(
                self.device.clone(),
                self.command_buffer_allocator.clone(),
                self.queue.clone(),
                self.vertex_buffer.clone(),
            );
        }
    }

    pub fn request_redraw(&mut self) {
        self.window_context
            .as_mut()
            .unwrap()
            .window
            .request_redraw();
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
