/// This example demonstrates how to convert the
/// text "Meshtext" into a 3D indexed mesh and how to
/// export it as GLTF.
///
use std::{fs, mem};

use gltf_json::validation::{Checked::Valid, USize64};
use meshtext::{BoundingBox, IndexedMeshText, MeshGenerator, TextSection};
use std::borrow::Cow;
use std::io::Write;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Output {
    Standard, // .gltf
    Binary,   // .glb
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

fn align_to_multiple_of_four(n: &mut u32) {
    *n = (*n + 3) & !3;
}

fn to_padded_byte_vector<T>(vec: Vec<T>) -> Vec<u8> {
    let byte_length = vec.len() * mem::size_of::<T>();
    let byte_capacity = vec.capacity() * mem::size_of::<T>();
    let alloc = vec.into_boxed_slice();
    let ptr = Box::<[T]>::into_raw(alloc) as *mut u8;
    let mut new_vec = unsafe { Vec::from_raw_parts(ptr, byte_length, byte_capacity) };
    while new_vec.len() % 4 != 0 {
        // Pad to multiple of four bytes.
        new_vec.push(0);
    }
    new_vec
}

fn export(
    vertex_data: &[f32],
    index_data: &[u32],
    bbox: &BoundingBox,
    color: [f32; 3],
    output: Output,
) {
    let mut vertices: Vec<Vertex> = vec![];
    for i in 0..vertex_data.len() {
        // Add new vertex with the provided color.
        if i % 3 == 2 {
            vertices.push(Vertex {
                position: [vertex_data[i - 2], vertex_data[i - 1], vertex_data[i]],
                color,
            });
        }
    }

    let vertex_buffer_length =
        (vertices.len() * mem::size_of::<Vertex>() + mem::size_of_val(index_data)).into();
    let vertex_buffer = gltf_json::Buffer {
        byte_length: vertex_buffer_length,
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        uri: if output == Output::Standard {
            Some("buffer0.bin".into())
        } else {
            None
        },
    };
    let vertex_buffer_view = gltf_json::buffer::View {
        buffer: gltf_json::Index::new(0),
        byte_length: vertex_buffer.byte_length,
        byte_offset: Some(USize64::from(0usize)),
        byte_stride: Some(gltf_json::buffer::Stride(mem::size_of::<Vertex>())),
        extensions: Default::default(),
        extras: Default::default(),
        name: Some(std::string::String::from("Vertices0")),
        target: Some(Valid(gltf_json::buffer::Target::ArrayBuffer)),
    };

    let index_buffer_view = gltf_json::buffer::View {
        buffer: gltf_json::Index::new(0),
        byte_length: mem::size_of_val(index_data).into(),
        byte_offset: Some((vertices.len() * mem::size_of::<Vertex>()).into()),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        name: Some(std::string::String::from("Indices0")),
        target: Some(Valid(gltf_json::buffer::Target::ElementArrayBuffer)),
    };

    let positions = gltf_json::Accessor {
        buffer_view: Some(gltf_json::Index::new(0)),
        byte_offset: Some(USize64::from(0usize)),
        count: vertices.len().into(),
        component_type: Valid(gltf_json::accessor::GenericComponentType(
            gltf_json::accessor::ComponentType::F32,
        )),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Valid(gltf_json::accessor::Type::Vec3),
        min: Some(gltf_json::Value::from(bbox.min.to_array().to_vec())),
        max: Some(gltf_json::Value::from(bbox.max.to_array().to_vec())),
        name: None,
        normalized: false,
        sparse: None,
    };
    let colors = gltf_json::Accessor {
        buffer_view: Some(gltf_json::Index::new(0)),
        byte_offset: Some((3 * mem::size_of::<f32>()).into()),
        count: vertices.len().into(),
        component_type: Valid(gltf_json::accessor::GenericComponentType(
            gltf_json::accessor::ComponentType::F32,
        )),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Valid(gltf_json::accessor::Type::Vec3),
        min: None,
        max: None,
        name: None,
        normalized: false,
        sparse: None,
    };

    let indices = gltf_json::Accessor {
        buffer_view: Some(gltf_json::Index::new(1)),
        byte_offset: Some(USize64::from(0usize)),
        count: index_data.len().into(),
        component_type: Valid(gltf_json::accessor::GenericComponentType(
            gltf_json::accessor::ComponentType::U32,
        )),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Valid(gltf_json::accessor::Type::Scalar),
        min: None,
        max: None,
        name: None,
        normalized: false,
        sparse: None,
    };

    let primitive = gltf_json::mesh::Primitive {
        attributes: {
            let mut map = std::collections::BTreeMap::new();
            map.insert(
                Valid(gltf_json::mesh::Semantic::Positions),
                gltf_json::Index::new(0),
            );
            map.insert(
                Valid(gltf_json::mesh::Semantic::Colors(0)),
                gltf_json::Index::new(1),
            );
            map
        },
        extensions: Default::default(),
        extras: Default::default(),
        indices: Some(gltf_json::Index::new(2)),
        material: None,
        mode: Valid(gltf_json::mesh::Mode::Triangles),
        targets: None,
    };

    let mesh = gltf_json::Mesh {
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        primitives: vec![primitive],
        weights: None,
    };

    let node = gltf_json::Node {
        camera: None,
        children: None,
        extensions: Default::default(),
        extras: Default::default(),
        matrix: None,
        mesh: Some(gltf_json::Index::new(0)),
        name: None,
        rotation: None,
        scale: None,
        translation: None,
        skin: None,
        weights: None,
    };

    let root = gltf_json::Root {
        accessors: vec![positions, colors, indices],
        buffers: vec![vertex_buffer],
        buffer_views: vec![vertex_buffer_view, index_buffer_view],
        meshes: vec![mesh],
        nodes: vec![node],
        scenes: vec![gltf_json::Scene {
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            nodes: vec![gltf_json::Index::new(0)],
        }],
        ..Default::default()
    };

    match output {
        Output::Standard => {
            let _ = fs::create_dir("mesh_text");

            let writer = fs::File::create("mesh_text/mesh_text.gltf").expect("I/O error");
            gltf_json::serialize::to_writer_pretty(writer, &root).expect("Serialization error");

            let mut bin = to_padded_byte_vector(vertices);
            bin.append(&mut to_padded_byte_vector(index_data.to_vec()));
            let mut writer = fs::File::create("mesh_text/buffer0.bin").expect("I/O error");
            writer.write_all(&bin).expect("I/O error");
        }
        Output::Binary => {
            let gltf_json_string =
                gltf_json::serialize::to_string(&root).expect("Serialization error");
            let mut gltf_json_offset = gltf_json_string.len() as u32;
            align_to_multiple_of_four(&mut gltf_json_offset);

            let mut bin = to_padded_byte_vector(vertices);
            bin.append(&mut to_padded_byte_vector(index_data.to_vec()));

            let glb = gltf::binary::Glb {
                header: gltf::binary::Header {
                    magic: *b"glTF",
                    version: 2,
                    length: gltf_json_offset + vertex_buffer_length.0 as u32,
                },
                bin: Some(Cow::Owned(bin)),
                json: Cow::Owned(gltf_json_string.into_bytes()),
            };
            let writer = std::fs::File::create("mesh_text.glb").expect("I/O error");
            glb.to_writer(writer).expect("glTF binary output error");
        }
    }
}

fn main() {
    let blue = [0f32, 0f32, 1f32];
    let before = std::time::Instant::now();
    let font_data = include_bytes!("../assets/font/FiraMono-Regular.ttf");
    let mut generator = MeshGenerator::new_with_quality(
        font_data,
        meshtext::QualitySettings {
            quad_interpolation_steps: 2,
            cubic_interpolation_steps: 2,
        },
    );
    let result: IndexedMeshText = generator
        .generate_section(
            "Meshtext",
            false,
            Some(&glam::Mat4::from_scale(glam::Vec3::new(1f32, 1f32, 0.1f32)).to_cols_array()),
        )
        .expect("Failed to generate glyph.");
    println!(
        "Generating glyph from scratch took {:#?}.",
        before.elapsed()
    );

    export(
        &result.vertices,
        &result.indices,
        &result.bbox,
        blue,
        Output::Binary,
    );
}
