use console::atlas_creator::{self, AtlasCreator};
use std::path::Path;
use std::sync::Arc;

use vulkano::{
    buffer::{
        Buffer, BufferCreateInfo, BufferUsage,
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
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{
                AttachmentBlend, BlendFactor, BlendOp, ColorBlendAttachmentState, ColorBlendState,
            },
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
    models::QuadVertex,
    shaders::{fs, vs},
};
use crate::texture::{create_atlas_texture, create_sampler, load_ppm};

pub struct QuadPass {
    pipeline: Arc<GraphicsPipeline>,
    texture_image: Arc<ImageView>,
    sampler: Arc<Sampler>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    uniform_allocator: SubbufferAllocator,
    text: String,
}

impl QuadPass {
    pub fn new(
        basic_context: &VulkanoContext,
        render_pass: Arc<RenderPass>,
        font_path: impl AsRef<Path>,
    ) -> Self {
        let atlas_creator = AtlasCreator::new();
        let atlas = atlas_creator.create_atlas();
        let pixel_data = atlas.pixel_data();
        let atlas_width = atlas.width();
        let atlas_height = atlas.height();

        let texture_image =
            create_atlas_texture(basic_context, pixel_data, atlas_width, atlas_height)
                .expect("Failed to create texture image");
        let sampler =
            create_sampler(basic_context.memory_allocator()).expect("Failed to create sampler");

        let pipeline = Self::create_graphics_pipeline(basic_context, render_pass);
        let uniform_allocator = Self::create_uniform_allocator(basic_context);

        QuadPass {
            pipeline,
            texture_image,
            sampler,
            memory_allocator: basic_context.memory_allocator().clone(),
            uniform_allocator,
            text: String::from("HELLO WORLD"),
        }
    }

    #[allow(dead_code)]
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
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

        let subpass = Subpass::from(render_pass, 0).unwrap();

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
                cull_mode: CullMode::None,
                ..Default::default()
            }),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState::default()),
            multisample_state: Some(MultisampleState::default()),
            depth_stencil_state: Some(DepthStencilState {
                depth: None,
                ..Default::default()
            }),
            subpass: Some(subpass.into()),
            dynamic_state: [DynamicState::Viewport].into_iter().collect(),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                1,
                ColorBlendAttachmentState {
                    blend: Some(AttachmentBlend {
                        color_blend_op: BlendOp::Add,
                        src_color_blend_factor: BlendFactor::SrcAlpha,
                        dst_color_blend_factor: BlendFactor::OneMinusSrcAlpha,
                        alpha_blend_op: BlendOp::Add,
                        src_alpha_blend_factor: BlendFactor::One,
                        dst_alpha_blend_factor: BlendFactor::Zero,
                    }),
                    ..Default::default()
                },
            )),
            ..GraphicsPipelineCreateInfo::layout(layout)
        };

        GraphicsPipeline::new(basic_context.device().clone(), None, create_info).unwrap()
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

    fn update_uniform(&self, extent: [f32; 2]) -> vulkano::buffer::Subbuffer<vs::Data> {
        let uniform_data = vs::Data {
            screen_size: extent,
        };

        let subbuffer = self.uniform_allocator.allocate_sized().unwrap();
        *subbuffer.write().unwrap() = uniform_data;
        subbuffer
    }

    // pub fn draw_within_pass(
    //     &mut self,
    //     _aspect_ratio: f32,
    //     desc_alloc: Arc<StandardDescriptorSetAllocator>,
    //     extent: [f32; 2],
    //     cb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    // ) {
    //     // Build per-frame vertex and index buffers from the laid-out glyphs
    //     let vertex_buffer = Buffer::from_iter(
    //         self.memory_allocator.clone(),
    //         BufferCreateInfo {
    //             usage: BufferUsage::VERTEX_BUFFER,
    //             ..Default::default()
    //         },
    //         AllocationCreateInfo {
    //             memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
    //                 | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
    //             ..Default::default()
    //         },
    //         verts,
    //     )
    //     .unwrap();

    //     let index_buffer = Buffer::from_iter(
    //         self.memory_allocator.clone(),
    //         BufferCreateInfo {
    //             usage: BufferUsage::INDEX_BUFFER,
    //             ..Default::default()
    //         },
    //         AllocationCreateInfo {
    //             memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
    //                 | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
    //             ..Default::default()
    //         },
    //         idxs,
    //     )
    //     .unwrap();

    //     let index_count = index_buffer.len() as u32;

    //     let uniform_buffer = self.update_uniform(extent);
    //     let layout = self.pipeline.layout().set_layouts()[0].clone();

    //     let descriptor_set = DescriptorSet::new(
    //         desc_alloc,
    //         layout,
    //         [
    //             WriteDescriptorSet::buffer(0, uniform_buffer),
    //             WriteDescriptorSet::image_view_sampler(
    //                 1,
    //                 self.texture_image.clone(),
    //                 self.sampler.clone(),
    //             ),
    //         ],
    //         [],
    //     )
    //     .unwrap();

    //     let viewport = Viewport {
    //         offset: [0.0, 0.0],
    //         extent,
    //         depth_range: 0.0..=1.0,
    //     };
    //     cb.set_viewport(0, [viewport].into_iter().collect())
    //         .unwrap();

    //     cb.bind_pipeline_graphics(self.pipeline.clone()).unwrap();

    //     cb.bind_descriptor_sets(
    //         PipelineBindPoint::Graphics,
    //         self.pipeline.layout().clone(),
    //         0,
    //         descriptor_set,
    //     )
    //     .unwrap();

    //     cb.bind_vertex_buffers(0, vertex_buffer).unwrap();
    //     cb.bind_index_buffer(index_buffer).unwrap();

    //     unsafe { cb.draw_indexed(index_count, 1, 0, 0, 0) }.unwrap();
    // }
}
