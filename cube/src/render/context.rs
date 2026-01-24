use std::sync::Arc;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo};
use vulkano::memory::allocator::MemoryTypeFilter;
use vulkano::sync::GpuFuture;
use vulkano_util::context::VulkanoContext;
use vulkano_util::window::{VulkanoWindows, WindowDescriptor, WindowMode};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

pub struct RenderContext {
    pub window_ctx: VulkanoWindows,
    pub id: WindowId,
    pub uniform_allocator: SubbufferAllocator,
}

impl RenderContext {
    pub fn new(event_loop: &ActiveEventLoop, basic_cntx: Arc<VulkanoContext>) -> Self {
        let mut window_ctx = VulkanoWindows::default();
        let window_descr = WindowDescriptor {
            title: "Cube".to_string(),
            mode: WindowMode::BorderlessFullscreen,
            ..Default::default()
        };
        let id = window_ctx.create_window(event_loop, &basic_cntx, &window_descr, |_| {});

        let uniform_allocator = SubbufferAllocator::new(
            basic_cntx.memory_allocator().clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        );

        Self {
            window_ctx,
            id,
            uniform_allocator,
        }
    }

    pub fn acquire(&mut self) -> Box<dyn GpuFuture> {
        loop {
            let acquire_result = self
                .window_ctx
                .get_renderer_mut(self.id)
                .unwrap()
                .acquire(None, |_| {});

            match acquire_result {
                Ok(future) => return future,

                Err(vulkano::VulkanError::OutOfDate) => {
                    self.window_ctx.get_renderer_mut(self.id).unwrap().resize();
                }
                Err(e) => {
                    panic!("failed to acquire swapchain image: {:?}", e)
                }
            }
        }
    }

    pub fn present(&mut self, til_present_future: Box<dyn GpuFuture>) {
        self.window_ctx
            .get_renderer_mut(self.id)
            .unwrap()
            .present(til_present_future, false);
    }
}
