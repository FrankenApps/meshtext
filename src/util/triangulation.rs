use glam::Vec3A;

/// The normalized coordinate of the upper z-coordinate of an edge.
const UPPER_Z: f32 = 0.5;

/// The normalized coordinate of the upper z-coordinate of an edge.
const LOWER_Z: f32 = -0.5;

/// Offsets an edge in 3D space by `1` unit in the z-direction and triangulates
/// the area in between these two edges.
///
/// Arguments:
///
/// * `vertices`: The [Vec] of vertices to which the vertices will be appended.
/// * `points`: The points on which the edges are defined.
/// * `edges`: The indices of points that form closed paths.
pub(crate) fn triangulate_between_edges(
    vertices: &mut Vec<Vec3A>,
    points: &[(f32, f32)],
    edges: &[(usize, usize)],
) {
    for edge in edges.iter() {
        // First triangle.
        vertices.push(Vec3A::new(points[edge.0].0, points[edge.0].1, UPPER_Z));
        vertices.push(Vec3A::new(points[edge.1].0, points[edge.1].1, UPPER_Z));
        vertices.push(Vec3A::new(points[edge.0].0, points[edge.0].1, LOWER_Z));

        // Second triangle.
        vertices.push(Vec3A::new(points[edge.1].0, points[edge.1].1, LOWER_Z));
        vertices.push(Vec3A::new(points[edge.0].0, points[edge.0].1, LOWER_Z));
        vertices.push(Vec3A::new(points[edge.1].0, points[edge.1].1, UPPER_Z));
    }
}

/// Offsets an edge in 3D space by `1` unit in the z-direction and triangulates
/// the area in between these two edges.
///
/// This function handles indexed meshes.
///
/// Arguments:
///
/// * `vertices`: The [Vec] of vertices to which the vertices will be appended.
/// * `indices`: The [Vec] of indices to which the indices will be appended.
/// * `edges`: The indices of points that form closed paths.
pub(crate) fn triangulate_between_edges_indexed(
    vertices: &mut [Vec3A],
    indices: &mut Vec<u32>,
    edges: &[(usize, usize)],
) {
    let flat_count = (vertices.len() / 2) as u32;

    for e in edges.iter() {
        // First triangle.
        indices.push(e.0 as u32);
        indices.push(e.1 as u32);
        indices.push(flat_count + e.1 as u32);

        // Second triangle.
        indices.push(flat_count + e.0 as u32);
        indices.push(e.0 as u32);
        indices.push(flat_count + e.1 as u32);
    }
}
