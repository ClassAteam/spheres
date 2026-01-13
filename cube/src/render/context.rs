use egui_winit_vulkano::{Gui, egui};
use std::sync::Arc;
use vulkano::buffer::Subbuffer;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, RenderPassBeginInfo,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::sync;
use vulkano::sync::GpuFuture;
use vulkano_util::context::VulkanoContext;
use vulkano_util::window::VulkanoWindows;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use super::builder::RenderContextBuilder;
use crate::control::TransformState;
use crate::counter::FpsCounter;
use crate::models::Position;
use crate::shaders::vs;

pub struct RenderContext {
    pub bctx: Arc<VulkanoContext>,
    pub window_ctx: VulkanoWindows,
    pub id: WindowId,
    pub render_pass: Arc<RenderPass>,
    pub pipeline: Arc<GraphicsPipeline>,
    pub vertex_buffer: Subbuffer<[Position]>,
    pub index_buffer: Subbuffer<[u16]>,
    pub uniform_buffers: Vec<Subbuffer<vs::Data>>,
    pub previous_frame_end: Option<Box<dyn GpuFuture>>,
    pub gui: Gui,
}

impl RenderContext {
    pub fn new(event_loop: &ActiveEventLoop, basic_cntx: Arc<VulkanoContext>) -> Self {
        RenderContextBuilder::new(event_loop, basic_cntx)
            .with_render_pass()
            .with_pipeline()
            .with_vertex_buffers()
            .with_index_buffer()
            .with_uniform_buffers()
            .build()
    }

    pub fn draw(
        &mut self,
        cb_alloc: Arc<StandardCommandBufferAllocator>,
        desc_alloc: Arc<StandardDescriptorSetAllocator>,
        transform: &TransformState,
        fps_counter: &FpsCounter,
    ) {
        let acquire_result = self
            .window_ctx
            .get_renderer_mut(self.id)
            .unwrap()
            .acquire(None, |_| {});

        let command_buffer = self.build_cmd_buf(cb_alloc, desc_alloc, transform);

        self.gui.immediate_ui(|gui| {
            let ctx = gui.context();
            egui::Window::new("Debug Info")
                .default_pos(egui::pos2(10.0, 10.0))
                .resizable(false)
                .show(&ctx, |ui| {
                    ui.label(format!("FPS: {:.1}", fps_counter.fps()));
                    ui.label(format!("Frame Time: {:.2} ms", fps_counter.frame_time_ms()));
                });
        });

        let acquire_future = match acquire_result {
            Ok(future) => future,

            Err(vulkano::VulkanError::OutOfDate) => {
                self.window_ctx.get_renderer_mut(self.id).unwrap().resize();
                return;
            }
            _ => {
                panic!("something went wrong with acquiring the swapchain index")
            }
        };

        let after_cube = acquire_future
            .join(self.previous_frame_end.take().unwrap())
            .then_execute(
                self.window_ctx
                    .get_renderer(self.id)
                    .unwrap()
                    .graphics_queue()
                    .clone(),
                command_buffer,
            )
            .unwrap();

        let final_future = self.gui.draw_on_image(
            after_cube,
            self.window_ctx
                .get_renderer(self.id)
                .unwrap()
                .swapchain_image_view(),
        );

        self.window_ctx
            .get_renderer_mut(self.id)
            .unwrap()
            .present(final_future, true);

        self.previous_frame_end = Some(sync::now(self.bctx.device().clone()).boxed());
    }

    pub fn build_cmd_buf(
        &mut self,
        cmb_alloc: Arc<StandardCommandBufferAllocator>,
        desc_mem_alloc: Arc<StandardDescriptorSetAllocator>,
        transform: &TransformState,
    ) -> Arc<PrimaryAutoCommandBuffer> {
        let uniform_buffer = self.update_uniform(transform);
        let layout = self.pipeline.layout().set_layouts()[0].clone();
        let descriptor_set = DescriptorSet::new(
            desc_mem_alloc,
            layout,
            [WriteDescriptorSet::buffer(0, uniform_buffer)],
            [],
        )
        .unwrap();
        let mut cb = AutoCommandBufferBuilder::primary(
            cmb_alloc,
            self.window_ctx
                .get_primary_renderer()
                .as_ref()
                .unwrap()
                .graphics_queue()
                .queue_family_index(),
            vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let image = self
            .window_ctx
            .get_renderer(self.id)
            .unwrap()
            .swapchain_image_view();

        let depth_view = self
            .window_ctx
            .get_renderer_mut(self.id)
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
            extent: self
                .window_ctx
                .get_renderer(self.id)
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

        let command_buffer = cb.build().unwrap();

        command_buffer
    }

    pub fn update_uniform(&self, transform: &TransformState) -> Subbuffer<vs::Data> {
        let aspect_ratio = self
            .window_ctx
            .get_renderer(self.id)
            .unwrap()
            .aspect_ratio();

        let mvp = transform.compute_mvp(aspect_ratio);

        let uniform_data = vs::Data {
            mvp: mvp.to_cols_array_2d(),
        };

        let buffer = self.uniform_buffers
            [self.window_ctx.get_renderer(self.id).unwrap().image_index() as usize]
            .clone();

        *buffer.write().unwrap() = uniform_data;

        buffer
    }
}
