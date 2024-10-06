use glam::{Vec2, Vec3A};

use crate::BoundingBox;

type Mesh = (Vec<Vec3A>, BoundingBox);
type Mesh2D = (Vec<Vec2>, BoundingBox);

type IndexedMesh = (Vec<u32>, Vec<Vec3A>, BoundingBox);
type IndexedMesh2D = (Vec<u32>, Vec<Vec2>, BoundingBox);

/// Converts a [Mesh] composed of three-component vertices to
/// a mesh composed of two-component vertices.
///
/// Arguments:
///
/// * `mesh_3d`: The original mesh using three-component vertices.
///
/// Returns:
///
/// A [Mesh2D] that is composed of two-component vertices.
pub(crate) fn mesh_to_flat_2d(mesh_3d: Mesh) -> Mesh2D {
    let positions = mesh_3d.0;
    let mut positions_2d = Vec::new();

    for pos in positions.iter() {
        positions_2d.push(Vec2::new(pos.x, pos.y))
    }

    let bounding_rect = make_flat_bbox(&mesh_3d.1);

    (positions_2d, bounding_rect)
}

/// Converts an [IndexedMesh] composed of three-component vertices to
/// a mesh composed of two-component vertices.
///
/// Arguments:
///
/// * `mesh_3d`: The original indexed mesh using three-component vertices.
///
/// Returns:
///
/// A [IndexedMesh2D] that is composed of two-component vertices.
pub(crate) fn mesh_to_indexed_flat_2d(mesh_3d: IndexedMesh) -> IndexedMesh2D {
    let positions = mesh_3d.1;
    let mut positions_2d = Vec::new();

    for pos in positions.iter() {
        positions_2d.push(Vec2::new(pos.x, pos.y))
    }

    let bounding_rect = make_flat_bbox(&mesh_3d.2);

    (mesh_3d.0, positions_2d, bounding_rect)
}

/// Converts any [BoundingBox] to a **flat** [BoundingBox].
///
/// Essentially sets all z-components to `0`.
///
/// Arguments:
///
/// * `bbox`: The original [BoundingBox] which might have an extent in
///   the z-direction.
///
/// Returns:
///
/// A **flat** [BoundingBox] that has no depth.
fn make_flat_bbox(bbox: &BoundingBox) -> BoundingBox {
    BoundingBox::new(
        Vec3A::new(bbox.min.x, bbox.min.y, 0f32),
        Vec3A::new(bbox.max.x, bbox.max.y, 0f32),
    )
}
