use console::atlas_creator::{Atlas, AtlasCreator, GlyphMetrics};
use std::collections::HashMap;
use std::sync::Arc;
use vulkano::image::sampler::Filter;
use vulkano::image::sampler::SamplerAddressMode;

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
    device::DeviceOwned,
    image::{
        sampler::{Sampler, SamplerCreateInfo},
        view::ImageView,
    },
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

use crate::text_renderer::{
    models::QuadVertex,
    shaders::{fs, vs},
};

pub struct TextRenderer {
    pipeline: Arc<GraphicsPipeline>,
    texture_image: Arc<ImageView>,
    sampler: Arc<Sampler>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    uniform_allocator: SubbufferAllocator,
    atlas: Atlas,
}

pub trait TextInfo {
    fn text(&self) -> String;
    fn place(&self) -> PixelPoint;
}

impl TextRenderer {
    pub fn new(basic_context: &VulkanoContext, render_pass: Arc<RenderPass>) -> Self {
        let mut atlas_creator = AtlasCreator::new();
        let atlas = atlas_creator.create_atlas();

        let texture_image = atlas_creator
            .create_vulkan_image(basic_context, &atlas)
            .unwrap();
        let sampler = Self::create_sampler(basic_context.memory_allocator())
            .expect("Failed to create sampler");

        let pipeline = Self::create_graphics_pipeline(basic_context, render_pass);
        let uniform_allocator = Self::create_uniform_allocator(basic_context);

        TextRenderer {
            pipeline,
            texture_image,
            sampler,
            memory_allocator: basic_context.memory_allocator().clone(),
            uniform_allocator,
            atlas: atlas,
        }
    }

    fn create_text_geometry(&self, string: String, starting_pixel: PixelPoint) -> TextGeometry {
        let atlas_info = &self.atlas.info;
        let mut creator = TextGeometryCreator::new(starting_pixel);
        creator.build(string, atlas_info)
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

    fn create_sampler(allocator: &Arc<StandardMemoryAllocator>) -> Result<Arc<Sampler>, String> {
        let device = allocator.device();

        Sampler::new(
            device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                address_mode: [SamplerAddressMode::ClampToEdge; 3],
                ..Default::default()
            },
        )
        .map_err(|e| format!("Failed to create sampler: {}", e))
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

    pub fn draw_within_pass(
        &mut self,
        desc_alloc: Arc<StandardDescriptorSetAllocator>,
        extent: [f32; 2],
        text_info: &dyn TextInfo,
        cb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        let text_geometry = self.create_text_geometry(text_info.text(), text_info.place());
        let vertex_buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            text_geometry.vertices,
        )
        .unwrap();

        let index_buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            text_geometry.indices,
        )
        .unwrap();

        let index_count = index_buffer.len() as u32;

        let uniform_buffer = self.update_uniform(extent);
        let layout = self.pipeline.layout().set_layouts()[0].clone();

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

        cb.bind_vertex_buffers(0, vertex_buffer).unwrap();
        cb.bind_index_buffer(index_buffer).unwrap();

        unsafe { cb.draw_indexed(index_count, 1, 0, 0, 0) }.unwrap();
    }
}

struct TextGeometry {
    pub vertices: Vec<QuadVertex>,
    pub indices: Vec<u16>,
}

#[derive(Copy, Clone)]
pub struct PixelPoint {
    pub x: f32,
    pub y: f32,
}

struct TextGeometryCreator {
    spacing: f32,
    current_point: PixelPoint,
    last_line_start: PixelPoint,
    last_height: f32,
    starting_pos: PixelPoint,
}

impl TextGeometryCreator {
    pub fn new(start: PixelPoint) -> Self {
        Self {
            current_point: start,
            last_line_start: start,
            last_height: 0.0,
            spacing: 2.0,
            starting_pos: start,
        }
    }

    pub fn move_right(&mut self, glyph_width: f32) {
        self.current_point.x += glyph_width + self.spacing;
    }

    fn new_rectangle(&mut self, glyph_width: f32, glyph_height: f32) -> PixelRectangle {
        let left = self.current_point.x;
        let top = self.current_point.y;
        let right = self.current_point.x + glyph_width;
        let bottom = self.current_point.y - glyph_height;
        self.move_right(glyph_width);
        PixelRectangle {
            left,
            top,
            right,
            bottom,
        }
    }

    fn start_newline(&mut self) {
        self.current_point.x = self.last_line_start.x;
        self.current_point.y = self.current_point.y - self.last_height;
    }

    fn build(&mut self, text: String, info: &HashMap<char, GlyphMetrics>) -> TextGeometry {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for ch in text.chars() {
            if ch == '\n' {
                self.start_newline();
                continue;
            }
            if let Some(glyph_metrics) = info.get(&ch) {
                let glyph_width = glyph_metrics.width as f32;
                let glyph_height = glyph_metrics.height as f32;
                self.last_height = self.last_height.max(glyph_height);
                let rectangle = self.new_rectangle(glyph_width, glyph_height);
                let first_vertex_index = vertices.len() as u16;
                let top_left = QuadVertex {
                    position: [rectangle.left, rectangle.top, 0.0],
                    uv: [glyph_metrics.uv_min.x, glyph_metrics.uv_min.y],
                };
                let top_right = QuadVertex {
                    position: [rectangle.right, rectangle.top, 0.0],
                    uv: [glyph_metrics.uv_max.x, glyph_metrics.uv_min.y],
                };
                let bottom_right = QuadVertex {
                    position: [rectangle.right, rectangle.bottom, 0.0],
                    uv: [glyph_metrics.uv_max.x, glyph_metrics.uv_max.y],
                };
                let bottom_left = QuadVertex {
                    position: [rectangle.left, rectangle.bottom, 0.0],
                    uv: [glyph_metrics.uv_min.x, glyph_metrics.uv_max.y],
                };
                vertices.push(top_left);
                vertices.push(top_right);
                vertices.push(bottom_right);
                vertices.push(bottom_left);

                let tl = first_vertex_index;
                let tr = first_vertex_index + 1;
                let br = first_vertex_index + 2;
                let bl = first_vertex_index + 3;

                indices.extend_from_slice(&[
                    tl, tr, br, // First triangle
                    br, bl, tl, // Second triangle
                ]);
            }
        }
        TextGeometry { vertices, indices }
    }
}

struct PixelRectangle {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}
