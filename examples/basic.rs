use meshtext::{Glyph, MeshGenerator, MeshText};

fn main() {
    let character = 'A';
    let font_data = include_bytes!("../assets/font/FiraMono-Regular.ttf");
    let mut generator = MeshGenerator::new(font_data);
    let result: MeshText = generator
        .generate_glyph(character, true, None)
        .expect("Failed to generate glyph.");

    println!("Generated a mesh for the letter \"{}\".", character);
    println!("Vertices: {:#?}", result.vertices);
}
