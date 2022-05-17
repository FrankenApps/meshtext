/// This example demonstrates how to generate an indexed mesh
/// for the uppercase character "L".
///
use meshtext::{Glyph, IndexedMeshText, MeshGenerator};

fn main() {
    let character = 'L';
    let font_data = include_bytes!("../assets/font/FiraMono-Regular.ttf");
    let mut generator = MeshGenerator::new(font_data);
    let result: IndexedMeshText = generator
        .generate_glyph(character, true, None)
        .expect("Failed to generate glyph.");

    println!("Generated a mesh for the letter \"{}\".", character);
    println!("Vertices: {:?}", result.vertices);
    println!("Indices: {:?}", result.indices);
}
