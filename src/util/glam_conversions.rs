use glam::{Vec2, Vec3A};

pub(crate) fn glam_vecs_from_raw(raw: Vec<f32>) -> Vec<Vec3A> {
    let mut glam_vecs = Vec::new();

    for components in raw.chunks(3) {
        glam_vecs.push(Vec3A::new(components[0], components[1], components[2]));
    }

    glam_vecs
}

pub(crate) fn glam_3d_vecs_from_raw_2d(raw: Vec<f32>) -> Vec<Vec3A> {
    let mut glam_vecs = Vec::new();

    for components in raw.chunks(2) {
        glam_vecs.push(Vec3A::new(components[0], components[1], 0f32));
    }

    glam_vecs
}

/// Converts a [Vec] of [Vec3A]s to a [Vec] of [f32].
///
/// Arguments:
///
/// * `vecs`: The list of [Vec3A]s.
///
/// Returns:
///
/// The concatenated components of all [Vec3A]s combined in a single
/// [Vec].
pub(crate) fn glam_vecs_to_raw(vecs: &[Vec3A]) -> Vec<f32> {
    let mut raw_vecs = Vec::new();

    for vec in vecs.iter() {
        raw_vecs.push(vec.x);
        raw_vecs.push(vec.y);
        raw_vecs.push(vec.z);
    }

    raw_vecs
}

/// Converts a [Vec] of [Vec2]s to a [Vec] of [f32].
///
/// Arguments:
///
/// * `vecs`: The list of [Vec2]s.
///
/// Returns:
///
/// The concatenated components of all [Vec2]s combined in a single
/// [Vec].
pub(crate) fn glam_vecs_to_raw_2d(vecs: &[Vec2]) -> Vec<f32> {
    let mut raw_vecs = Vec::new();

    for vec in vecs.iter() {
        raw_vecs.push(vec.x);
        raw_vecs.push(vec.y);
    }

    raw_vecs
}
