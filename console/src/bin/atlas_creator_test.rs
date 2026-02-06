use console::atlas_creator::AtlasCreator;
use std::fs;

fn main() {
    let creator = AtlasCreator::new();

    // Save individual glyphs for inspection
    fs::create_dir_all("output/glyphs").unwrap();

    for glyph in creator.glyphs() {
        let ch = glyph.character();
        let filename = format!(
            "output/glyphs/glyph_{:03}_'{}'.png",
            ch as u32,
            if ch.is_ascii_graphic() && ch != '/' {
                ch
            } else {
                '_'
            }
        );
        glyph.image().save(&filename).unwrap();
    }

    println!(
        "Saved {} glyphs to console/output/glyphs/",
        creator.glyphs().len()
    );
}
