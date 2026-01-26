use std::sync::Arc;

use glam::Vec3;
use vulkano::{
    buffer::{
        Buffer, BufferCreateInfo, BufferUsage, Subbuffer,
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
    },
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, RenderPassBeginInfo},
    descriptor_set::{
        DescriptorSet, WriteDescriptorSet,
        allocator::StandardDescriptorSetAllocator,
        layout::{
            DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo,
            DescriptorType,
        },
    },
    format::Format,
    image::ImageUsage,
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
    pipeline::{
        DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, RasterizationState},
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
        },
        layout::PipelineLayoutCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    shader::ShaderStages,
    single_pass_renderpass,
};
use vulkano_util::{context::VulkanoContext, renderer::VulkanoWindowRenderer};

use crate::{
    cube_pass::{
        models::{INDICES, POSITIONS, Position},
        shaders::{
            fs,
            vs::{self, Data},
        },
        transform::TransformState,
    },
    render::RenderContext,
};

pub struct CubePass {
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,
    vertex_buffer: Subbuffer<[Position]>,
    index_buffer: Subbuffer<[u16]>,
    transform: TransformState,
    uniform_allocator: SubbufferAllocator,
}

impl CubePass {
    pub fn new(renderer: &mut VulkanoWindowRenderer, basic_context: &VulkanoContext) -> Self {
        let render_pass = Self::create_render_pass(renderer, basic_context);
        let pipeline = Self::create_graphics_pipeline(basic_context, render_pass.clone());
        let vertex_buffer = Self::create_vertex_buffers(basic_context);
        let index_buffer = Self::create_index_buffer(basic_context);
        let uniform_allocator = Self::create_uniform_buffers(basic_context);
        CubePass {
            render_pass,
            pipeline,
            vertex_buffer,
            index_buffer,
            transform: TransformState::new(),
            uniform_allocator,
        }
    }

    pub fn get_transform_state(&self) -> &TransformState {
        &self.transform
    }

    pub fn yaw_left(&mut self) {
        self.transform.rotate_model(Vec3::new(0.0, -0.01, 0.0));
    }
    pub fn yaw_right(&mut self) {
        self.transform.rotate_model(Vec3::new(0.0, 0.01, 0.0));
    }
    pub fn pitch_down(&mut self) {
        self.transform.rotate_model(Vec3::new(-0.01, 0.0, 0.0));
    }
    pub fn pitch_up(&mut self) {
        self.transform.rotate_model(Vec3::new(0.01, 0.0, 0.0));
    }

    pub fn move_right(&mut self) {
        self.transform.translate_model(Vec3::new(0.01, 0.0, 0.0));
    }
    pub fn move_left(&mut self) {
        self.transform.translate_model(Vec3::new(-0.01, 0.0, 0.0));
    }
    pub fn move_down(&mut self) {
        self.transform.translate_model(Vec3::new(0.0, -0.01, 0.0));
    }
    pub fn move_up(&mut self) {
        self.transform.translate_model(Vec3::new(0.0, 0.01, 0.0));
    }
    pub fn move_back(&mut self) {
        self.transform.translate_model(Vec3::new(0.0, 0.0, 0.01));
    }
    pub fn move_forward(&mut self) {
        self.transform.translate_model(Vec3::new(0.0, 0.0, -0.01));
    }

    pub fn scale_up(&mut self) {
        self.transform.scale_model(Vec3::new(0.01, 0.01, 0.01));
    }
    pub fn scale_down(&mut self) {
        self.transform.scale_model(Vec3::new(-0.01, -0.01, -0.01));
    }

    pub fn camera_move_right(&mut self) {
        self.transform.camera_position(Vec3::new(0.01, 0.0, 0.0));
    }
    pub fn camera_move_left(&mut self) {
        self.transform.camera_position(Vec3::new(-0.01, 0.0, 0.0));
    }
    pub fn camera_move_up(&mut self) {
        self.transform.camera_position(Vec3::new(0.0, 0.01, 0.0));
    }
    pub fn camera_move_down(&mut self) {
        self.transform.camera_position(Vec3::new(0.0, -0.01, 0.0));
    }
    pub fn camera_move_back(&mut self) {
        self.transform.camera_position(Vec3::new(0.0, 0.0, 0.01));
    }
    pub fn camera_move_forward(&mut self) {
        self.transform.camera_position(Vec3::new(0.0, 0.0, -0.01));
    }
    pub fn camera_look_right(&mut self) {
        self.transform.camera_target(Vec3::new(0.01, 0.0, 0.0));
    }
    pub fn camera_look_left(&mut self) {
        self.transform.camera_target(Vec3::new(-0.01, 0.0, 0.0));
    }
    pub fn camera_look_up(&mut self) {
        self.transform.camera_target(Vec3::new(0.0, 0.01, 0.0));
    }
    pub fn camera_look_down(&mut self) {
        self.transform.camera_target(Vec3::new(0.0, -0.01, 0.0));
    }
    pub fn camera_look_back(&mut self) {
        self.transform.camera_target(Vec3::new(0.0, 0.0, 0.01));
    }
    pub fn camera_look_forward(&mut self) {
        self.transform.camera_target(Vec3::new(0.0, 0.0, -0.01));
    }
    pub fn camera_up_x_up(&mut self) {
        self.transform.camera_up(Vec3::new(0.01, 0.0, 0.0));
    }
    pub fn camera_up_x_down(&mut self) {
        self.transform.camera_up(Vec3::new(-0.01, 0.0, 0.0));
    }
    pub fn camera_up_y_up(&mut self) {
        self.transform.camera_up(Vec3::new(0.0, 0.01, 0.0));
    }
    pub fn camera_up_y_down(&mut self) {
        self.transform.camera_up(Vec3::new(0.0, -0.01, 0.0));
    }
    pub fn camera_up_z_up(&mut self) {
        self.transform.camera_up(Vec3::new(0.0, 0.0, 0.01));
    }
    pub fn camera_up_z_down(&mut self) {
        self.transform.camera_up(Vec3::new(0.0, 0.0, -0.01));
    }

    fn create_render_pass(
        renderer: &mut VulkanoWindowRenderer,
        basic_context: &VulkanoContext,
    ) -> Arc<RenderPass> {
        let pass = single_pass_renderpass!(
            basic_context.device().clone(),
            attachments: {
                color: {
                    format: renderer.swapchain_format(),
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

        renderer.add_additional_image_view(
            0,
            Format::D16_UNORM,
            ImageUsage::DEPTH_STENCIL_ATTACHMENT,
        );

        return pass;
    }

    fn create_graphics_pipeline(
        basic_context: &VulkanoContext,
        render_pass: Arc<RenderPass>,
    ) -> Arc<GraphicsPipeline> {
        let stages = [
            PipelineShaderStageCreateInfo::new(
                vs::load(basic_context.device().clone())
                    .unwrap()
                    .entry_point("main")
                    .unwrap(),
            ),
            PipelineShaderStageCreateInfo::new(
                fs::load(basic_context.device().clone())
                    .unwrap()
                    .entry_point("main")
                    .unwrap(),
            ),
        ]
        .to_vec()
        .into();

        let layout = PipelineLayout::new(
            basic_context.device().clone(),
            PipelineLayoutCreateInfo {
                set_layouts: vec![
                    DescriptorSetLayout::new(
                        basic_context.device().clone(),
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

        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

        let vertex_input_state = [Position::per_vertex()]
            .definition(
                &vs::load(basic_context.device().clone())
                    .unwrap()
                    .entry_point("main")
                    .unwrap(),
            )
            .unwrap();

        let create_info = GraphicsPipelineCreateInfo {
            stages: stages,
            rasterization_state: Some(RasterizationState {
                cull_mode: CullMode::Back,
                front_face: FrontFace::Clockwise,
                ..Default::default()
            }),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState::default()),
            multisample_state: Some(MultisampleState::default()),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState {
                    compare_op: CompareOp::Less,
                    write_enable: true,
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

        GraphicsPipeline::new(basic_context.device().clone(), None, create_info).unwrap()
    }

    fn create_vertex_buffers(basic_context: &VulkanoContext) -> Subbuffer<[Position]> {
        let buffers = Buffer::from_iter(
            basic_context.memory_allocator().clone(),
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
        return buffers;
    }

    fn create_index_buffer(basic_context: &VulkanoContext) -> Subbuffer<[u16]> {
        let index_buffer = Buffer::from_iter(
            basic_context.memory_allocator().clone(),
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
        return index_buffer;
    }

    fn create_uniform_buffers(basic_context: &VulkanoContext) -> SubbufferAllocator {
        SubbufferAllocator::new(
            basic_context.memory_allocator().clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        )
    }

    pub fn update_uniform_and_create_pass(
        &mut self,
        desc_alloc: Arc<StandardDescriptorSetAllocator>,
        window_ctx: &mut RenderContext,
        cb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        let uniform_buffer = self.update_uniform(
            window_ctx
                .window_ctx
                .get_renderer(window_ctx.id)
                .unwrap()
                .aspect_ratio(),
        );
        let layout = self.pipeline.layout().set_layouts()[0].clone();
        let descriptor_set = DescriptorSet::new(
            desc_alloc,
            layout,
            [WriteDescriptorSet::buffer(0, uniform_buffer)],
            [],
        )
        .unwrap();

        let image = window_ctx
            .window_ctx
            .get_renderer(window_ctx.id)
            .unwrap()
            .swapchain_image_view();

        let depth_view = window_ctx
            .window_ctx
            .get_renderer_mut(window_ctx.id)
            .unwrap()
            .get_additional_image_view(0);

        let framebuffer = Framebuffer::new(
            self.render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![image, depth_view],
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

        cb.begin_render_pass(render_pass_begin_info, Default::default())
            .unwrap();

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: window_ctx
                .window_ctx
                .get_renderer(window_ctx.id)
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
    }

    fn update_uniform(&self, aspect_ratio: f32) -> Subbuffer<Data> {
        let mvp = self.transform.compute_mvp(aspect_ratio);

        let uniform_data = Data {
            mvp: mvp.to_cols_array_2d(),
        };

        // Allocate a fresh subbuffer each frame - no synchronization needed!
        let subbuffer = self.uniform_allocator.allocate_sized().unwrap();
        *subbuffer.write().unwrap() = uniform_data;

        subbuffer
    }
}
