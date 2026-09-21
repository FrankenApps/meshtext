/// This example demonstrates how to generate a mesh
/// for the uppercase character "A".
///
use meshtext::{Glyph, MeshGenerator, MeshText};
use skrifa::{
    instance::{LocationRef, Size},
    MetadataProvider,
};

fn main() {
    let character = 'A';
    let font_data = include_bytes!("../assets/font/FiraMono-Regular.ttf");
    #[cfg(feature = "owned")]
    let font_data = font_data.to_vec();
    let mut generator = MeshGenerator::new(font_data);

    let font = generator.font();
    #[cfg(feature = "owned")]
    let font = font.as_face_ref();
    println!(
        "Loaded font with {} glyphs.",
        font.metrics(Size::unscaled(), LocationRef::default())
            .glyph_count
    );

    let result: MeshText = generator
        .generate_glyph(character, true, None)
        .expect("Failed to generate glyph.");

    println!("Generated a mesh for the letter \"{}\".", character);
    println!("Vertices: {:#?}", result.vertices);
}
