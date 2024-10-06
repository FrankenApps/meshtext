#![doc(html_favicon_url = "https://raw.githubusercontent.com/FrankenApps/meshtext/master/logo.png")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/FrankenApps/meshtext/master/logo.png")]
//! Generate 2D or 3D triangle meshes from text.
//!
//! Generate vertices or indices and vertices for a
//! [vertex-vertex mesh](https://en.wikipedia.org/wiki/Polygon_mesh#Vertex-vertex_meshes).
//!
//! - Supports [TrueType](https://docs.microsoft.com/en-us/typography/truetype/),
//! [OpenType](https://docs.microsoft.com/en-us/typography/opentype/spec/)
//! and [AAT](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6AATIntro.html)
//! fonts
//! - Handles caching of characters that were already triangulated
//! - Allows transforming text sections
//! - Fully customizable to easily integrate in your rendering pipeline

/// Contains the various errors that may occur
/// while using this crate.
pub mod error;

// Re-export the faces.
#[cfg(not(feature = "owned"))]
pub use ttf_parser::Face;
#[cfg(feature = "owned")]
pub use owned_ttf_parser::OwnedFace;

mod mesh_generator;
pub use mesh_generator::MeshGenerator;

mod types {
    mod bounding_box;
    pub use bounding_box::BoundingBox;

    mod cache_type;
    pub use cache_type::CacheType;

    mod glyph_outline;
    pub(crate) use glyph_outline::GlyphOutline;

    mod indexed_mesh_text;
    pub use indexed_mesh_text::*;

    mod mesh_text;
    pub use mesh_text::*;

    mod quality_settings;
    pub use quality_settings::QualitySettings;

    mod traits {
        mod font_face;
        pub(crate) use font_face::*;

        mod glyph;
        pub use glyph::*;

        mod triangle_mesh;
        pub use triangle_mesh::*;

        mod text_section;
        pub use text_section::*;
    }
    pub use traits::*;
}
pub use types::*;

pub(crate) mod util {
    mod glam_conversions;
    pub(crate) use glam_conversions::*;

    mod mesh_to_flat_2d;
    pub(crate) use mesh_to_flat_2d::*;

    mod outline_builder;
    pub(crate) use outline_builder::GlyphOutlineBuilder;

    mod raster_to_mesh;
    pub(crate) use raster_to_mesh::*;

    mod text_mesh;
    pub(crate) use text_mesh::*;

    mod triangulation;
    pub(crate) use triangulation::*;
}
