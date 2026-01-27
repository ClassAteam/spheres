use ab_glyph::{Font, FontArc, PxScale, ScaleFont};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

// Test: Rasterize 'A' and save as PPM file
fn main() -> std::io::Result<()> {
    let font_data = include_bytes!("./FreeMono.ttf");
    let font = FontArc::try_from_slice(font_data).unwrap();

    let scale = PxScale::from(64.0); // Larger size for better visibility
    let glyph_id = font.glyph_id('A');
    let glyph = glyph_id.with_scale(scale);

    if let Some(outlined) = font.outline_glyph(glyph) {
        let bounds = outlined.px_bounds();
        let width = bounds.width() as u32;
        let height = bounds.height() as u32;

        println!("Rasterizing 'A' at {}x{} pixels", width, height);

        let mut pixels = vec![0u8; (width * height) as usize];

        outlined.draw(|x, y, coverage| {
            let idx = (y * width + x) as usize;
            pixels[idx] = (coverage * 255.0) as u8;
        });

        // Create output directory if it doesn't exist
        let output_dir = PathBuf::from("console/output");
        fs::create_dir_all(&output_dir)?;

        // Save as PPM (Portable Pixmap - grayscale P5 format)
        let output_path = output_dir.join("glyph_A.ppm");
        let mut file = File::create(&output_path)?;

        // PPM header: P5 (grayscale), width, height, max value
        writeln!(file, "P5")?;
        writeln!(file, "{} {}", width, height)?;
        writeln!(file, "255")?;

        // Write pixel data
        file.write_all(&pixels)?;

        println!("Saved to {}", output_path.display());
        println!("Open with: feh {}  (or any image viewer)", output_path.display());
    } else {
        println!("Could not outline glyph");
    }

    Ok(())
}
