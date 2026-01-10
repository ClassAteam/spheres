use std::sync::Arc;
use vulkano::descriptor_set::layout::{
    DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorType,
};
use vulkano::device::Device;
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::depth_stencil::{CompareOp, DepthState, DepthStencilState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::{CullMode, FrontFace, RasterizationState};
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineLayoutCreateInfo;
use vulkano::pipeline::{DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::render_pass::{RenderPass, Subpass};
use vulkano::shader::ShaderStages;

use crate::models::Position;
use crate::shaders::{fs, vs};

pub fn create_graphics_pipeline(
    device: Arc<Device>,
    render_pass: Arc<RenderPass>,
) -> Arc<GraphicsPipeline> {
    let stages = [
        PipelineShaderStageCreateInfo::new(
            vs::load(device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap(),
        ),
        PipelineShaderStageCreateInfo::new(
            fs::load(device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap(),
        ),
    ]
    .to_vec()
    .into();

    let layout = PipelineLayout::new(
        device.clone(),
        PipelineLayoutCreateInfo {
            set_layouts: vec![DescriptorSetLayout::new(
                device.clone(),
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
            .unwrap()],
            ..Default::default()
        },
    )
    .unwrap();

    let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

    let vertex_input_state = [Position::per_vertex()]
        .definition(
            &vs::load(device.clone())
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

    GraphicsPipeline::new(device.clone(), None, create_info).unwrap()
}
