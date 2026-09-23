use glam::Vec3A;

use crate::{
    error::{GlyphTriangulationError, MeshTextError},
    GlyphOutline,
};

use super::{triangulate_between_edges, triangulate_between_edges_indexed};

use crate::triangulation::{Edge as EdgeIndices, Triangle as TriangleIndices, Triangulation};

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
                points[i.0 as usize].0,
                points[i.0 as usize].1,
                0f32,
            ));
            vertices.push(Vec3A::new(
                points[i.1 as usize].0,
                points[i.1 as usize].1,
                0f32,
            ));
            vertices.push(Vec3A::new(
                points[i.2 as usize].0,
                points[i.2 as usize].1,
                0f32,
            ));
        }

        Ok(vertices)
    } else {
        let mut vertices = Vec::new();
        for i in triangles {
            // The first triangle.
            vertices.push(Vec3A::new(
                points[i.0 as usize].0,
                points[i.0 as usize].1,
                0.5f32,
            ));
            vertices.push(Vec3A::new(
                points[i.1 as usize].0,
                points[i.1 as usize].1,
                0.5f32,
            ));
            vertices.push(Vec3A::new(
                points[i.2 as usize].0,
                points[i.2 as usize].1,
                0.5f32,
            ));

            // The second triangle.
            // The order of vertices is changed, so that the triangle faces outward.
            vertices.push(Vec3A::new(
                points[i.2 as usize].0,
                points[i.2 as usize].1,
                -0.5f32,
            ));
            vertices.push(Vec3A::new(
                points[i.1 as usize].0,
                points[i.1 as usize].1,
                -0.5f32,
            ));
            vertices.push(Vec3A::new(
                points[i.0 as usize].0,
                points[i.0 as usize].1,
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
            indices.push(i.0);
            indices.push(i.1);
            indices.push(i.2);
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
            indices.push(i.0);
            indices.push(i.1);
            indices.push(i.2);

            indices.push(i.2 + flat_count);
            indices.push(i.1 + flat_count);
            indices.push(i.0 + flat_count);
        }

        // Add the vertices and indices in between the contours (e.g. in the z-axis).
        triangulate_between_edges_indexed(&mut vertices, &mut indices, &edges);

        Ok((vertices, indices))
    }
}

fn get_glyph_area_triangulation(
    outline: &GlyphOutline,
) -> Result<(Vec<TriangleIndices>, Vec<EdgeIndices>), Box<dyn MeshTextError>> {
    let triangulation = Triangulation::build_from_contours(&outline.points, &outline.contours)
        .map_err(|err| Box::new(GlyphTriangulationError(err)) as Box<dyn MeshTextError>)?;
    Ok((triangulation.triangles, triangulation.boundary_edges))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn split_constraints_make_watertight_extrusions() {
        // A triangular hole touches the middle of the outer square's lower
        // edge. The cap and side walls must share that newly split vertex.
        let outline = GlyphOutline {
            points: vec![
                (0., 0.),
                (10., 0.),
                (10., 10.),
                (0., 10.),
                (5., 0.),
                (6., 2.),
                (4., 2.),
            ],
            contours: vec![vec![0, 1, 2, 3, 0], vec![4, 5, 6, 4]],
        };
        let (vertices, indices) = raster_to_mesh_indexed(&outline, false).unwrap();
        let mut edges = BTreeMap::<(u32, u32), Vec<(u32, u32)>>::new();
        let mut volume = 0.0f64;
        for triangle in indices.chunks_exact(3) {
            let (a, b, c) = (triangle[0], triangle[1], triangle[2]);
            for (u, v) in [(a, b), (b, c), (c, a)] {
                edges
                    .entry(if u < v { (u, v) } else { (v, u) })
                    .or_default()
                    .push((u, v));
            }
            let (a, b, c) = (
                vertices[a as usize].as_dvec3(),
                vertices[b as usize].as_dvec3(),
                vertices[c as usize].as_dvec3(),
            );
            assert!((b - a).cross(c - a).length_squared() > 0.0);
            volume += a.dot(b.cross(c)) / 6.0;
        }
        // The touching hole has one non-manifold vertical edge at the contact
        // point. Every directed edge still has an oppositely directed partner.
        for directions in edges.values() {
            assert_eq!(
                directions.iter().filter(|&&(a, b)| a < b).count(),
                directions.iter().filter(|&&(a, b)| a > b).count()
            );
        }
        assert!((volume - 98.0).abs() < 1e-9, "signed volume {volume}");
        let vertices = raster_to_mesh(&outline, false).unwrap();
        let volume: f64 = vertices
            .chunks_exact(3)
            .map(|t| t[0].as_dvec3().dot(t[1].as_dvec3().cross(t[2].as_dvec3())) / 6.0)
            .sum();
        assert!((volume - 98.0).abs() < 1e-9);
    }

    #[test]
    fn redundant_contour_points_do_not_create_degenerate_walls() {
        let outline = GlyphOutline {
            points: vec![(0., 0.), (1., 0.), (1., 1.), (0., 1.), (1., 0.)],
            contours: vec![vec![0, 1, 4, 2, 3, 0]],
        };
        let (vertices, indices) = raster_to_mesh_indexed(&outline, false).unwrap();
        for triangle in indices.chunks_exact(3) {
            let (a, b, c) = (
                vertices[triangle[0] as usize],
                vertices[triangle[1] as usize],
                vertices[triangle[2] as usize],
            );
            assert!((b - a).cross(c - a).length_squared() > 0.0);
        }
        assert_eq!(indices.len(), 36);
    }
}
