use crate::BoundingBox;

/// Represents a [triangle mesh](https://en.wikipedia.org/wiki/Triangle_mesh),
/// which comprises a set of vertices, that form one or more triangles.
pub trait TriangleMesh {
    /// Gets the bounding box of this mesh.
    fn bbox(&self) -> BoundingBox;

    /// Gets the optional indices of this mesh.
    ///
    /// The returned value will be [None], if this mesh is not indexed.
    fn indices(&self) -> Option<Vec<u32>>;

    /// Gets the vertices of this mesh.
    fn vertices(&self) -> Vec<f32>;
}
