use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano_util::context::{VulkanoConfig, VulkanoContext};

pub struct VulkanBasicContext {
    pub bctx: Arc<VulkanoContext>,
    pub cb_alloc: Arc<StandardCommandBufferAllocator>,
    pub descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl VulkanBasicContext {
    pub fn new() -> Self {
        let bctx = Arc::new(VulkanoContext::new(VulkanoConfig::default()));
        let cb_alloc = Arc::new(StandardCommandBufferAllocator::new(
            bctx.device().clone(),
            Default::default(),
        ));

        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            bctx.device().clone(),
            Default::default(),
        ));
        VulkanBasicContext {
            bctx,
            cb_alloc,
            descriptor_set_allocator,
        }
    }
}
