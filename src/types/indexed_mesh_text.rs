use crate::BoundingBox;

/// Holds the generated mesh data for the given text input.
///
/// The triangles use indexed vertices.
pub struct IndexedMeshText {
    /// The bounding box of this mesh.
    pub bbox: BoundingBox,

    /// The indices of this mesh.
    pub indices: Vec<u32>,

    /// The vertices of this mesh.
    pub vertices: Vec<f32>,
}
