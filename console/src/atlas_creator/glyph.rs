use ab_glyph::{Font, FontArc, PxScale};
use image::{GrayImage, Luma};

pub struct GlyphData {
    ch: char,
    image: GrayImage,
}

impl GlyphData {
    pub fn character(&self) -> char {
        self.ch
    }

    pub fn image(&self) -> &GrayImage {
        &self.image
    }
}

pub struct Glyphs {
    pub data: Vec<GlyphData>,
}

impl Glyphs {
    pub fn new(font: FontArc, scale: PxScale) -> Self {
        let mut glyph_data = Vec::new();
        for ch in ' '..='~' {
            let glyph_id = font.glyph_id(ch);
            let glyph_scaled = glyph_id.with_scale(scale);

            if let Some(outlined) = font.outline_glyph(glyph_scaled) {
                let bounds = outlined.px_bounds();
                let width = bounds.width().ceil() as u32;
                let height = bounds.height().ceil() as u32;

                let mut image = GrayImage::new(width, height);
                outlined.draw(|x, y, coverage| {
                    // coverage is f32 in range [0.0, 1.0]
                    // Convert to u8 grayscale [0, 255]
                    let pixel_value = (coverage * 255.0) as u8;
                    image.put_pixel(x, y, Luma([pixel_value]));
                });

                glyph_data.push(GlyphData { ch, image });
            }
        }

        Glyphs { data: glyph_data }
    }

    pub fn get_glyphs(&self) -> &[GlyphData] {
        &self.data
    }

    pub fn total_area(&self) -> u32 {
        self.data
            .iter()
            .map(|g| g.image().width() * g.image().height())
            .sum()
    }
}
