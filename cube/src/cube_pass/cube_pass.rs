use std::sync::Arc;

use glam::Vec3;
use vulkano::{
    buffer::{
        Buffer, BufferCreateInfo, BufferUsage, Subbuffer,
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
    },
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    descriptor_set::{
        DescriptorSet, WriteDescriptorSet,
        allocator::StandardDescriptorSetAllocator,
        layout::{
            DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo,
            DescriptorType,
        },
    },
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
    render_pass::{RenderPass, Subpass},
    shader::ShaderStages,
};
use vulkano_util::context::VulkanoContext;

use crate::{
    cube_pass::{
        models::{INDICES, POSITIONS, Position},
        shaders::{
            fs,
            vs::{self, Data},
        },
        transform::TransformState,
    },
    text_renderer::{PixelPoint, TextInfo, TextItem},
};

pub struct CubePass {
    pipeline: Arc<GraphicsPipeline>,
    vertex_buffer: Subbuffer<[Position]>,
    index_buffer: Subbuffer<[u16]>,
    transform: TransformState,
    uniform_allocator: SubbufferAllocator,
    aspect_ratio: f32,
}

impl CubePass {
    pub fn new(basic_context: &VulkanoContext, render_pass: Arc<RenderPass>) -> Self {
        let pipeline = Self::create_graphics_pipeline(basic_context, render_pass.clone());
        let vertex_buffer = Self::create_vertex_buffers(basic_context);
        let index_buffer = Self::create_index_buffer(basic_context);
        let uniform_allocator = Self::create_uniform_buffers(basic_context);
        CubePass {
            pipeline,
            vertex_buffer,
            index_buffer,
            transform: TransformState::new(),
            uniform_allocator,
            aspect_ratio: 1.0,
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

    pub fn draw_within_pass(
        &mut self,
        aspect_ratio: f32,
        desc_alloc: Arc<StandardDescriptorSetAllocator>,
        extent: [f32; 2],
        cb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        self.aspect_ratio = aspect_ratio;
        let uniform_buffer = self.update_uniform(aspect_ratio);
        let layout = self.pipeline.layout().set_layouts()[0].clone();
        let descriptor_set = DescriptorSet::new(
            desc_alloc,
            layout,
            [WriteDescriptorSet::buffer(0, uniform_buffer)],
            [],
        )
        .unwrap();

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: extent,
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
    }
}

impl TextInfo for CubePass {
    fn text_items(&self) -> Vec<TextItem> {
        let mut vertices_text = String::from("Vertices (Transformed):\n");
        for (i, vertex) in POSITIONS.iter().enumerate() {
            let t = self
                .transform
                .transform_vertex(vertex.position, self.aspect_ratio);
            vertices_text.push_str(&format!(
                "[{}] clip: [{:.3}, {:.3}, {:.3}, {:.3}] -> ndc: [{:.3}, {:.3}, {:.3}]\n",
                i,
                t.clip_space[0],
                t.clip_space[1],
                t.clip_space[2],
                t.clip_space[3],
                t.ndc[0],
                t.ndc[1],
                t.ndc[2],
            ));
        }
        vec![
            TextItem {
                text: format!("Transform state:{:#?}", self.transform),
                place: PixelPoint { x: 0.0, y: 0.0 },
            },
            TextItem {
                text: vertices_text,
                place: PixelPoint { x: 2100.0, y: 0.0 },
            },
        ]
    }
}
