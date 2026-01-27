use ab_glyph::{Font, FontArc, PxScale, ScaleFont};

// Test: Can you load a font and print glyph metrics?
fn main() {
    let font_data = include_bytes!("./FreeMono.ttf");
    let font = FontArc::try_from_slice(font_data).unwrap();

    let scaled_font = font.as_scaled(PxScale::from(24.0));

    let glyph_id = font.glyph_id('A');
    println!("Advance width: {}", scaled_font.h_advance(glyph_id));
}
