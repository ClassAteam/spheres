use std::collections::HashMap;

use crate::atlas_creator::glyph::{GlyphData, Glyphs};
use crate::atlas_creator::packer::{GlyphMetrics, Packer};
use ab_glyph::{FontArc, PxScale};
use image::GrayImage;

pub struct AtlasCreator {
    glyphs: Glyphs,
    packer: Packer,
}

impl AtlasCreator {
    pub fn new() -> Self {
        let font_data = include_bytes!("./../../resources/FreeMono.ttf");
        let font = FontArc::try_from_slice(font_data).unwrap();

        let scale = PxScale::from(30.0);
        let glyphs = Glyphs::new(font, scale);
        let total_area = glyphs.total_area();
        let packer = Packer::new(total_area);
        AtlasCreator { glyphs, packer }
    }

    pub fn create_atlas(self) -> Atlas {
        let glyphs = &self.glyphs.data;
        self.packer.pack_to_atlas(glyphs)
    }

    pub fn glyphs(&self) -> &[GlyphData] {
        self.glyphs.get_glyphs()
    }
}

pub struct Atlas {
    pub image: GrayImage,
    pub info: HashMap<char, GlyphMetrics>,
}

impl Atlas {
    pub fn write_to_file(&self) {
        self.image.save("output.png").unwrap();
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
