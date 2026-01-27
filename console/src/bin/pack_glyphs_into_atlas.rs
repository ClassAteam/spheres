use ab_glyph::{Font, FontArc, PxScale};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

// Test: Pack multiple glyphs into a single buffer
#[derive(Debug)]
struct GlyphInfo {
    uv_min: (f32, f32), // Atlas coordinates (normalized 0.0-1.0)
    uv_max: (f32, f32),
    bearing: (i32, i32), // Offset from baseline
    size: (u32, u32),    // Width and height in pixels
}

fn main() -> std::io::Result<()> {
    let font_data = include_bytes!("./FreeMono.ttf");
    let font = FontArc::try_from_slice(font_data).unwrap();

    let scale = PxScale::from(48.0);

    // Simple row-based packing algorithm
    let atlas_width = 512;
    let mut current_x = 0;
    let mut current_y = 0;
    let mut row_height = 0;
    let mut glyph_map: HashMap<char, GlyphInfo> = HashMap::new();

    // First pass: collect all glyph data and calculate atlas dimensions
    let mut glyph_data = Vec::new();

    for ch in 'A'..='Z' {
        let glyph_id = font.glyph_id(ch);
        let glyph = glyph_id.with_scale(scale);

        if let Some(outlined) = font.outline_glyph(glyph) {
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

            glyph_data.push((ch, width, height, pixels, bounds));
        }
    }

    // Calculate required atlas height
    let mut atlas_height = 0;
    current_x = 0;
    current_y = 0;
    row_height = 0;

    for (_, width, height, _, _) in &glyph_data {
        if current_x + width > atlas_width {
            // Move to next row
            current_y += row_height + 1; // +1 for padding
            current_x = 0;
            row_height = 0;
        }

        row_height = row_height.max(*height);
        current_x += width + 1; // +1 for padding
    }
    atlas_height = current_y + row_height;

    println!("Creating atlas: {}x{}", atlas_width, atlas_height);

    // Create atlas buffer
    let mut atlas_data = vec![0u8; (atlas_width * atlas_height) as usize];

    // Second pass: pack glyphs into atlas
    current_x = 0;
    current_y = 0;
    row_height = 0;

    for (ch, width, height, pixels, bounds) in glyph_data {
        // Check if we need to move to next row
        if current_x + width > atlas_width {
            current_y += row_height + 1;
            current_x = 0;
            row_height = 0;
        }

        // Copy glyph pixels to atlas
        for y in 0..height {
            for x in 0..width {
                let src_idx = (y * width + x) as usize;
                let dst_idx = ((current_y + y) * atlas_width + (current_x + x)) as usize;
                if src_idx < pixels.len() && dst_idx < atlas_data.len() {
                    atlas_data[dst_idx] = pixels[src_idx];
                }
            }
        }

        // Store glyph info with normalized UV coordinates
        let info = GlyphInfo {
            uv_min: (
                current_x as f32 / atlas_width as f32,
                current_y as f32 / atlas_height as f32,
            ),
            uv_max: (
                (current_x + width) as f32 / atlas_width as f32,
                (current_y + height) as f32 / atlas_height as f32,
            ),
            bearing: (bounds.min.x as i32, bounds.min.y as i32),
            size: (width, height),
        };

        glyph_map.insert(ch, info);

        row_height = row_height.max(height);
        current_x += width + 1;
    }

    // Create output directory
    let output_dir = PathBuf::from("console/output");
    fs::create_dir_all(&output_dir)?;

    // Save atlas as PPM
    let atlas_path = output_dir.join("glyph_atlas.ppm");
    let mut file = File::create(&atlas_path)?;

    writeln!(file, "P5")?;
    writeln!(file, "{} {}", atlas_width, atlas_height)?;
    writeln!(file, "255")?;
    file.write_all(&atlas_data)?;

    println!("Saved atlas to {}", atlas_path.display());

    // Save glyph metadata
    let metadata_path = output_dir.join("glyph_atlas_metadata.txt");
    let mut metadata_file = File::create(&metadata_path)?;

    writeln!(
        metadata_file,
        "Atlas size: {}x{}",
        atlas_width, atlas_height
    )?;
    writeln!(metadata_file, "Glyphs packed: {}", glyph_map.len())?;
    writeln!(metadata_file)?;

    for ch in 'A'..='Z' {
        if let Some(info) = glyph_map.get(&ch) {
            writeln!(metadata_file, "Char '{}': {:?}", ch, info)?;
        }
    }

    println!("Saved metadata to {}", metadata_path.display());
    println!("\nPacked {} glyphs into atlas", glyph_map.len());

    Ok(())
}
