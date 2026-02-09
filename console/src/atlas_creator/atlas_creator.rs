use crate::atlas_creator::glyph::{GlyphData, Glyphs};
use crate::atlas_creator::packer::Packer;
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

        let scale = PxScale::from(48.0);
        let glyphs = Glyphs::new(font, scale);
        let packer = Packer::new(glyphs.total_area());
        AtlasCreator { glyphs, packer }
    }

    pub fn create_atlas(self) -> Atlas {
        let glyphs = &self.glyphs.data;
        let image = self.packer.pack_to_atlas(glyphs);
        Atlas { image }
    }

    pub fn glyphs(&self) -> &[GlyphData] {
        self.glyphs.get_glyphs()
    }
}

pub struct Atlas {
    image: GrayImage,
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
