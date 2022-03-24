use glam::Vec3A;

use crate::{BoundingBox, IndexedMeshText, MeshText};

use super::glam_vecs_to_raw;

/// Generates a [BasicMeshText] from the internal data representation.
///
/// It is a bit unfortunate, that this is needed, because it adds the
/// main bulk of execution time when loading from the cache.
///
/// Arguments:
///
/// * `data`: The internal data from the cache or freshly generated.
///
/// Returns:
///
/// The corresponding [BasicMeshText].
pub(crate) fn text_mesh_from_data(data: (Vec<Vec3A>, BoundingBox)) -> MeshText {
    MeshText {
        bbox: data.1,
        vertices: glam_vecs_to_raw(&data.0),
    }
}

/// Generates a [IndexedMeshText] from the internal data representation.
///
/// This variant handles indexed meshes.
///
/// It is a bit unfortunate, that this is needed, because it adds the
/// main bulk of execution time when loading from the cache.
///
/// Arguments:
///
/// * `data`: The internal data from the cache or freshly generated.
///
/// Returns:
///
/// The corresponding [IndexedMeshText].
pub(crate) fn text_mesh_from_data_indexed(
    data: (Vec<u32>, Vec<Vec3A>, BoundingBox),
) -> IndexedMeshText {
    IndexedMeshText {
        bbox: data.2,
        indices: data.0,
        vertices: glam_vecs_to_raw(&data.1),
    }
}
