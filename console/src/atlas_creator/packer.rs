use std::collections::HashMap;

use image::GrayImage;

use crate::atlas_creator::{atlas_creator::Atlas, glyph::GlyphData};

pub struct Packer {
    image: GrayImage,
    cursor: Cursor,
}

pub struct GlyphMetrics {
    pub width: u32,
    pub height: u32,
    pub uv_min: UvMinData,
    pub uv_max: UvMaxData,
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub advance_width: f32,
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
        let padding = 2;
        let cursor = Cursor::new(&dimensions, padding);
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

    pub fn pack_to_atlas(&mut self, glyphs: &[GlyphData], line_height: f32) -> Atlas {
        let mut meta_data = HashMap::new();
        for glyph in glyphs {
            let glyph_width = glyph.image().width();
            let glyph_height = glyph.image().height();
            let atlas_width = self.image.width();
            let atlas_height = self.image.height();
            let start = self.cursor.next_glyph_start(glyph_width, glyph_height);
            let uv_min = UvMinData::new(&start, atlas_width, atlas_height);
            let end = self.write_glyph(&glyph, &start);
            let uv_max = UvMaxData::new(&end, atlas_width, atlas_height);
            self.cursor.advance(start, end);
            let metrics = GlyphMetrics {
                width: glyph_width,
                height: glyph_height,
                uv_min,
                uv_max,
                bearing_x: glyph.bearing_x(),
                bearing_y: glyph.bearing_y(),
                advance_width: glyph.advance_width(),
            };

            meta_data.insert(glyph.character(), metrics);
        }

        let ascent = glyphs.iter()
            .map(|g| g.bearing_y())
            .fold(0.0f32, f32::max);

        let image = self.image.to_owned();

        Atlas {
            image,
            info: meta_data,
            ascent,
            line_height,
        }
    }
}

pub struct UvMinData {
    pub x: f32,
    pub y: f32,
}

pub struct UvMaxData {
    pub x: f32,
    pub y: f32,
}

impl UvMinData {
    fn new(first_corner: &GlyphStart, atlas_width: u32, atlas_height: u32) -> Self {
        Self {
            x: first_corner.x_pos as f32 / atlas_width as f32,
            y: first_corner.top_row_y as f32 / atlas_height as f32,
        }
    }
}

impl UvMaxData {
    fn new(second_corner: &GlyphEnd, atlas_width: u32, atlas_height: u32) -> Self {
        Self {
            x: second_corner.x_pos as f32 / atlas_width as f32,
            y: second_corner.y_pos as f32 / atlas_height as f32,
        }
    }
}

impl AtlasDimensions {
    pub fn new(total_area: u32) -> Self {
        let with_overhead = (total_area as f32 * 6.0).ceil() as u32;

        // Calculate width based on area
        let width = ((with_overhead as f32).sqrt().ceil() as u32).next_power_of_two();

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
    pub fn new(dimensions: &AtlasDimensions, padding: u32) -> Self {
        Cursor {
            atlas_width: dimensions.x,
            last_written: LastWrittenEnd { x_pos: 0, y_pos: 0 },
            padding,
            current_top_row_y: 0,
            current_row_height: 0,
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
            x_pos: self.last_written.x_pos + self.padding,
            top_row_y: self.current_top_row_y,
        }
    }

    fn is_out_of_width(&self, glyph_width: u32) -> bool {
        self.last_written.x_pos + self.padding + glyph_width > self.atlas_width
    }

    fn move_to_next_row(&self, _glyph_height: u32) -> GlyphStart {
        GlyphStart {
            x_pos: self.padding,
            top_row_y: self.current_top_row_y + self.current_row_height + self.padding,
        }
    }
}
