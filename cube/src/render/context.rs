use std::sync::Arc;
use vulkano::sync::GpuFuture;
use vulkano_util::context::VulkanoContext;
use vulkano_util::window::{VulkanoWindows, WindowDescriptor, WindowMode};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

pub struct RenderContext {
    pub window_ctx: VulkanoWindows,
    pub id: WindowId,
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

        Self { window_ctx, id }
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
