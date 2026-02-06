use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use ab_glyph::{Font, FontArc, PxScale, Rect};

pub struct AtlasCreator {
    glyph_data: Vec<GlyphData>,
    atlas_width: u32,
    atlas_height: u32,
    atlas_pixels: Vec<u8>,
}

struct GlyphData {
    ch: char,
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    bounds: Rect,
}

#[derive(Default)]
struct PackingCursor {
    x: u32,
    y: u32,
    row_height: u32,
}

impl AtlasCreator {
    pub fn new() -> Self {
        let font_data = include_bytes!("./../../resources/FreeMono.ttf");
        let font = FontArc::try_from_slice(font_data).unwrap();

        let scale = PxScale::from(48.0);
        let glyph_data = Self::fill_glyph_data(font, scale);
        let atlas_width = 512;
        let atlas_height = Self::compute_atlas_height(&glyph_data, atlas_width);
        let atlas_pixels = Self::write_atlas_pixels(&glyph_data, atlas_width, atlas_height);
        AtlasCreator {
            glyph_data,
            atlas_width,
            atlas_height,
            atlas_pixels,
        }
    }

    pub fn write_to_file(&self) {
        let output_dir = PathBuf::from("console/output");
        fs::create_dir_all(&output_dir).unwrap();

        // Save atlas as PPM
        let atlas_path = output_dir.join("test_atlas.ppm");
        let mut file = File::create(&atlas_path).unwrap();

        writeln!(file, "P5").unwrap();
        writeln!(file, "{} {}", self.atlas_width, self.atlas_height).unwrap();
        writeln!(file, "255").unwrap();
        file.write_all(&self.atlas_pixels).unwrap();

        println!("Saved atlas to {}", atlas_path.display());
    }

    fn fill_glyph_data(font: FontArc, scale: PxScale) -> Vec<GlyphData> {
        let mut glyph_data = Vec::new();
        for ch in 'A'..='Z' {
            let glyph_id = font.glyph_id(ch);
            let glyph_scaled = glyph_id.with_scale(scale);

            if let Some(outlined) = font.outline_glyph(glyph_scaled) {
                let bounds = outlined.px_bounds();
                let width = bounds.width().ceil() as u32;
                let height = bounds.height().ceil() as u32;

                // Collect pixel data
                let mut pixels = vec![0u8; (width * height) as usize];
                outlined.draw(|x, y, coverage| {
                    let idx = (y * width + x) as usize;
                    if idx < pixels.len() {
                        pixels[idx] = (coverage * 255.0) as u8;
                    }
                });

                glyph_data.push(GlyphData {
                    ch,
                    width,
                    height,
                    pixels,
                    bounds,
                });
            }
        }

        return glyph_data;
    }

    fn write_atlas_pixels(
        glyph_data: &[GlyphData],
        atlas_width: u32,
        atlas_height: u32,
    ) -> Vec<u8> {
        let mut cursor = PackingCursor::default();
        let mut atlas_data = vec![0u8; (atlas_width * atlas_height) as usize];
        for glyph in glyph_data {
            if cursor.x + glyph.width > atlas_width {
                cursor.y += cursor.row_height + 1;
                cursor.x = 0;
                cursor.row_height = 0;
            }

            // Copy glyph pixels to atlas
            for y in 0..glyph.height {
                for x in 0..glyph.width {
                    let src_idx = (y * glyph.width + x) as usize;
                    let dst_idx = ((cursor.y + y) * atlas_width + (cursor.x + x)) as usize;
                    if src_idx < glyph.pixels.len() && dst_idx < atlas_data.len() {
                        atlas_data[dst_idx] = glyph.pixels[src_idx];
                    }
                }
            }

            cursor.row_height = cursor.row_height.max(glyph.height);
            cursor.x += glyph.width + 1;
        }
        return atlas_data;
    }

    fn compute_atlas_height(glyph_data: &[GlyphData], atlas_width: u32) -> u32 {
        let mut cursor = PackingCursor::default();
        for entry in glyph_data {
            if cursor.x + entry.width > atlas_width {
                // Glyph doesn't fit — move to next row
                cursor.y += cursor.row_height + 1; // +1 for row padding
                cursor.x = 0;
                cursor.row_height = 0;
            }
            cursor.row_height = cursor.row_height.max(entry.height);
            cursor.x += entry.width + 1; // +1 for horizontal padding
        }
        cursor.y + cursor.row_height
    }
}
