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
        let cursor = Cursor::new(&dimensions);
        Self { image, cursor }
    }

    fn write_glyph(&mut self, glyph: &GlyphData, start: &GlyphStart) -> GlyphEnd {
        for (x, y, pixel) in glyph.image().enumerate_pixels() {
            self.image
                .put_pixel(start.x_pos + x, start.top_row_y + y, *pixel);
        }
        GlyphEnd {
            x_pos: start.x_pos + glyph.image().width(),
            y_pos: start.top_row_y + glyph.image().height(),
            height: glyph.image().height(),
        }
    }

    pub fn pack_to_atlas(mut self, glyphs: &[GlyphData]) -> GrayImage {
        for glyph in glyphs {
            let start = self
                .cursor
                .next_glyph_start(glyph.image().width(), glyph.image().height());
            let end = self.write_glyph(&glyph, &start);
            self.cursor.advance(start, end);
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
    last_written: LastWrittenEnd,
    padding: u32,
    current_top_row_y: u32,
    current_row_height: u32,
}

#[derive(Debug)]
struct LastWrittenEnd {
    x_pos: u32,
    y_pos: u32,
}

#[derive(Debug)]
struct GlyphStart {
    x_pos: u32,
    top_row_y: u32,
}

#[derive(Debug)]
struct GlyphEnd {
    x_pos: u32,
    y_pos: u32,
    height: u32,
}

impl Cursor {
    pub fn new(dimensions: &AtlasDimensions) -> Self {
        Cursor {
            atlas_width: (dimensions.x),
            atlas_height: (dimensions.y),
            last_written: LastWrittenEnd {
                x_pos: (0),
                y_pos: (0),
            },
            padding: (10),
            current_top_row_y: (0),
            current_row_height: (0),
        }
    }

    pub fn next_glyph_start(&self, glyph_width: u32, glyph_height: u32) -> GlyphStart {
        if self.is_out_of_width(glyph_width) {
            self.move_to_next_row(glyph_height)
        } else {
            self.add_x_padding()
        }
    }

    pub fn advance(&mut self, start_pos: GlyphStart, ending_pos: GlyphEnd) {
        self.last_written.x_pos = ending_pos.x_pos;
        self.last_written.y_pos = ending_pos.y_pos;
        self.current_row_height = self.current_row_height.max(ending_pos.height);
        self.current_top_row_y = self.current_top_row_y.max(start_pos.top_row_y);
    }

    fn add_x_padding(&self) -> GlyphStart {
        GlyphStart {
            x_pos: (self.last_written.x_pos + self.padding),
            top_row_y: (self.current_top_row_y),
        }
    }

    fn is_out_of_width(&self, glyph_width: u32) -> bool {
        if self.last_written.x_pos + glyph_width > self.atlas_width {
            true
        } else {
            false
        }
    }

    fn move_to_next_row(&self, glyph_height: u32) -> GlyphStart {
        GlyphStart {
            x_pos: (0 + self.padding),
            top_row_y: (self.current_top_row_y + glyph_height + self.padding),
        }
    }
}
