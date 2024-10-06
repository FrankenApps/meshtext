use crate::{error::MeshTextError, BoundingBox, TriangleMesh};

/// Holds the generated mesh data for the given text input.
pub struct MeshText {
    /// The bounding box of this mesh.
    pub bbox: BoundingBox,

    /// The vertices of this mesh.
    ///
    /// Each vertex is composed of three [f32] values in the order _XYZ_.
    /// If the mesh is flat the third component must be set to zero.
    pub vertices: Vec<f32>,
}

impl MeshText {
    /// Creates a new [MeshText].
    ///
    /// Arguments:
    ///
    /// * `vertices`: The vertices forming the mesh. Each vertex is composed of three [f32]
    ///    values in the order _XYZ_. If the mesh is flat the third component must be set to zero.
    ///
    /// Returns:
    ///
    /// The new [MeshText] or a [MeshTextError] if the operation failed.
    pub fn new(vertices: Vec<f32>) -> Result<Self, Box<dyn MeshTextError>> {
        let bbox = BoundingBox::from_vertices(&vertices)?;

        Ok(Self { bbox, vertices })
    }
}

impl TriangleMesh for MeshText {
    fn bbox(&self) -> BoundingBox {
        self.bbox
    }

    fn indices(&self) -> Option<Vec<u32>> {
        None
    }

    fn vertices(&self) -> Vec<f32> {
        self.vertices.clone()
    }
}
