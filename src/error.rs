use std::{error::Error, fmt};

/// Any error that can occur while generating a [crate::MeshText] or an [crate::IndexedMeshText].
pub trait MeshTextError: fmt::Debug + fmt::Display {}

/// An error that can occur while parsing the outline of a font.
#[derive(Debug)]
pub struct GlyphOutlineError;

impl MeshTextError for GlyphOutlineError {}

impl Error for GlyphOutlineError {}

impl fmt::Display for GlyphOutlineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "The glyph outline of this font seems to be malformed / unsupported."
        )
    }
}

/// An error that can occur while triangulating the outline of a font.
#[derive(Debug)]
pub struct GlyphTriangulationError(pub ghx_constrained_delaunay::triangulation::TriangulationError);

impl MeshTextError for GlyphTriangulationError {}

impl Error for GlyphTriangulationError {}

impl fmt::Display for GlyphTriangulationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The glyph outline could not be triangulated.")
    }
}

/// An error related to malformed vertices.
#[derive(Debug)]
pub struct VertexError;

impl MeshTextError for VertexError {}

impl Error for VertexError {}

impl fmt::Display for VertexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The given vertex or vertices seem to be malformed.")
    }
}
