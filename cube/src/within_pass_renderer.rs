use std::sync::Arc;

use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    descriptor_set::allocator::StandardDescriptorSetAllocator,
};
use winit::event::WindowEvent;

pub trait WithinPassRenderer {
    fn draw_within_pass(
        &mut self,
        desc_alloc: Arc<StandardDescriptorSetAllocator>,
        extent: [f32; 2],
        cb: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    );

    fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        false // Default: ignore all events
    }
}
