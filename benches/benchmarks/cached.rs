use criterion::{criterion_group, Criterion};
use meshtext::{Glyph, IndexedMeshText, MeshGenerator, TextSection};

/// Measures the time required to load a cached glyph.
///
/// Arguments:
///
/// * `c`: The [Criterion] benchmark manager.
fn load_cached_glyph(c: &mut Criterion) {
    let font_data = include_bytes!("../../assets/font/FiraMono-Regular.ttf");
    let mut generator = MeshGenerator::new(font_data);
    generator
        .precache_glyphs("A", false, None)
        .expect("Failed to pre-cache characters.");

    c.bench_function("load cached glyph", |b| {
        b.iter(|| {
            let _data: IndexedMeshText = generator
                .generate_glyph('A', false, None)
                .expect("Failed to generate glyph.");
        });
    });
}

/// A sample transform matrix that scales the text to 10 percent
/// on the z-axis and translates it by three units in the x-direction.
#[rustfmt::skip]
const SECTION_TRANSFORM: [f32; 16] = [
    1f32, 0f32, 0f32, 3f32,
    0f32, 1f32, 0f32, 0f32,
    0f32, 0f32, 0.10, 0f32,
    0f32, 0f32, 0f32, 1f32
];

/// Measures the time required to load a text section
/// that consists of cached characters with a custom transformation.
///
/// Arguments:
///
/// * `c`: The [Criterion] benchmark manager.
fn load_cached_section(c: &mut Criterion) {
    let font_data = include_bytes!("../../assets/font/FiraMono-Regular.ttf");
    let mut generator = MeshGenerator::new(font_data);
    generator
        .precache_glyphs(
            // cspell: disable-next-line
            "ABCDEFHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
            false,
            None,
        )
        .expect("Failed to pre-cache characters.");

    c.bench_function("load cached section", |b| {
        b.iter(|| {
            let _data: IndexedMeshText = generator
                .generate_section("Hello World.", false, Some(&SECTION_TRANSFORM))
                .expect("Failed to generate glyph.");
        });
    });
}

criterion_group!(benches, load_cached_glyph, load_cached_section);
