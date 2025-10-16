use std::sync::Arc;
use vulkano::{
    Validated, VulkanError,
    command_buffer::{
        CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo,
        SubpassContents, allocator::StandardCommandBufferAllocator, auto::AutoCommandBufferBuilder,
    },
    device::{Device, Queue},
    image::{Image, ImageUsage, view::ImageView},
    instance::Instance,
    pipeline::{
        DynamicState, GraphicsPipeline, Pipeline, PipelineCreateFlags,
        PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::VertexInputState,
            viewport::{Viewport, ViewportState},
        },
        layout::{PipelineLayout, PushConstantRange},
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    shader::{EntryPoint, ShaderStages},
    swapchain::{
        Surface, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo, SwapchainPresentInfo,
        acquire_next_image,
    },
    sync::future::GpuFuture,
};

use winit::{dpi::PhysicalSize, window::Window};

use crate::control::Control;
use crate::model::CircleParams;

pub struct WindowDependentContext {
    pub window: Arc<Window>,
    swapchain: Arc<Swapchain>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    pipeline: Arc<GraphicsPipeline>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    viewport: Viewport,
    recreate_swapchain: bool,
    control: Control,
    circle_params: CircleParams,
}

impl WindowDependentContext {
    pub fn new(instance: Arc<Instance>, window: Arc<Window>, device: Arc<Device>) -> Self {
        let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();
        let window_size = window.inner_size();

        let (swapchain, images) =
            Self::create_first_swapchain(device.clone(), surface, window_size);

        let render_pass = Self::create_render_pass(device.clone(), swapchain.clone());

        let (vertex_shader, fragment_shader) = Self::prepare_shaders(device.clone());

        let viewport = Self::create_viewport(window_size);

        let framebuffers = Self::create_frame_buffers(images, render_pass.clone());

        use vulkano::sync;
        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        let pipeline =
            Self::create_pipeline(vertex_shader, fragment_shader, device, render_pass.clone());

        Self {
            window,
            swapchain,
            render_pass,
            framebuffers,
            pipeline,
            previous_frame_end,
            viewport,
            recreate_swapchain: false,
            control: Control::new(),
            circle_params: CircleParams::default(),
        }
    }

    pub fn redraw(
        &mut self,
        device: Arc<Device>,
        cba: Arc<StandardCommandBufferAllocator>,
        queue: Arc<Queue>,
    ) {
        let window_size = self.window.inner_size();

        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if self.recreate_swapchain {
            self.recreate_swapchain(window_size, device);
        }

        let (image_index, _suboptimal, acquire_future) =
            match Self::acquire_image(self.swapchain.clone()) {
                Some(result) => result,
                None => return,
            };

        let command_buffer = self.create_cmd_buffer(cba, queue.clone(), image_index);

        self.execute_and_present(acquire_future, queue, command_buffer, image_index);
    }

    fn create_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Arc<RenderPass> {
        vulkano::single_pass_renderpass!(
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
        .unwrap()
    }

    fn create_frame_buffers(
        images: Vec<Arc<Image>>,
        render_pass: Arc<RenderPass>,
    ) -> Vec<Arc<Framebuffer>> {
        images
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
            .collect::<Vec<_>>()
    }

    fn recreate_swapchain(&mut self, window_size: PhysicalSize<u32>, device: Arc<Device>) {
        let (new_swapchain, new_images) = self
            .swapchain
            .recreate(SwapchainCreateInfo {
                image_extent: window_size.into(),
                ..self.swapchain.create_info()
            })
            .expect("failed to recreate swapchain");

        self.swapchain = new_swapchain;

        self.render_pass = Self::create_render_pass(device.clone(), self.swapchain.clone());

        self.viewport = Self::create_viewport(window_size);

        self.recreate_swapchain = false;

        self.framebuffers = Self::create_frame_buffers(new_images, self.render_pass.clone());
    }

    fn acquire_image(swapchain: Arc<Swapchain>) -> Option<(u32, bool, SwapchainAcquireFuture)> {
        match acquire_next_image(swapchain, None).map_err(Validated::unwrap) {
            Ok(r) => Some(r),
            Err(VulkanError::OutOfDate) => None,
            Err(e) => panic!("failed to acquire next image: {e}"),
        }
    }

    fn create_cmd_buffer(
        &mut self,
        cba: Arc<StandardCommandBufferAllocator>,
        queue: Arc<Queue>,
        image_index: u32,
    ) -> Arc<PrimaryAutoCommandBuffer> {
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

        let aspect_ratio = self.viewport.extent[0] / self.viewport.extent[1];
        let push_constants = [
            self.control.rotation_angle, 
            aspect_ratio,
            self.circle_params.radius,
            self.circle_params.segments as f32,
        ];
        
        builder
            .bind_pipeline_graphics(self.pipeline.clone())
            .unwrap()
            .push_constants(
                self.pipeline.layout().clone(),
                0,
                push_constants,
            )
            .unwrap();

        // Draw a single circle for now (one circle worth of vertices)
        unsafe { builder.draw(self.circle_params.segments as u32, 1, 0, 0) }.unwrap();

        builder.end_render_pass(Default::default()).unwrap();

        builder.build().unwrap()
    }

    fn execute_and_present(
        &mut self,
        acquire_future: SwapchainAcquireFuture,
        queue: Arc<Queue>,
        command_buffer: Arc<PrimaryAutoCommandBuffer>,
        image_index: u32,
    ) {
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

    fn create_first_swapchain(
        device: Arc<Device>,
        surface: Arc<Surface>,
        window_size: PhysicalSize<u32>,
    ) -> (Arc<Swapchain>, Vec<Arc<Image>>) {
        let surface_capabilities = device
            .physical_device()
            .surface_capabilities(&surface, Default::default())
            .unwrap();
        let (image_format, _) = device
            .physical_device()
            .surface_formats(&surface, Default::default())
            .unwrap()[0];

        Swapchain::new(
            device,
            surface,
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
    }

    fn create_pipeline(
        vs: EntryPoint,
        fs: EntryPoint,
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
    ) -> Arc<GraphicsPipeline> {
        let vertex_input_state = VertexInputState::default(); // No vertex input - procedural generation

        let stages = [
            PipelineShaderStageCreateInfo::new(vs.clone()),
            PipelineShaderStageCreateInfo::new(fs.clone()),
        ]
        .to_vec()
        .into();

        let push_constant_range = PushConstantRange {
            stages: ShaderStages::VERTEX,
            offset: 0,
            size: (std::mem::size_of::<f32>() * 4) as u32, // rotation_angle + aspect_ratio + circle_radius + segments
        };

        let layout = PipelineLayout::new(
            device.clone(),
            vulkano::pipeline::layout::PipelineLayoutCreateInfo {
                push_constant_ranges: vec![push_constant_range],
                ..Default::default()
            },
        )
        .unwrap();
        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

        GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                flags: PipelineCreateFlags::default(),
                stages,
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState {
                    topology: PrimitiveTopology::LineStrip,
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
    }

    fn prepare_shaders(device: Arc<Device>) -> (EntryPoint, EntryPoint) {
        let vs = vs::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let fs = fs::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        (vs, fs)
    }

    fn create_viewport(window_size: PhysicalSize<u32>) -> Viewport {
        let width = window_size.width as f32;
        let height = window_size.height as f32;
        Viewport {
            offset: [0.0, 0.0],
            extent: [width, height],
            depth_range: 0.0..=1.0,
        }
    }

    pub fn swapchain_recreation_needed(&mut self) {
        self.recreate_swapchain = true;
    }

    pub fn change_angle_up(&mut self) {
        self.control.rotate_up();
    }

    pub fn change_angle_down(&mut self) {
        self.control.rotate_down();
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
