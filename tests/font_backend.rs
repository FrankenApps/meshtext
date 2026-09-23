use meshtext::{Glyph, IndexedMeshText, MeshGenerator, MeshText, QualitySettings, TextSection};
use skrifa::{
    instance::{LocationRef, Size},
    FontRef, MetadataProvider,
};

const FONT: &[u8] = include_bytes!("../assets/font/FiraMono-Regular.ttf");

#[cfg(not(feature = "owned"))]
type Generator = MeshGenerator<meshtext::Face<'static>>;
#[cfg(feature = "owned")]
type Generator = MeshGenerator<meshtext::OwnedFace>;

fn generator(constructor: u8) -> Generator {
    #[cfg(not(feature = "owned"))]
    let data = FONT;
    #[cfg(feature = "owned")]
    let data = FONT.to_vec();
    match constructor {
        0 => MeshGenerator::new(data),
        1 => MeshGenerator::new_with_quality(data, QualitySettings::default()),
        _ => MeshGenerator::new_without_cache(data, QualitySettings::default()),
    }
}

fn expand(mesh: &IndexedMeshText, dimensions: usize) -> Vec<f32> {
    mesh.indices
        .iter()
        .flat_map(|&index| {
            let start = index as usize * dimensions;
            mesh.vertices[start..start + dimensions].iter().copied()
        })
        .collect()
}

fn surface_area(vertices: &[f32]) -> f32 {
    vertices
        .chunks_exact(9)
        .map(|triangle| {
            let a = glam::Vec3::from_slice(&triangle[..3]);
            let b = glam::Vec3::from_slice(&triangle[3..6]);
            let c = glam::Vec3::from_slice(&triangle[6..]);
            (b - a).cross(c - a).length() * 0.5
        })
        .sum()
}

#[test]
fn indexed_and_non_indexed_meshes_match_in_all_dimensions() {
    for constructor in 0..3 {
        let mut generator = generator(constructor);
        for flat in [true, false] {
            let mesh: MeshText = generator.generate_section("A Ö", flat, None).unwrap();
            let indexed: IndexedMeshText = generator.generate_section("A Ö", flat, None).unwrap();
            assert!(!mesh.vertices.is_empty());
            let expanded = expand(&indexed, 3);
            if flat {
                assert_eq!(mesh.vertices, expanded);
            } else {
                // The two extrusion paths split side quads along different diagonals.
                assert_eq!(mesh.vertices.len(), expanded.len());
                assert!((surface_area(&mesh.vertices) - surface_area(&expanded)).abs() < 1e-4);
            }
            assert_eq!(mesh.bbox, indexed.bbox);

            let cached: MeshText = generator.generate_section("A Ö", flat, None).unwrap();
            assert_eq!(mesh.vertices, cached.vertices);
        }
        let mesh: MeshText = generator.generate_section_2d("A Ö", None).unwrap();
        let indexed: IndexedMeshText = generator.generate_section_2d("A Ö", None).unwrap();
        assert_eq!(mesh.vertices, expand(&indexed, 2));
        assert_eq!(mesh.bbox, indexed.bbox);
    }
}

#[test]
fn whitespace_is_empty_but_advances_the_next_glyph() {
    let mut generator = generator(0);
    for flat in [true, false] {
        let space: MeshText = generator.generate_glyph(' ', flat, None).unwrap();
        let indexed_space: IndexedMeshText = generator.generate_glyph(' ', flat, None).unwrap();
        assert!(space.vertices.is_empty());
        assert!(indexed_space.vertices.is_empty());
        assert!(indexed_space.indices.is_empty());
        assert_eq!(space.bbox.size(), glam::Vec3A::ZERO);
        assert_eq!(space.bbox, indexed_space.bbox);
    }
    let empty: MeshText = generator.generate_section("", true, None).unwrap();
    assert!(empty.vertices.is_empty());

    let letter: MeshText = generator.generate_glyph_2d('A', None).unwrap();
    let spaced: MeshText = generator.generate_section_2d(" A", None).unwrap();
    assert_eq!(letter.vertices.len(), spaced.vertices.len());
    for (letter, spaced) in letter
        .vertices
        .chunks_exact(2)
        .zip(spaced.vertices.chunks_exact(2))
    {
        // Fira Mono advances 600 design units at a height of 1200.
        assert!((spaced[0] - letter[0] - 0.5).abs() < 1e-6);
        assert_eq!(spaced[1], letter[1]);
    }
}

#[test]
fn missing_characters_use_notdef() {
    let mut generator = generator(0);
    let missing: MeshText = generator.generate_glyph('\u{10ffff}', true, None).unwrap();
    let notdef: MeshText = generator.generate_glyph('\0', true, None).unwrap();
    assert!(!missing.vertices.is_empty());
    assert_eq!(missing.vertices, notdef.vertices);
    assert_eq!(missing.bbox, notdef.bbox);
}

#[test]
fn metadata_and_normalization_are_preserved() {
    let mut generator = generator(0);
    #[cfg(not(feature = "owned"))]
    let font = generator.font();
    #[cfg(feature = "owned")]
    let font = generator.font().as_face_ref();
    let metrics = font.metrics(Size::unscaled(), LocationRef::default());
    assert_eq!(metrics.ascent - metrics.descent, 1200.0);
    assert_eq!(
        font.charmap().map('A'),
        FontRef::new(FONT).unwrap().charmap().map('A')
    );

    let mesh: MeshText = generator.generate_glyph('A', false, None).unwrap();
    assert_eq!(mesh.bbox.min, glam::Vec3A::new(20.0 / 1200.0, 0.0, -0.5));
    assert_eq!(
        mesh.bbox.max,
        glam::Vec3A::new(579.0 / 1200.0, 689.0 / 1200.0, 0.5)
    );
}

#[cfg(not(feature = "owned"))]
#[test]
fn borrowed_constructors_accept_non_static_buffers() {
    let data = FONT.to_vec();
    let mut generators = [
        MeshGenerator::new(&data),
        MeshGenerator::new_with_quality(&data, QualitySettings::default()),
        MeshGenerator::new_without_cache(&data, QualitySettings::default()),
    ];
    for generator in &mut generators {
        let mesh: MeshText = generator.generate_glyph('A', true, None).unwrap();
        assert!(!mesh.vertices.is_empty());
    }
}

#[cfg(feature = "owned")]
#[test]
fn owned_generators_outlive_loading_scope_and_move_between_threads() {
    fn assert_send_sync_static<T: Send + Sync + 'static>() {}
    assert_send_sync_static::<Generator>();
    for constructor in 0..3 {
        let generator = generator(constructor);
        // Each generator owns a locally allocated buffer from the helper above.
        let mesh = std::thread::spawn(move || {
            let mut generator = generator;
            let mesh: MeshText = generator.generate_glyph('A', false, None).unwrap();
            mesh
        })
        .join()
        .unwrap();
        assert!(!mesh.vertices.is_empty());
    }
}

#[cfg(feature = "owned")]
#[test]
fn owned_face_retains_and_returns_the_original_allocation() {
    let data = FONT.to_vec();
    let pointer = data.as_ptr();
    let face = meshtext::OwnedFace::from_vec(data, 0).unwrap();
    assert_eq!(face.as_slice().as_ptr(), pointer);
    assert!(face.as_face_ref().charmap().map('A').is_some());
    let data = face.into_vec();
    assert_eq!(data.as_ptr(), pointer);
    assert_eq!(data, FONT);

    assert!(meshtext::OwnedFace::from_vec(vec![], 0).is_err());
    assert!(meshtext::OwnedFace::from_vec(FONT.to_vec(), 1).is_err());
}

#[test]
#[should_panic(expected = "Failed to generate font from data.")]
fn invalid_font_is_rejected_at_construction() {
    #[cfg(not(feature = "owned"))]
    let _ = MeshGenerator::new(&[]);
    #[cfg(feature = "owned")]
    let _ = MeshGenerator::new(Vec::new());
}

#[test]
fn font_collections_are_supported() {
    // Wrap the bundled font in a one-face TTC, adjusting table offsets because
    // collection table offsets are relative to the entire file.
    let mut collection = b"ttcf\0\x01\0\0\0\0\0\x01\0\0\0\x10".to_vec();
    collection.extend_from_slice(FONT);
    let table_count = u16::from_be_bytes(FONT[4..6].try_into().unwrap()) as usize;
    for table in 0..table_count {
        let offset_pos = 16 + 12 + table * 16 + 8;
        let offset = u32::from_be_bytes(collection[offset_pos..offset_pos + 4].try_into().unwrap());
        collection[offset_pos..offset_pos + 4].copy_from_slice(&(offset + 16).to_be_bytes());
    }
    #[cfg(feature = "owned")]
    {
        let face = meshtext::OwnedFace::from_vec(collection.clone(), 0).unwrap();
        assert!(face.as_face_ref().charmap().map('A').is_some());
        assert!(meshtext::OwnedFace::from_vec(collection.clone(), 1).is_err());
    }
    #[cfg(not(feature = "owned"))]
    let collection = collection.as_slice();
    let mut generator = MeshGenerator::new(collection);
    let mesh: MeshText = generator.generate_glyph('A', true, None).unwrap();
    assert!(!mesh.vertices.is_empty());
    assert_eq!(mesh.bbox.max.y, 689.0 / 1200.0);
}

#[test]
fn bundled_font_glyphs_triangulate_at_multiple_quality_levels() {
    for steps in [2, 5, 10] {
        #[cfg(not(feature = "owned"))]
        let data = FONT;
        #[cfg(feature = "owned")]
        let data = FONT.to_vec();
        let mut generator = MeshGenerator::new_with_quality(
            data,
            QualitySettings {
                quad_interpolation_steps: steps,
                cubic_interpolation_steps: steps,
            },
        );
        for glyph in (b'!'..=b'~').map(char::from).chain(['Ö']) {
            let mesh: IndexedMeshText = generator
                .generate_glyph(glyph, false, None)
                .unwrap_or_else(|error| panic!("glyph {glyph:?}, steps {steps}: {error:?}"));
            assert!(mesh.vertices.iter().all(|v| v.is_finite()));
            for t in mesh.indices.chunks_exact(3) {
                let a = glam::Vec3::from_slice(&mesh.vertices[t[0] as usize * 3..]);
                let b = glam::Vec3::from_slice(&mesh.vertices[t[1] as usize * 3..]);
                let c = glam::Vec3::from_slice(&mesh.vertices[t[2] as usize * 3..]);
                assert!(
                    (b - a).cross(c - a).length_squared() > 0.0,
                    "degenerate face for {glyph:?}"
                );
            }
        }
    }
}

#[test]
fn indexed_sections_offset_by_vertex_count_including_unused_duplicates() {
    let mut generator = generator(0);
    let flat = IndexedMeshText {
        bbox: meshtext::BoundingBox::new(glam::Vec3A::ZERO, glam::Vec3A::new(1., 1., 0.)),
        indices: vec![0, 1, 2],
        // A trailing duplicate may be unreferenced after canonicalization.
        vertices: vec![0., 0., 1., 0., 0., 1., 0., 1.],
    };
    generator.precache_custom_glyph('A', true, &flat).unwrap();
    let mesh: IndexedMeshText = generator.generate_section_2d("AA", None).unwrap();
    assert_eq!(mesh.indices, vec![0, 1, 2, 4, 5, 6]);
    let mesh: IndexedMeshText = generator.generate_section("AA", true, None).unwrap();
    assert_eq!(mesh.indices, vec![0, 1, 2, 4, 5, 6]);
    let extruded = IndexedMeshText {
        vertices: vec![0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 1., 0.],
        ..flat
    };
    generator
        .precache_custom_glyph('A', false, &extruded)
        .unwrap();
    let mesh: IndexedMeshText = generator.generate_section("AA", false, None).unwrap();
    assert_eq!(mesh.indices, vec![0, 1, 2, 4, 5, 6]);
}
