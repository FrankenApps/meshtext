use crate::{error::MeshTextError, BoundingBox, TriangleMesh};

/// Holds the generated mesh data for the given text input.
///
/// The triangles use indexed vertices.
pub struct IndexedMeshText {
    /// The bounding box of this mesh.
    pub bbox: BoundingBox,

    /// The indices of this mesh.
    pub indices: Vec<u32>,

    /// The vertices of this mesh.
    ///
    /// Each vertex is composed of three [f32] values in the order _XYZ_.
    /// If the mesh is flat the third component must be set to zero.
    pub vertices: Vec<f32>,
}

impl IndexedMeshText {
    /// Creates a new [IndexedMeshText].
    ///
    /// Arguments:
    ///
    /// * `indices`: The indices used to construct a triangle mesh from the supplied `vertices`.
    /// * `vertices`: The vertices forming the mesh. Each vertex is composed of three [f32]
    ///   values in the order _XYZ_. If the mesh is flat the third component must be set to zero.
    ///
    /// Returns:
    ///
    /// The new [IndexedMeshText] or a [MeshTextError] if the operation failed.
    pub fn new(indices: Vec<u32>, vertices: Vec<f32>) -> Result<Self, Box<dyn MeshTextError>> {
        let bbox = BoundingBox::from_vertices(&vertices)?;

        Ok(Self {
            bbox,
            indices,
            vertices,
        })
    }
}

impl TriangleMesh for IndexedMeshText {
    fn bbox(&self) -> BoundingBox {
        self.bbox
    }

    fn indices(&self) -> Option<Vec<u32>> {
        Some(self.indices.clone())
    }

    fn vertices(&self) -> Vec<f32> {
        self.vertices.clone()
    }
}
