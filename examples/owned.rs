/// This example demonstrates how to generate a mesh
/// for the uppercase character "A" using the `owned` _feature_.
///
/// Run using `cargo run --example owned --features owned`.
///
#[cfg(feature = "owned")]
use meshtext::{Glyph, MeshGenerator, MeshText};

#[cfg(feature = "owned")]
fn main() {
    let character = 'A';
    let font_data = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/font/FiraMono-Regular.ttf"
    ))
    .expect("Failed to read font.");
    let mut generator = MeshGenerator::new(font_data);
    let result: MeshText = generator
        .generate_glyph(character, true, None)
        .expect("Failed to generate glyph.");

    println!("Generated a mesh for the letter \"{}\".", character);
    println!("Vertices: {:#?}", result.vertices);
}

#[cfg(not(feature = "owned"))]
fn main() {
    println!("The \"owned\" feature is disabled.");
}
