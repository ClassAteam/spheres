use ab_glyph::{Font, FontVec, PxScale, ScaleFont};
use std::collections::HashMap;
use std::path::Path;

pub struct GlyphMetrics {
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
    pub width: f32,
    pub height: f32,
    pub bearing_x: f32,
    pub bearing_y: f32,
}

pub struct AtlasMetaData {
    pub glyphs: HashMap<char, GlyphMetrics>,
    pub advance: f32,
}

struct GlyphData {
    ch: char,
    width: u32,
    height: u32,
    min_x: f32,
    min_y: f32,
}

#[derive(Default)]
struct PackingCursor {
    x: u32,
    y: u32,
    row_height: u32,
}

impl AtlasMetaData {
    /// Derive glyph UV layout by replaying the exact row-packing algorithm
    /// used by `pack_glyphs_into_atlas`.  `scale` and `atlas_width` must match
    /// the values that were used to generate the atlas PPM.
    pub fn new(font_path: impl AsRef<Path>, scale: f32, atlas_width: u32) -> Self {
        let font_data = std::fs::read(font_path).expect("failed to read font file");
        let font = FontVec::try_from_vec(font_data).expect("failed to parse font");
        let px_scale = PxScale::from(scale);
        let scaled_font = font.as_scaled(px_scale);

        let glyph_bounds = Self::collect_glyph_bounds(&font, px_scale);

        let atlas_height = Self::compute_atlas_height(&glyph_bounds, atlas_width);

        let advance = scaled_font.h_advance(font.glyph_id('A'));

        let metrics = Self::create_metrics_table(&glyph_bounds, atlas_width, atlas_height);

        AtlasMetaData {
            glyphs: metrics,
            advance,
        }
    }

    /// Extracts bounding box dimensions for all uppercase letters (A-Z) at the
    /// specified scale. Returns the data needed for atlas packing: pixel width/height
    /// and bearing offsets relative to the glyph origin.
    fn collect_glyph_bounds(font: &FontVec, px_scale: PxScale) -> Vec<GlyphData> {
        let mut glyph_bounds = Vec::new();
        for ch in 'A'..='Z' {
            let glyph = font.glyph_id(ch).with_scale(px_scale);
            if let Some(outlined) = font.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                glyph_bounds.push(GlyphData {
                    ch,
                    width: bounds.width().ceil() as u32,
                    height: bounds.height().ceil() as u32,
                    min_x: bounds.min.x,
                    min_y: bounds.min.y,
                });
            }
        }
        glyph_bounds
    }

    /// Computes the minimum atlas height needed to pack all glyphs using
    /// left-to-right, top-to-bottom row packing.
    ///
    /// Glyphs are placed sequentially in a row until the next glyph would
    /// exceed `atlas_width`. When that happens, a new row is started below
    /// the previous one. Each row's height is determined by its tallest glyph.
    /// One pixel of padding is added between glyphs and rows.
    fn compute_atlas_height(glyph_bounds: &[GlyphData], atlas_width: u32) -> u32 {
        let mut cursor = PackingCursor::default();
        for entry in glyph_bounds {
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

    /// Packs glyphs into the atlas using the same row-packing algorithm as
    /// `compute_atlas_height`, and computes normalized UV coordinates for each.
    ///
    /// Returns a map from character to its metrics (UV bounds, pixel dimensions,
    /// and bearing offsets for correct positioning during text layout).
    fn create_metrics_table(
        glyph_bounds: &[GlyphData],
        atlas_width: u32,
        atlas_height: u32,
    ) -> HashMap<char, GlyphMetrics> {
        let mut cursor = PackingCursor::default();
        let mut metrics = HashMap::new();

        for entry in glyph_bounds {
            if cursor.x + entry.width > atlas_width {
                cursor.y += cursor.row_height + 1;
                cursor.x = 0;
                cursor.row_height = 0;
            }

            metrics.insert(
                entry.ch,
                GlyphMetrics {
                    uv_min: [
                        cursor.x as f32 / atlas_width as f32,
                        cursor.y as f32 / atlas_height as f32,
                    ],
                    uv_max: [
                        (cursor.x + entry.width) as f32 / atlas_width as f32,
                        (cursor.y + entry.height) as f32 / atlas_height as f32,
                    ],
                    width: entry.width as f32,
                    height: entry.height as f32,
                    bearing_x: entry.min_x,
                    bearing_y: entry.min_y,
                },
            );

            cursor.row_height = cursor.row_height.max(entry.height);
            cursor.x += entry.width + 1;
        }
        metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_glyph(ch: char, width: u32, height: u32) -> GlyphData {
        GlyphData {
            ch,
            width,
            height,
            min_x: 0.0,
            min_y: 0.0,
        }
    }

    #[test]
    fn empty_input_returns_zero() {
        let height = AtlasMetaData::compute_atlas_height(&[], 512);
        assert_eq!(height, 0);
    }

    #[test]
    fn single_glyph_returns_its_height() {
        // Atlas: width=100
        // ┌────────────┐
        // │ A(10x20)   │  ← height = 20
        // └────────────┘
        let glyphs = vec![make_glyph('A', 10, 20)];
        let height = AtlasMetaData::compute_atlas_height(&glyphs, 100);
        assert_eq!(height, 20);
    }

    #[test]
    fn multiple_glyphs_in_one_row_use_tallest_height() {
        // Atlas: width=100
        // ┌────────────────────────┐
        // │ A(20x10) B(15x25)      │  ← height = max(10, 25) = 25
        // └────────────────────────┘
        // Total width: 20 + 1(pad) + 15 = 36, fits in 100
        let glyphs = vec![make_glyph('A', 20, 10), make_glyph('B', 15, 25)];
        let height = AtlasMetaData::compute_atlas_height(&glyphs, 100);
        assert_eq!(height, 25);
    }

    #[test]
    fn glyphs_wrap_to_next_row_when_width_exceeded() {
        // Atlas: width=20
        // ┌──────────────────────┐
        // │ A(10x5)              │  ← row 0: height=5
        // │                      │
        // │ B(12x8)              │  ← row 1: height=8, wrapped because 10+1+12 > 20
        // └──────────────────────┘
        // Total height: 5 (row0) + 1 (padding) + 8 (row1) = 14
        let glyphs = vec![make_glyph('A', 10, 5), make_glyph('B', 12, 8)];
        let height = AtlasMetaData::compute_atlas_height(&glyphs, 20);
        assert_eq!(height, 14);
    }

    #[test]
    fn multiple_rows_with_varying_heights() {
        // Atlas: width=15
        // ┌─────────────────┐
        // │ A(6x10) B(5x3)  │  ← row 0: max(10,3)=10, width=6+1+5=12 fits
        // │                 │
        // │ C(8x7) D(4x12)  │  ← row 1: max(7,12)=12, width=8+1+4=13 fits
        // └─────────────────┘
        // Total: 10 (row0) + 1 (padding) + 12 (row1) = 23
        let glyphs = vec![
            make_glyph('A', 6, 10),
            make_glyph('B', 5, 3),
            make_glyph('C', 8, 7),
            make_glyph('D', 4, 12),
        ];
        let height = AtlasMetaData::compute_atlas_height(&glyphs, 15);
        assert_eq!(height, 23);
    }

    #[test]
    fn glyph_exactly_at_width_boundary_does_not_wrap() {
        // Atlas: width=21
        // ┌───────────────────────┐
        // │ A(10x5) B(10x8)       │  ← both fit: 10+1+10=21 exactly
        // └───────────────────────┘
        // Height should be max(5,8)=8, no wrapping
        let glyphs = vec![make_glyph('A', 10, 5), make_glyph('B', 10, 8)];
        let height = AtlasMetaData::compute_atlas_height(&glyphs, 21);
        assert_eq!(height, 8);
    }

    #[test]
    fn glyph_one_pixel_over_boundary_wraps() {
        // Atlas: width=20
        // ┌──────────────────────┐
        // │ A(10x5)              │  ← row 0: height=5
        // │                      │
        // │ B(10x8)              │  ← row 1: wraps because 10+1+10=21 > 20
        // └──────────────────────┘
        // Total: 5 + 1 + 8 = 14
        let glyphs = vec![make_glyph('A', 10, 5), make_glyph('B', 10, 8)];
        let height = AtlasMetaData::compute_atlas_height(&glyphs, 20);
        assert_eq!(height, 14);
    }
}
