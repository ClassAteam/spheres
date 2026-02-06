use console::atlas_creator::AtlasCreator;
fn main() {
    let atlas = AtlasCreator::new();
    atlas.write_to_file();
}
