use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

/// Load a PPM file in P5 format (grayscale binary)
/// Returns (pixel_data, width, height)
pub fn load_ppm<P: AsRef<Path>>(path: P) -> Result<(Vec<u8>, u32, u32), String> {
    let file = File::open(path).map_err(|e| format!("Failed to open PPM file: {}", e))?;
    let mut reader = BufReader::new(file);

    // Read magic number
    let mut magic = String::new();
    reader
        .read_line(&mut magic)
        .map_err(|e| format!("Failed to read magic number: {}", e))?;

    if !magic.trim().starts_with("P5") {
        return Err(format!(
            "Unsupported PPM format: {}. Only P5 (grayscale binary) is supported.",
            magic.trim()
        ));
    }

    // Read dimensions (skip comments)
    let (width, height) = read_dimensions(&mut reader)?;

    // Read max value
    let mut max_val_line = String::new();
    reader
        .read_line(&mut max_val_line)
        .map_err(|e| format!("Failed to read max value: {}", e))?;
    let max_val: u32 = max_val_line
        .trim()
        .parse()
        .map_err(|e| format!("Invalid max value: {}", e))?;

    if max_val != 255 {
        return Err(format!(
            "Unsupported max value: {}. Only 255 is supported.",
            max_val
        ));
    }

    // Read pixel data
    let pixel_count = (width * height) as usize;
    let mut pixels = vec![0u8; pixel_count];
    reader
        .read_exact(&mut pixels)
        .map_err(|e| format!("Failed to read pixel data: {}", e))?;

    Ok((pixels, width, height))
}

fn read_dimensions<R: BufRead>(reader: &mut R) -> Result<(u32, u32), String> {
    let mut line = String::new();

    loop {
        line.clear();
        reader
            .read_line(&mut line)
            .map_err(|e| format!("Failed to read dimensions: {}", e))?;

        let trimmed = line.trim();

        // Skip comment lines
        if trimmed.starts_with('#') {
            continue;
        }

        // Parse width and height
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() == 2 {
            let width: u32 = parts[0]
                .parse()
                .map_err(|e| format!("Invalid width: {}", e))?;
            let height: u32 = parts[1]
                .parse()
                .map_err(|e| format!("Invalid height: {}", e))?;
            return Ok((width, height));
        }
    }
}
