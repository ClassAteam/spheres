use std::sync::Arc;
use vulkano::{
    command_buffer::allocator::StandardCommandBufferAllocator,
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
        physical::{PhysicalDevice, PhysicalDeviceType},
    },
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    library::VulkanLibrary,
    memory::allocator::StandardMemoryAllocator,
    swapchain::Surface,
};

use winit::{event_loop::EventLoop, window::Window};

// No longer need vertex data - using procedural generation

use crate::window_context::WindowDependentContext;

pub struct RenderContext {
    queue: Arc<Queue>,
    instance: Arc<Instance>,
    device: Arc<Device>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    window_context: Option<WindowDependentContext>,
}

impl RenderContext {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let library = VulkanLibrary::new().unwrap();
        let required_extensions = Surface::required_extensions(event_loop).unwrap();
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: required_extensions,
                ..Default::default()
            },
        )
        .unwrap();

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) =
            Self::pick_physical_device_and_queue(instance.clone(), &device_extensions, event_loop);

        println!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        let (device, queue) = Self::create_device_and_one_queue(
            physical_device,
            queue_family_index,
            device_extensions,
        );

        let _memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        Self {
            instance,
            device,
            queue,
            command_buffer_allocator,
            window_context: None,
        }
    }

    fn create_device_and_one_queue(
        physical_device: Arc<PhysicalDevice>,
        queue_family_index: u32,
        device_extensions: DeviceExtensions,
    ) -> (Arc<Device>, Arc<Queue>) {
        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: [QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }]
                .to_vec(),
                ..Default::default()
            },
        )
        .unwrap();

        let queue = queues.next().unwrap();

        (device, queue)
    }

    fn pick_physical_device_and_queue(
        instance: Arc<Instance>,
        extensions: &DeviceExtensions,
        event_loop: &EventLoop<()>,
    ) -> (Arc<PhysicalDevice>, u32) {
        instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| p.supported_extensions().contains(extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && p.presentation_support(i as u32, event_loop).unwrap()
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .unwrap()
    }


    pub fn resumed(&mut self, window: Arc<Window>) {
        self.window_context = Some(WindowDependentContext::new(
            self.instance.clone(),
            window,
            self.device.clone(),
        ));
    }

    pub fn window_invalidated(&mut self) {
        if let Some(ref mut window_context) = self.window_context {
            window_context.swapchain_recreation_needed();
        }
    }

    pub fn draw(&mut self) {
        if let Some(ref mut window_context) = self.window_context {
            window_context.redraw(
                self.device.clone(),
                self.command_buffer_allocator.clone(),
                self.queue.clone(),
            );
        }
    }

    pub fn request_redraw(&mut self) {
        self.window_context
            .as_mut()
            .unwrap()
            .window
            .request_redraw();
    }

    pub fn rotate_up(&mut self) {
        self.window_context.as_mut().unwrap().change_angle_up();
        self.request_redraw();
    }

    pub fn rotate_down(&mut self) {
        self.window_context.as_mut().unwrap().change_angle_down();
        self.request_redraw();
    }
    
    pub fn rotate_left(&mut self) {
        self.window_context.as_mut().unwrap().change_angle_left();
        self.request_redraw();
    }
    
    pub fn rotate_right(&mut self) {
        self.window_context.as_mut().unwrap().change_angle_right();
        self.request_redraw();
    }
}
