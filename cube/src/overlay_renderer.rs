use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    render_pass::RenderPass,
};
use vulkano_util::context::VulkanoContext;
use winit::event::WindowEvent;

use crate::{
    counter::FpsCounter,
    renderer_pool::RendererPool,
    text_renderer::{PixelPoint, TextInfo, TextItem, TextRenderer},
    within_pass_renderer::WithinPassRenderer,
};

pub struct OverlayRenderer {
    text_renderer: TextRenderer,
    renderers_pool: Rc<RefCell<RendererPool>>,
    fps_counter: Rc<RefCell<FpsCounter>>,
}

impl OverlayRenderer {
    pub fn new(
        basic_context: &VulkanoContext,
        render_pass: Arc<RenderPass>,
        renderers_pool: Rc<RefCell<RendererPool>>,
        fps_counter: Rc<RefCell<FpsCounter>>,
    ) -> Self {
        Self {
            text_renderer: TextRenderer::new(basic_context, render_pass),
            renderers_pool,
            fps_counter,
        }
    }

    fn text_items(&self) -> Vec<TextItem> {
        let fps_items = self.fps_counter.borrow().text_items();

        let active_renderer_name = {
            let pool = self.renderers_pool.borrow();
            pool.active_ref().name().to_string()
        };

        let active_renderer_item = TextItem {
            text: format!("Active Renderer: {}", active_renderer_name),
            place: PixelPoint { x: 0.0, y: 1410.0 },
        };

        let mut items = fps_items;
        items.push(active_renderer_item);
        items
    }
}

impl WithinPassRenderer for OverlayRenderer {
    fn draw_within_pass(
        &mut self,
        desc_alloc: Arc<StandardDescriptorSetAllocator>,
        extent: [f32; 2],
        cb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        let text = self.text_items();
        self.text_renderer
            .draw_within_pass(desc_alloc, extent, text, cb);
    }

    fn handle_window_event(&mut self, _event: &WindowEvent) -> bool {
        // Overlay doesn't handle any events
        false
    }

    fn name(&self) -> &str {
        "OverlayRenderer (HUD)"
    }
}
