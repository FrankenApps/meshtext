//! Constrained Delaunay triangulation of planar points and closed boundaries.
//!
//! This module is independent of meshtext, fonts and glam. It can be extracted
//! into a dependency-free crate by copying this directory and making this file
//! `lib.rs`.
//!
//! Points and indices use the same representation as glyph outlines. Output
//! indices refer to the original input; no Steiner points are introduced.
//! Triangles have counterclockwise winding. Closed boundaries use the even-odd
//! fill rule, independent of contour winding. Constraints are split at existing
//! vertices on their segments, including vertices belonging to other contours.
//! Properly crossing constraints without an input vertex at the intersection
//! are rejected. Duplicate coordinates are identified exactly, without a
//! tolerance that could erase small features.

mod engine;
mod predicates;
#[cfg(test)]
mod tests;

use std::{error, fmt};

use engine::Mesh;

/// A position in the plane. Both coordinates must be finite.
pub type Point = (f32, f32);
/// Indices of the endpoints of a segment.
pub type Edge = (u32, u32);
/// Counterclockwise triangle indices.
pub type Triangle = (u32, u32, u32);

/// Invalid geometry or an internal topology inconsistency.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// No points were supplied.
    EmptyInput,
    /// Fewer than three points were supplied.
    TooFewPoints,
    /// A point contains infinity or NaN.
    InvalidInput,
    /// The point count or internal half-edge capacity exceeds the `u32` range.
    TooManyPoints,
    /// An edge references a nonexistent point.
    InvalidEdge,
    /// A contour is not closed, or the boundary graph has odd-degree vertices.
    OpenContour,
    /// Two constraints cross without an existing vertex at the intersection.
    CrossingFixedEdge,
    /// A constraint could not be recovered or the mesh topology is inconsistent.
    InvalidTopology,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::EmptyInput => "input cannot be empty",
            Self::TooFewPoints => "at least three points are required",
            Self::InvalidInput => "points must have finite coordinates",
            Self::TooManyPoints => "point or half-edge count exceeds the u32 index range",
            Self::InvalidEdge => "edge index is out of bounds",
            Self::OpenContour => "boundaries must be closed",
            Self::CrossingFixedEdge => "constraints cross without a vertex at their intersection",
            Self::InvalidTopology => "triangulation topology is inconsistent",
        })
    }
}
impl error::Error for Error {}

/// A triangulated region and its normalized boundary.
#[derive(Clone, Debug, Default)]
pub struct Triangulation {
    /// Counterclockwise triangles indexing the original point array.
    pub triangles: Vec<Triangle>,
    /// Boundary edges with the filled region on their right.
    ///
    /// Edges are split at collinear input vertices. Duplicate coordinates use
    /// their first input index. Shared internal boundaries are omitted. This
    /// makes these edges suitable for extruding a watertight triangle mesh.
    pub boundary_edges: Vec<Edge>,
}

impl Triangulation {
    /// Triangulates the convex hull of a point cloud.
    ///
    /// Duplicate points are merged. Collinear input has an empty triangulation.
    pub fn build(points: &[Point]) -> Result<Self, Error> {
        let input = canonicalize(points)?;
        Mesh::build(points, &input.order, input.center, &mut [])?.finish(false)
    }

    /// Triangulates closed contours with an even-odd fill rule.
    ///
    /// Every nonempty contour must repeat its initial index at its end.
    /// Repeated vertices and zero-length segments are ignored. With no contour
    /// segments, the filled region is empty.
    pub fn build_from_contours(points: &[Point], contours: &[Vec<u32>]) -> Result<Self, Error> {
        let mut edges =
            Vec::with_capacity(contours.iter().map(|c| c.len().saturating_sub(1)).sum());
        for contour in contours {
            if contour.first() != contour.last() {
                return Err(Error::OpenContour);
            }
            if contour.iter().any(|&i| i as usize >= points.len()) {
                return Err(Error::InvalidEdge);
            }
            edges.extend(contour.windows(2).map(|pair| (pair[0], pair[1])));
        }
        if edges.is_empty() {
            canonicalize(points)?;
            return Ok(Self::default());
        }
        Self::build_with_edges(points, &edges)
    }

    /// Triangulates edges forming closed boundaries, independent of edge order
    /// and direction. An empty edge list selects the whole convex hull.
    ///
    /// Coincident segments count with multiplicity under the even-odd rule.
    /// Nonempty boundaries that cancel out therefore produce no triangles.
    pub fn build_with_edges(points: &[Point], edges: &[Edge]) -> Result<Self, Error> {
        let input = canonicalize(points)?;
        let canonical = &input.canonical;
        let mut constraints = Vec::with_capacity(edges.len());
        let mut degree = vec![false; points.len()];
        for &(a, b) in edges {
            let (&a, &b) = canonical
                .get(a as usize)
                .zip(canonical.get(b as usize))
                .ok_or(Error::InvalidEdge)?;
            if a == b {
                continue;
            }
            constraints.push(((a, b), true));
            degree[a as usize] ^= true;
            degree[b as usize] ^= true;
        }
        if degree.iter().any(|&odd| odd) {
            return Err(Error::OpenContour);
        }
        Mesh::build(points, &input.order, input.center, &mut constraints)?.finish(!edges.is_empty())
    }
}

/// Triangulates the convex hull, returning indices into `points`.
pub fn triangulate_points(points: &[Point]) -> Result<Vec<Triangle>, Error> {
    Ok(Triangulation::build(points)?.triangles)
}

/// Triangulates closed contours, returning only filled triangles.
pub fn triangulate_contours(
    points: &[Point],
    contours: &[Vec<u32>],
) -> Result<Vec<Triangle>, Error> {
    Ok(Triangulation::build_from_contours(points, contours)?.triangles)
}

/// Triangulates closed boundary edges, returning only filled triangles.
pub fn triangulate_with_edges(points: &[Point], edges: &[Edge]) -> Result<Vec<Triangle>, Error> {
    Ok(Triangulation::build_with_edges(points, edges)?.triangles)
}

#[cfg(test)]
fn edge_key(a: u32, b: u32) -> Edge {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

struct Input {
    canonical: Vec<u32>,
    order: Vec<u32>,
    center: (f64, f64),
}

fn canonicalize(points: &[Point]) -> Result<Input, Error> {
    if points.is_empty() {
        return Err(Error::EmptyInput);
    }
    if points.len() < 3 {
        return Err(Error::TooFewPoints);
    }
    if points.len() > u32::MAX as usize {
        return Err(Error::TooManyPoints);
    }
    if points
        .iter()
        .any(|&(x, y)| !x.is_finite() || !y.is_finite())
    {
        return Err(Error::InvalidInput);
    }
    let (mut xmin, mut ymin) = (f64::INFINITY, f64::INFINITY);
    let (mut xmax, mut ymax) = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    for &(x, y) in points {
        xmin = xmin.min(x as f64);
        xmax = xmax.max(x as f64);
        ymin = ymin.min(y as f64);
        ymax = ymax.max(y as f64);
    }
    let center = ((xmin + xmax) * 0.5, (ymin + ymax) * 0.5);
    let mut sorted: Vec<_> = points
        .iter()
        .enumerate()
        .map(|(i, &(x, y))| {
            let (x, y) = (x as f64 - center.0, y as f64 - center.1);
            (i as u32, x * x + y * y)
        })
        .collect();
    // One sort supplies both exact duplicate groups and the radial sweep order.
    // Coordinate tie breakers keep identical positions adjacent, including ±0.
    sorted.sort_unstable_by(|a, b| {
        a.1.total_cmp(&b.1).then_with(|| {
            let (a, b) = (points[a.0 as usize], points[b.0 as usize]);
            a.0.partial_cmp(&b.0)
                .unwrap()
                .then_with(|| a.1.partial_cmp(&b.1).unwrap())
        })
    });
    let mut canonical = vec![0; points.len()];
    let mut unique = Vec::with_capacity(points.len());
    let mut start = 0;
    while start < sorted.len() {
        let mut end = start + 1;
        while end < sorted.len()
            && points[sorted[end].0 as usize] == points[sorted[start].0 as usize]
        {
            end += 1;
        }
        let first = sorted[start..end].iter().map(|&(i, _)| i).min().unwrap();
        unique.push(first);
        for &(i, _) in &sorted[start..end] {
            canonical[i as usize] = first;
        }
        start = end;
    }
    Ok(Input {
        canonical,
        order: unique,
        center,
    })
}
