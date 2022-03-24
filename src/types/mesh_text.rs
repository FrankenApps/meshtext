use crate::BoundingBox;

/// Holds the generated mesh data for the given text input.
pub struct MeshText {
    /// The bounding box of this mesh.
    pub bbox: BoundingBox,

    /// The vertices of this mesh.
    pub vertices: Vec<f32>,
}
