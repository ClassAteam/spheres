use std::sync::Arc;

use glam::Vec3;
use vulkano::{
    buffer::{
        Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer,
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

use crate::{transform::TransformState, within_pass_renderer::WithinPassRenderer};

use crate::proc_cube_pass::shaders::{
    fs,
    vs::{self, Data},
};

#[derive(BufferContents, Vertex, Debug, Clone, Copy)]
#[repr(C)]
pub struct Position {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub color: [f32; 3],
}

/// Face orientation for procedural cube generation
#[derive(Debug, Clone, Copy)]
enum Face {
    Back,   // -Z
    Front,  // +Z
    Left,   // -X
    Right,  // +X
    Bottom, // +Y
    Top,    // -Y
}

impl Face {
    fn vertices(&self) -> [[f32; 3]; 4] {
        match self {
            Face::Back => [
                [-1.0, 1.0, -1.0],  // Top-left
                [1.0, 1.0, -1.0],   // Top-right
                [1.0, -1.0, -1.0],  // Bottom-right
                [-1.0, -1.0, -1.0], // Bottom-left
            ],
            Face::Front => [
                [-1.0, 1.0, 1.0],  // Top-left
                [1.0, 1.0, 1.0],   // Top-right
                [1.0, -1.0, 1.0],  // Bottom-right
                [-1.0, -1.0, 1.0], // Bottom-left
            ],
            Face::Left => [
                [-1.0, 1.0, -1.0],  // Top-back
                [-1.0, -1.0, -1.0], // Bottom-back
                [-1.0, -1.0, 1.0],  // Bottom-front
                [-1.0, 1.0, 1.0],   // Top-front
            ],
            Face::Right => [
                [1.0, 1.0, -1.0],  // Top-back
                [1.0, 1.0, 1.0],   // Top-front
                [1.0, -1.0, 1.0],  // Bottom-front
                [1.0, -1.0, -1.0], // Bottom-back
            ],
            Face::Bottom => [
                [-1.0, 1.0, -1.0], // Back-left
                [-1.0, 1.0, 1.0],  // Front-left
                [1.0, 1.0, 1.0],   // Front-right
                [1.0, 1.0, -1.0],  // Back-right
            ],
            Face::Top => [
                [-1.0, -1.0, -1.0], // Back-left
                [1.0, -1.0, -1.0],  // Back-right
                [1.0, -1.0, 1.0],   // Front-right
                [-1.0, -1.0, 1.0],  // Front-left
            ],
        }
    }

    fn color(&self) -> [f32; 3] {
        match self {
            Face::Back => [1.0, 0.0, 0.0],   // Red
            Face::Front => [0.0, 1.0, 0.0],  // Green
            Face::Left => [0.0, 0.0, 1.0],   // Blue
            Face::Right => [1.0, 1.0, 0.0],  // Yellow
            Face::Bottom => [1.0, 0.0, 1.0], // Magenta
            Face::Top => [0.0, 1.0, 1.0],    // Cyan
        }
    }
}

/// Generate procedural cube vertices
fn generate_cube_vertices() -> Vec<Position> {
    let faces = [
        Face::Back,
        Face::Front,
        Face::Left,
        Face::Right,
        Face::Bottom,
        Face::Top,
    ];

    let mut vertices = Vec::with_capacity(24); // 6 faces × 4 vertices

    for face in &faces {
        let color = face.color();
        let positions = face.vertices();

        for position in &positions {
            vertices.push(Position {
                position: *position,
                color,
            });
        }
    }

    vertices
}

/// Generate procedural cube indices
fn generate_cube_indices() -> Vec<u16> {
    let mut indices = Vec::with_capacity(36); // 6 faces × 2 triangles × 3 vertices

    // Generate indices for all 6 faces
    for face_index in 0..6 {
        let base = (face_index * 4) as u16;

        // First triangle (0, 1, 2)
        indices.push(base);
        indices.push(base + 1);
        indices.push(base + 2);

        // Second triangle (2, 3, 0)
        indices.push(base + 2);
        indices.push(base + 3);
        indices.push(base);
    }

    indices
}

pub struct ProcCubePass {
    pipeline: Arc<GraphicsPipeline>,
    vertex_buffer: Subbuffer<[Position]>,
    index_buffer: Subbuffer<[u16]>,
    transform: TransformState,
    uniform_allocator: SubbufferAllocator,
    aspect_ratio: f32,
}

impl ProcCubePass {
    pub fn new(basic_context: &VulkanoContext, render_pass: Arc<RenderPass>) -> Self {
        let pipeline = Self::create_graphics_pipeline(basic_context, render_pass.clone());
        let vertex_buffer = Self::create_vertex_buffers(basic_context);
        let index_buffer = Self::create_index_buffer(basic_context);
        let uniform_allocator = Self::create_uniform_buffers(basic_context);
        ProcCubePass {
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
        let vertices = generate_cube_vertices();
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
            vertices,
        )
        .unwrap();
        return buffers;
    }

    fn create_index_buffer(basic_context: &VulkanoContext) -> Subbuffer<[u16]> {
        let indices = generate_cube_indices();
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
            indices,
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
}
impl WithinPassRenderer for ProcCubePass {
    fn draw_within_pass(
        &mut self,
        desc_alloc: Arc<StandardDescriptorSetAllocator>,
        extent: [f32; 2],
        cb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        let aspect_ratio = extent[0] / extent[1];
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
