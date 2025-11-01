use ghx_constrained_delaunay::{
    constrained_triangulation::ConstrainedTriangulationConfiguration,
    triangulation::TriangulationError, types::Edge,
};
use glam::Vec3A;

use crate::{
    error::{GlyphTriangulationError, MeshTextError},
    GlyphOutline,
};

use super::{triangulate_between_edges, triangulate_between_edges_indexed};

type EdgeIndices = (usize, usize);
type TriangleIndices = [u32; 3];

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
            vertices.push(Vec3A::new(
                points[i[0] as usize].0,
                points[i[0] as usize].1,
                0f32,
            ));
            vertices.push(Vec3A::new(
                points[i[1] as usize].0,
                points[i[1] as usize].1,
                0f32,
            ));
            vertices.push(Vec3A::new(
                points[i[2] as usize].0,
                points[i[2] as usize].1,
                0f32,
            ));
        }

        Ok(vertices)
    } else {
        let mut vertices = Vec::new();
        for i in triangles {
            // The first triangle.
            vertices.push(Vec3A::new(
                points[i[0] as usize].0,
                points[i[0] as usize].1,
                0.5f32,
            ));
            vertices.push(Vec3A::new(
                points[i[1] as usize].0,
                points[i[1] as usize].1,
                0.5f32,
            ));
            vertices.push(Vec3A::new(
                points[i[2] as usize].0,
                points[i[2] as usize].1,
                0.5f32,
            ));

            // The second triangle.
            // The order of vertices is changed, so that the triangle faces outward.
            vertices.push(Vec3A::new(
                points[i[2] as usize].0,
                points[i[2] as usize].1,
                -0.5f32,
            ));
            vertices.push(Vec3A::new(
                points[i[1] as usize].0,
                points[i[1] as usize].1,
                -0.5f32,
            ));
            vertices.push(Vec3A::new(
                points[i[0] as usize].0,
                points[i[0] as usize].1,
                -0.5f32,
            ));
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
            indices.push(i[0]);
            indices.push(i[1]);
            indices.push(i[2]);
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
            indices.push(i[0]);
            indices.push(i[1]);
            indices.push(i[2]);

            indices.push(i[2] + flat_count);
            indices.push(i[1] + flat_count);
            indices.push(i[0] + flat_count);
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
    let points: Vec<ghx_constrained_delaunay::glam::Vec2> = outline
        .points
        .iter()
        .map(|p| ghx_constrained_delaunay::glam::Vec2::new(p.0, p.1))
        .collect();
    let mut contours = Vec::new();
    for c in outline.contours.iter() {
        let path_indices: Vec<u32> = c.iter().map(|i| *i).collect();
        contours.push(path_indices);
    }

    // We might need access to the edges later, so we compute them here once.
    let mut edges = Vec::new();
    for c in contours.iter() {
        let next = edges.len();
        for (a, b) in c.iter().zip(c.iter().skip(1)) {
            edges.push(Edge::new(*a, *b));
        }
        if let Some(start) = edges.get(next) {
            if start.from != edges.last().unwrap().to {
                return Err(Box::new(crate::error::GlyphTriangulationError(
                    //cdt::Error::OpenContour,
                    TriangulationError {
                        msg: "Open contour.".to_string(),
                    },
                )));
            }
        }
    }

    let configuration = ConstrainedTriangulationConfiguration::default();

    // Triangulate the contours.
    let triangles = match ghx_constrained_delaunay::constrained_triangulation::constrained_triangulation_from_2d_vertices(&points, &edges, configuration) {
        Ok(result) => result,
        Err(err) => return Err(Box::new(GlyphTriangulationError(err))),
    };

    Ok((
        triangles.triangles,
        edges
            .iter()
            .map(|e| (e.from as usize, e.to as usize))
            .collect(),
    ))
}
