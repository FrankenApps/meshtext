use glam::Vec3A;

use crate::{
    error::{GlyphTriangulationError, MeshTextError},
    GlyphOutline,
};

use super::{triangulate_between_edges, triangulate_between_edges_indexed};

type EdgeIndices = (usize, usize);
type TriangleIndices = (usize, usize, usize);

/// Generates a triangle mesh from a discrete [GlyphOutline].
///
/// Arguments:
///
/// * `outline`: The outline of the desired glyph.
/// * `flat`: Generates a two dimensional mesh if `true`, otherwise
///   a three dimensional mesh with depth `1.0` units is generated.
///
/// Returns:
///
/// A [Result] containing the generated mesh data or an [MeshTextError] if
/// anything went wrong in the process.
pub(crate) fn raster_to_mesh(
    outline: &GlyphOutline,
    flat: bool,
) -> Result<Vec<Vec3A>, Box<dyn MeshTextError>> {
    let points = &outline.points;
    let (triangles, edges) = get_glyph_area_triangulation(outline)?;

    if flat {
        let mut vertices = Vec::new();
        for i in triangles {
            vertices.push(Vec3A::new(points[i.0].0, points[i.0].1, 0f32));
            vertices.push(Vec3A::new(points[i.1].0, points[i.1].1, 0f32));
            vertices.push(Vec3A::new(points[i.2].0, points[i.2].1, 0f32));
        }

        Ok(vertices)
    } else {
        let mut vertices = Vec::new();
        for i in triangles {
            // The first triangle.
            vertices.push(Vec3A::new(points[i.0].0, points[i.0].1, 0.5f32));
            vertices.push(Vec3A::new(points[i.1].0, points[i.1].1, 0.5f32));
            vertices.push(Vec3A::new(points[i.2].0, points[i.2].1, 0.5f32));

            // The second triangle.
            // The order of vertices is changed, so that the triangle faces outward.
            vertices.push(Vec3A::new(points[i.2].0, points[i.2].1, -0.5f32));
            vertices.push(Vec3A::new(points[i.1].0, points[i.1].1, -0.5f32));
            vertices.push(Vec3A::new(points[i.0].0, points[i.0].1, -0.5f32));
        }

        // Finally add the triangles in between the contours (e.g. in the z-axis).
        triangulate_between_edges(&mut vertices, &outline.points, &edges);

        Ok(vertices)
    }
}

/// Generates an indexed triangle mesh from a discrete [GlyphOutline].
///
/// Arguments:
///
/// * `outline`: The outline of the desired glyph.
/// * `flat`: Generates a two dimensional mesh if `true`, otherwise
///   a three dimensional mesh with depth `1.0` units is generated.
///
/// Returns:
///
/// A [Result] containing the generated mesh data or an [MeshTextError] if
/// anything went wrong in the process.
#[allow(unused)]
pub(crate) fn raster_to_mesh_indexed(
    outline: &GlyphOutline,
    flat: bool,
) -> Result<(Vec<Vec3A>, Vec<u32>), Box<dyn MeshTextError>> {
    let points = &outline.points;
    let (triangles, edges) = get_glyph_area_triangulation(outline)?;

    if flat {
        let mut vertices = Vec::new();
        for p in points {
            vertices.push(Vec3A::new(p.0, p.1, 0f32));
        }

        let mut indices = Vec::new();
        for i in triangles {
            indices.push(i.0 as u32);
            indices.push(i.1 as u32);
            indices.push(i.2 as u32);
        }

        Ok((vertices, indices))
    } else {
        let mut vertices = Vec::new();
        for p in points {
            vertices.push(Vec3A::new(p.0, p.1, 0.5f32));
        }
        let flat_count = vertices.len() as u32;

        for p in points {
            vertices.push(Vec3A::new(p.0, p.1, -0.5f32));
        }

        let mut indices = Vec::new();
        for i in triangles {
            indices.push(i.0 as u32);
            indices.push(i.1 as u32);
            indices.push(i.2 as u32);

            indices.push(i.2 as u32 + flat_count);
            indices.push(i.1 as u32 + flat_count);
            indices.push(i.0 as u32 + flat_count);
        }

        // Add the vertices and indices in between the contours (e.g. in the z-axis).
        triangulate_between_edges_indexed(&mut vertices, &mut indices, &edges);

        Ok((vertices, indices))
    }
}

fn get_glyph_area_triangulation(
    outline: &GlyphOutline,
) -> Result<(Vec<TriangleIndices>, Vec<EdgeIndices>), Box<dyn MeshTextError>> {
    // TODO: Implement a custom triangulation algorithm to get rid of these conversions.
    let points: Vec<(f64, f64)> = outline
        .points
        .iter()
        .map(|p| (p.0 as f64, p.1 as f64))
        .collect();
    let mut contours = Vec::new();
    for c in outline.contours.iter() {
        let path_indices: Vec<usize> = c.iter().map(|i| *i as usize).collect();
        contours.push(path_indices);
    }

    // We might need access to the edges later, so we compute them here once.
    let mut edges = Vec::new();
    for c in contours.iter() {
        let next = edges.len();
        for (a, b) in c.iter().zip(c.iter().skip(1)) {
            edges.push((*a, *b));
        }
        if let Some(start) = edges.get(next) {
            if start.0 != edges.last().unwrap().1 {
                return Err(Box::new(crate::error::GlyphTriangulationError(
                    cdt::Error::OpenContour,
                )));
            }
        }
    }

    // Triangulate the contours.
    let triangles = match cdt::triangulate_with_edges(&points, &edges) {
        Ok(result) => result,
        Err(err) => return Err(Box::new(GlyphTriangulationError(err))),
    };
    Ok((triangles, edges))
}
