use std::collections::HashMap;
use std::sync::Arc;

use crate::atlas_creator::glyph::{GlyphData, Glyphs};
use crate::atlas_creator::packer::{GlyphMetrics, Packer};
use ab_glyph::{Font, FontArc, PxScale, ScaleFont};
use image::GrayImage;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferToImageInfo,
    PrimaryCommandBufferAbstract,
};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::sync::GpuFuture;
use vulkano_util::context::VulkanoContext;

pub struct AtlasCreator {
    glyphs: Glyphs,
    packer: Packer,
    line_height: f32,
}

impl AtlasCreator {
    pub fn new() -> Self {
        let font_data = include_bytes!("./../../resources/FreeMono.ttf");
        let font = FontArc::try_from_slice(font_data).unwrap();

        let scale = PxScale::from(30.0);
        let line_height = {
            let scaled = font.as_scaled(scale);
            scaled.ascent() - scaled.descent() + scaled.line_gap()
        };

        let glyphs = Glyphs::new(font, scale);
        let total_area = glyphs.total_area();
        let packer = Packer::new(total_area);
        AtlasCreator { glyphs, packer, line_height }
    }

    pub fn create_atlas(&mut self) -> Atlas {
        let glyphs = &self.glyphs.data;
        self.packer.pack_to_atlas(glyphs, self.line_height)
    }

    pub fn create_vulkan_image(
        &mut self,
        vulkan_ctx: &VulkanoContext,
        atlas: &Atlas,
    ) -> Result<Arc<ImageView>, String> {
        let width = atlas.image.width();
        let height = atlas.image.height();
        let device = vulkan_ctx.device();
        let queue = vulkan_ctx.graphics_queue();
        let memory_allocator = vulkan_ctx.memory_allocator();

        // Create command buffer allocator for this operation
        let cb_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

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
            atlas.image.iter().copied(),
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

    pub fn glyphs(&self) -> &[GlyphData] {
        self.glyphs.get_glyphs()
    }
}

pub struct Atlas {
    pub image: GrayImage,
    pub info: HashMap<char, GlyphMetrics>,
    pub ascent: f32,
    pub line_height: f32,
}

impl Atlas {
    pub fn write_to_file(&self) {
        self.image.save("output.png").unwrap();

        let mut lines = Vec::new();
        lines.push(format!(
            "Atlas: {}x{}\n",
            self.image.width(),
            self.image.height()
        ));

        let mut chars: Vec<char> = self.info.keys().copied().collect();
        chars.sort();

        for ch in chars {
            let m = &self.info[&ch];
            lines.push(format!(
                "'{ch}' (U+{:04X})  size={}x{}  uv_min=({:.4},{:.4})  uv_max=({:.4},{:.4})\n",
                ch as u32, m.width, m.height, m.uv_min.x, m.uv_min.y, m.uv_max.x, m.uv_max.y,
            ));
        }

        std::fs::write("output_glyphs.txt", lines.join("")).unwrap();
    }

    pub fn pixel_data(&self) -> &[u8] {
        self.image.as_raw()
    }

    pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }
}
