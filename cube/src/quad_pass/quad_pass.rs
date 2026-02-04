use std::sync::Arc;

use glam::Mat4;
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
    image::{sampler::Sampler, view::ImageView},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
    pipeline::{
        DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::DepthStencilState,
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::{CullMode, RasterizationState},
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
        },
        layout::PipelineLayoutCreateInfo,
    },
    render_pass::{RenderPass, Subpass},
    shader::ShaderStages,
};
use vulkano_util::context::VulkanoContext;

use crate::quad_pass::{
    models::{QUAD_INDICES, QUAD_VERTICES, QuadVertex},
    shaders::{
        fs,
        vs::{self, Data},
    },
};
use std::path::Path;

use crate::texture::{create_sampler, create_texture_image, load_ppm};

pub struct QuadPass {
    pipeline: Arc<GraphicsPipeline>,
    vertex_buffer: Subbuffer<[QuadVertex]>,
    index_buffer: Subbuffer<[u16]>,
    texture_image: Arc<ImageView>,
    sampler: Arc<Sampler>,
    uniform_allocator: SubbufferAllocator,
}

impl QuadPass {
    pub fn new(
        basic_context: &VulkanoContext,
        render_pass: Arc<RenderPass>,
        atlas_path: impl AsRef<Path>,
    ) -> Self {
        // Load atlas texture
        let (pixel_data, width, height) = load_ppm(atlas_path).expect("Failed to load atlas PPM");
        let texture_image = create_texture_image(basic_context, &pixel_data, width, height)
            .expect("Failed to create texture image");

        let sampler =
            create_sampler(basic_context.memory_allocator()).expect("Failed to create sampler");

        let pipeline = Self::create_graphics_pipeline(basic_context, render_pass.clone());
        let vertex_buffer = Self::create_vertex_buffer(basic_context);
        let index_buffer = Self::create_index_buffer(basic_context);
        let uniform_allocator = Self::create_uniform_allocator(basic_context);

        QuadPass {
            pipeline,
            vertex_buffer,
            index_buffer,
            texture_image,
            sampler,
            uniform_allocator,
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

        // Create descriptor set layout with two bindings:
        // Binding 0: Uniform buffer (orthographic matrix)
        // Binding 1: Combined image sampler (texture)
        let layout = PipelineLayout::new(
            basic_context.device().clone(),
            PipelineLayoutCreateInfo {
                set_layouts: vec![
                    DescriptorSetLayout::new(
                        basic_context.device().clone(),
                        DescriptorSetLayoutCreateInfo {
                            bindings: [
                                (
                                    0,
                                    DescriptorSetLayoutBinding {
                                        stages: ShaderStages::VERTEX,
                                        ..DescriptorSetLayoutBinding::descriptor_type(
                                            DescriptorType::UniformBuffer,
                                        )
                                    },
                                ),
                                (
                                    1,
                                    DescriptorSetLayoutBinding {
                                        stages: ShaderStages::FRAGMENT,
                                        ..DescriptorSetLayoutBinding::descriptor_type(
                                            DescriptorType::CombinedImageSampler,
                                        )
                                    },
                                ),
                            ]
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

        let vertex_input_state = [QuadVertex::per_vertex()]
            .definition(
                &vs::load(basic_context.device().clone())
                    .unwrap()
                    .entry_point("main")
                    .unwrap(),
            )
            .unwrap();

        let create_info = GraphicsPipelineCreateInfo {
            stages,
            rasterization_state: Some(RasterizationState {
                cull_mode: CullMode::None, // No culling for 2D quad
                ..Default::default()
            }),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState::default()),
            multisample_state: Some(MultisampleState::default()),
            depth_stencil_state: Some(DepthStencilState {
                depth: None, // Disable depth testing for 2D overlay
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

    fn create_vertex_buffer(basic_context: &VulkanoContext) -> Subbuffer<[QuadVertex]> {
        Buffer::from_iter(
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
            QUAD_VERTICES,
        )
        .unwrap()
    }

    fn create_index_buffer(basic_context: &VulkanoContext) -> Subbuffer<[u16]> {
        Buffer::from_iter(
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
            QUAD_INDICES,
        )
        .unwrap()
    }

    fn create_uniform_allocator(basic_context: &VulkanoContext) -> SubbufferAllocator {
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

    fn create_orthographic_matrix(aspect_ratio: f32) -> Mat4 {
        let quad_width = 0.5;
        let atlas_aspect = 512.0 / 64.0;
        let quad_height = quad_width * aspect_ratio / atlas_aspect;

        let left = 0.5;
        let right = left + quad_width;
        let bottom = -1.0;
        let top = bottom + quad_height;

        let scale_x = (right - left) / 2.0;
        let scale_y = (top - bottom) / 2.0;
        let translate_x = (right + left) / 2.0;
        let translate_y = (top + bottom) / 2.0;

        Mat4::from_cols_array_2d(&[
            [scale_x, 0.0, 0.0, 0.0],
            [0.0, scale_y, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [translate_x, translate_y, 0.0, 1.0],
        ])
    }

    fn update_uniform(&self, aspect_ratio: f32) -> Subbuffer<Data> {
        let ortho = Self::create_orthographic_matrix(aspect_ratio);

        let uniform_data = Data {
            ortho: ortho.to_cols_array_2d(),
        };

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
        let uniform_buffer = self.update_uniform(aspect_ratio);
        let layout = self.pipeline.layout().set_layouts()[0].clone();

        // Create descriptor set with both uniform buffer and texture sampler
        let descriptor_set = DescriptorSet::new(
            desc_alloc,
            layout,
            [
                WriteDescriptorSet::buffer(0, uniform_buffer),
                WriteDescriptorSet::image_view_sampler(
                    1,
                    self.texture_image.clone(),
                    self.sampler.clone(),
                ),
            ],
            [],
        )
        .unwrap();

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent,
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
