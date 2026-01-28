use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferToImageInfo,
        PrimaryCommandBufferAbstract, allocator::StandardCommandBufferAllocator,
    },
    device::DeviceOwned,
    format::Format,
    image::{
        sampler::{Sampler, SamplerAddressMode, SamplerCreateInfo, Filter},
        Image, ImageCreateInfo, ImageType, ImageUsage, view::ImageView,
    },
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    sync::GpuFuture,
};
use vulkano_util::context::VulkanoContext;

/// Create a texture image from raw pixel data
pub fn create_texture_image(
    context: &VulkanoContext,
    data: &[u8],
    width: u32,
    height: u32,
) -> Result<Arc<ImageView>, String> {
    let device = context.device();
    let queue = context.graphics_queue();
    let memory_allocator = context.memory_allocator();

    // Create command buffer allocator for this operation
    let cb_allocator = Arc::new(StandardCommandBufferAllocator::new(device.clone(), Default::default()));

    // Create staging buffer
    let staging_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        data.iter().copied(),
    )
    .map_err(|e| format!("Failed to create staging buffer: {}", e))?;

    // Create image
    let image = Image::new(
        memory_allocator.clone(),
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: Format::R8_UNORM, // Single-channel grayscale
            extent: [width, height, 1],
            usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
            ..Default::default()
        },
        AllocationCreateInfo::default(),
    )
    .map_err(|e| format!("Failed to create image: {}", e))?;

    // Create command buffer for transfer
    let mut builder = AutoCommandBufferBuilder::primary(
        cb_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .map_err(|e| format!("Failed to create command buffer: {}", e))?;

    builder
        .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
            staging_buffer,
            image.clone(),
        ))
        .map_err(|e| format!("Failed to copy buffer to image: {}", e))?;

    let command_buffer = builder
        .build()
        .map_err(|e| format!("Failed to build command buffer: {}", e))?;

    // Execute transfer
    command_buffer
        .execute(queue.clone())
        .map_err(|e| format!("Failed to execute transfer: {}", e))?
        .then_signal_fence_and_flush()
        .map_err(|e| format!("Failed to flush: {}", e))?
        .wait(None)
        .map_err(|e| format!("Failed to wait for transfer: {}", e))?;

    // Create image view
    let image_view = ImageView::new_default(image)
        .map_err(|e| format!("Failed to create image view: {}", e))?;

    Ok(image_view)
}

/// Create a sampler for texture sampling
pub fn create_sampler(allocator: &Arc<StandardMemoryAllocator>) -> Result<Arc<Sampler>, String> {
    // Get device from allocator using DeviceOwned trait
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
