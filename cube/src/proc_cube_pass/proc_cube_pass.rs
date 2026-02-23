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
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::{
    text_renderer::{PixelPoint, TextItem, TextRenderer},
    transform::TransformState,
    within_pass_renderer::WithinPassRenderer,
};

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
                [-1.0, 1.0, -1.0],
                [1.0, 1.0, -1.0],
                [1.0, -1.0, -1.0],
                [-1.0, -1.0, -1.0],
            ],
            Face::Front => [
                [-1.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [1.0, -1.0, 1.0],
                [-1.0, -1.0, 1.0],
            ],
            Face::Left => [
                [-1.0, 1.0, -1.0],
                [-1.0, -1.0, -1.0],
                [-1.0, -1.0, 1.0],
                [-1.0, 1.0, 1.0],
            ],
            Face::Right => [
                [1.0, 1.0, -1.0],
                [1.0, 1.0, 1.0],
                [1.0, -1.0, 1.0],
                [1.0, -1.0, -1.0],
            ],
            Face::Bottom => [
                [-1.0, 1.0, -1.0],
                [-1.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [1.0, 1.0, -1.0],
            ],
            Face::Top => [
                [-1.0, -1.0, -1.0],
                [1.0, -1.0, -1.0],
                [1.0, -1.0, 1.0],
                [-1.0, -1.0, 1.0],
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
    // Match the exact index pattern from the working CubePass
    vec![
        // Back face (standard pattern)
        0, 1, 2, 2, 3, 0,
        // Front face (flipped pattern for opposite-facing face)
        4, 6, 5, 4, 7, 6,
        // Left face (standard pattern)
        8, 9, 10, 10, 11, 8,
        // Right face (standard pattern)
        12, 13, 14, 14, 15, 12,
        // Bottom face (standard pattern)
        16, 17, 18, 18, 19, 16,
        // Top face (standard pattern)
        20, 21, 22, 22, 23, 20,
    ]
}

pub struct ProcCubePass {
    pipeline: Arc<GraphicsPipeline>,
    vertex_buffer: Subbuffer<[Position]>,
    index_buffer: Subbuffer<[u16]>,
    transform: TransformState,
    uniform_allocator: SubbufferAllocator,
    text_renderer: TextRenderer,
    auto_rotate: bool,
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
            auto_rotate: true,
            text_renderer: TextRenderer::new(basic_context, render_pass),
        }
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

    fn text_items(&self) -> Vec<TextItem> {
        vec![TextItem {
            text: format!("{:#?}", self.transform),
            place: PixelPoint { x: 0.0, y: 0.0 },
        }]
    }
}
impl WithinPassRenderer for ProcCubePass {
    fn draw_within_pass(
        &mut self,
        desc_alloc: Arc<StandardDescriptorSetAllocator>,
        extent: [f32; 2],
        cb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        if self.auto_rotate {
            self.transform.rotate_model(Vec3::new(0.005, 0.01, 0.0));
        }

        let text_info = self.text_items();

        self.text_renderer
            .draw_within_pass(desc_alloc.clone(), extent, text_info, cb);

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

    fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        if let WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    physical_key: PhysicalKey::Code(key_code),
                    state: ElementState::Pressed,
                    ..
                },
            ..
        } = event
        {
            match key_code {
                KeyCode::KeyR => {
                    self.auto_rotate = !self.auto_rotate;
                    println!(
                        "Auto-rotation: {}",
                        if self.auto_rotate { "ON" } else { "OFF" }
                    );
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    fn name(&self) -> &str {
        "ProcCubePass (Procedural)"
    }
}
