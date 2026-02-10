use console::atlas_creator::AtlasCreator;

fn main() {
    let mut atlas_creator = AtlasCreator::new();
    let atlas = atlas_creator.create_atlas();
    atlas.write_to_file();
}
