use crate::atlas_creator::glyph::{GlyphData, Glyphs};
use ab_glyph::{FontArc, PxScale};

pub struct AtlasCreator {
    glyphs: Glyphs,
}

impl AtlasCreator {
    pub fn new() -> Self {
        let font_data = include_bytes!("./../../resources/FreeMono.ttf");
        let font = FontArc::try_from_slice(font_data).unwrap();

        let scale = PxScale::from(48.0);
        let glyphs = Glyphs::new(font, scale);
        AtlasCreator { glyphs }
    }

    pub fn glyphs(&self) -> &[GlyphData] {
        self.glyphs.get_glyphs()
    }
}
