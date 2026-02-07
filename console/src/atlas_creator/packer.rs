use image::GrayImage;

use crate::atlas_creator::glyph::GlyphData;

pub struct Packer {
    image: GrayImage,
    cursor: Cursor,
}

#[derive(Debug)]
struct AtlasDimensions {
    x: u32,
    y: u32,
}

impl Packer {
    pub fn new(area: u32) -> Self {
        let dimensions = AtlasDimensions::new(area);
        let image = GrayImage::new(dimensions.x, dimensions.y);
        Self {
            image,
            cursor: Cursor {
                atlas_width: (dimensions.x),
                atlas_height: (dimensions.y),
                x_pos: (0),
                y_pos: (0),
                padding: (1),
                row_height: (0),
            },
        }
    }

    fn write_glyph(&mut self, glyph: &GlyphData, start: GlyphStart) -> LastGlyphEnd {
        for (x, y, pixel) in glyph.image().enumerate_pixels() {
            self.image
                .put_pixel(start.x_pos + x, start.y_pos + y, *pixel);
        }
        LastGlyphEnd {
            width: glyph.image().width(),
            height: glyph.image().height(),
        }
    }

    pub fn pack_to_atlas(mut self, glyphs: &[GlyphData]) -> GrayImage {
        for glyph in glyphs {
            let start = self.cursor.next_glyph_start(glyph.image().width());
            let end = self.write_glyph(&glyph, start);
            self.cursor.advance(end);
        }

        self.image
    }
}

impl AtlasDimensions {
    pub fn new(total_area: u32) -> Self {
        let with_overhead = (total_area as f32 * 1.5).ceil() as u32;

        // Start with a reasonable width
        let width = 1024u32
            .max((with_overhead as f32).sqrt().ceil() as u32)
            .next_power_of_two();

        // Calculate minimum height needed
        let height = ((with_overhead as f32 / width as f32).ceil() as u32).next_power_of_two();

        AtlasDimensions {
            x: width,
            y: height,
        }
    }
}

#[derive(Debug)]
struct Cursor {
    atlas_width: u32,
    atlas_height: u32,
    x_pos: u32,
    y_pos: u32,
    padding: u32,
    row_height: u32,
}

#[derive(Debug)]
struct GlyphStart {
    x_pos: u32,
    y_pos: u32,
}

struct LastGlyphEnd {
    width: u32,
    height: u32,
}

impl Cursor {
    pub fn next_glyph_start(&mut self, glyph_width: u32) -> GlyphStart {
        if self.is_out_of_width(glyph_width) {
            self.move_to_next_row()
        } else {
            self.add_x_padding()
        }
    }

    pub fn advance(&mut self, last_dim: LastGlyphEnd) {
        self.x_pos += last_dim.width;
        self.row_height = self.row_height.max(last_dim.height);
    }

    fn add_x_padding(&mut self) -> GlyphStart {
        self.x_pos += self.padding;
        GlyphStart {
            x_pos: self.x_pos,
            y_pos: self.y_pos,
        }
    }

    fn is_out_of_width(&self, glyph_width: u32) -> bool {
        if self.x_pos + glyph_width > self.atlas_width {
            true
        } else {
            false
        }
    }

    fn move_to_next_row(&mut self) -> GlyphStart {
        self.y_pos += self.row_height + self.padding;
        self.x_pos = 0;
        self.row_height = 0;
        GlyphStart {
            x_pos: self.x_pos,
            y_pos: self.y_pos,
        }
    }
}
