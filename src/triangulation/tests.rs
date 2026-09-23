use super::predicates::{on_segment, proper_intersection};
use super::*;
use std::collections::{BTreeMap, BTreeSet};

fn cross(a: Point, b: Point, c: Point) -> f64 {
    (b.0 as f64 - a.0 as f64) * (c.1 as f64 - a.1 as f64)
        - (b.1 as f64 - a.1 as f64) * (c.0 as f64 - a.0 as f64)
}

fn polygon_area(points: &[Point], path: &[u32]) -> f64 {
    path.windows(2)
        .map(|p| {
            let (a, b) = (points[p[0] as usize], points[p[1] as usize]);
            a.0 as f64 * b.1 as f64 - a.1 as f64 * b.0 as f64
        })
        .sum::<f64>()
        .abs()
        * 0.5
}

fn area(points: &[Point], mesh: &Triangulation) -> f64 {
    mesh.triangles
        .iter()
        .map(|&(a, b, c)| cross(points[a as usize], points[b as usize], points[c as usize]) * 0.5)
        .sum()
}

fn closed(start: u32, count: u32) -> Vec<u32> {
    (start..start + count)
        .chain(std::iter::once(start))
        .collect()
}

fn edges(contours: &[Vec<u32>]) -> Vec<Edge> {
    contours
        .iter()
        .flat_map(|c| c.windows(2).map(|p| (p[0], p[1])))
        .collect()
}

/// Independent, deliberately slow geometry oracle. Check area, orientation,
/// manifold edges, no overlapping faces, actual boundary, and local Delaunay.
fn verify(points: &[Point], mesh: &Triangulation, constraints: &[Edge], expected_area: f64) {
    let actual = area(points, mesh);
    assert!(
        (actual - expected_area).abs() <= expected_area.abs() * 1e-6 + 1e-10,
        "area {actual} != {expected_area}"
    );
    let mut adjacent = BTreeMap::<Edge, Vec<u32>>::new();
    for &(a, b, c) in &mesh.triangles {
        assert!([a, b, c].iter().all(|&i| (i as usize) < points.len()));
        assert!(cross(points[a as usize], points[b as usize], points[c as usize]) > 0.0);
        if !constraints.is_empty() {
            // Independently classify the centroid with an even-odd ray cast.
            let x = (points[a as usize].0 as f64
                + points[b as usize].0 as f64
                + points[c as usize].0 as f64)
                / 3.0;
            let y = (points[a as usize].1 as f64
                + points[b as usize].1 as f64
                + points[c as usize].1 as f64)
                / 3.0;
            let mut parity = false;
            for &(u, v) in constraints {
                let (p, q) = (points[u as usize], points[v as usize]);
                if (p.1 as f64 > y) != (q.1 as f64 > y) {
                    let intersection = p.0 as f64
                        + (y - p.1 as f64) * (q.0 as f64 - p.0 as f64) / (q.1 as f64 - p.1 as f64);
                    if x < intersection {
                        parity = !parity;
                    }
                }
            }
            assert!(parity, "triangle lies in an unfilled region");
        }
        for (u, v, w) in [(a, b, c), (b, c, a), (c, a, b)] {
            adjacent.entry(edge_key(u, v)).or_default().push(w);
        }
    }
    for (&(a, b), opposite) in &adjacent {
        assert!(opposite.len() <= 2, "non-manifold edge");
        if opposite.len() == 2
            && !constraints.iter().any(|&(u, v)| {
                on_segment(points[u as usize], points[v as usize], points[a as usize])
                    && on_segment(points[u as usize], points[v as usize], points[b as usize])
            })
        {
            let (c, d) = (opposite[0], opposite[1]);
            let (a, b, c, d) = (
                points[a as usize],
                points[b as usize],
                points[c as usize],
                points[d as usize],
            );
            // Direct f64 determinant is an independent oracle on the bounded
            // coordinates used in randomized tests. Allow rounding at zero.
            let (ax, ay) = (a.0 as f64 - d.0 as f64, a.1 as f64 - d.1 as f64);
            let (bx, by) = (b.0 as f64 - d.0 as f64, b.1 as f64 - d.1 as f64);
            let (cx, cy) = (c.0 as f64 - d.0 as f64, c.1 as f64 - d.1 as f64);
            let det = (ax * ax + ay * ay) * (bx * cy - by * cx)
                - (bx * bx + by * by) * (ax * cy - ay * cx)
                + (cx * cx + cy * cy) * (ax * by - ay * bx);
            assert!(
                det * cross(a, b, c).signum() <= 1e-7,
                "illegal Delaunay edge: {a:?} {b:?}, {det}"
            );
        }
    }
    let actual_boundary: BTreeSet<_> = adjacent
        .iter()
        .filter(|(_, opposite)| opposite.len() == 1)
        .map(|(&e, _)| e)
        .collect();
    let reported: BTreeSet<_> = mesh
        .boundary_edges
        .iter()
        .map(|&(a, b)| edge_key(a, b))
        .collect();
    assert_eq!(actual_boundary, reported);
    assert_eq!(reported.len(), mesh.boundary_edges.len());
    let mut degree = BTreeMap::<u32, i32>::new();
    for &(a, b) in &mesh.boundary_edges {
        *degree.entry(a).or_default() += 1;
        *degree.entry(b).or_default() -= 1;
        let c = adjacent[&edge_key(a, b)][0];
        assert!(
            cross(points[a as usize], points[b as usize], points[c as usize]) < 0.0,
            "boundary must keep solid on right"
        );
    }
    assert!(degree.values().all(|&balance| balance == 0));
    for (i, &(a, b, c)) in mesh.triangles.iter().enumerate() {
        let t = [points[a as usize], points[b as usize], points[c as usize]];
        for &(d, e, f) in &mesh.triangles[i + 1..] {
            let u = [points[d as usize], points[e as usize], points[f as usize]];
            for p in t {
                assert!(
                    !(cross(u[0], u[1], p) > 0.0
                        && cross(u[1], u[2], p) > 0.0
                        && cross(u[2], u[0], p) > 0.0),
                    "overlapping triangles"
                );
            }
            for p in u {
                assert!(
                    !(cross(t[0], t[1], p) > 0.0
                        && cross(t[1], t[2], p) > 0.0
                        && cross(t[2], t[0], p) > 0.0),
                    "overlapping triangles"
                );
            }
            for j in 0..3 {
                for k in 0..3 {
                    assert!(
                        !proper_intersection(t[j], t[(j + 1) % 3], u[k], u[(k + 1) % 3]),
                        "crossing triangle edges"
                    );
                }
            }
        }
    }
}

#[test]
fn triangle_square_and_convex_cloud() {
    let points = [
        (0., 0.),
        (2., 0.),
        (2., 2.),
        (0., 2.),
        (1., 1.),
        (1., 0.),
        (0., 1.),
    ];
    let mesh = Triangulation::build(&points).unwrap();
    verify(&points, &mesh, &[], 4.0);
    assert_eq!(mesh.triangles.len(), 6);
    assert_eq!(triangulate_points(&points).unwrap(), mesh.triangles);
    let triangle = Triangulation::build(&points[..3]).unwrap();
    verify(&points, &triangle, &[], 2.0);
}

#[test]
fn long_collinear_sweep_seed_preserves_every_boundary_vertex() {
    for swap_axes in [false, true] {
        for sign in [-1.0, 1.0] {
            let mut points: Vec<Point> = (0..17).map(|i| (i as f32, 0.0)).collect();
            points.extend([(17.0, 10.0), (18.0, 0.0)]);
            for p in &mut points {
                p.1 *= sign;
                if swap_axes {
                    *p = (p.1, p.0);
                }
            }
            let contours = vec![vec![0, 18, 17, 0]];
            let mesh = Triangulation::build_from_contours(&points, &contours).unwrap();
            verify(&points, &mesh, &edges(&contours), 90.0);
            assert_eq!(mesh.boundary_edges.len(), points.len());
            assert_eq!(mesh.triangles.len(), points.len() - 2);
        }
    }
}

#[test]
fn rounded_radial_distances_fall_back_without_losing_a_vertex() {
    // All four squared distances to (0.5, 0.5) round to 0.5 in f64. The last
    // point is on the seed hull, so inserting it as an exterior point would
    // fail. The lexicographic fallback must keep this subnormal-sized edge.
    let points = [(0., 0.), (1., 0.), (0., 1.), (f32::from_bits(1), 0.)];
    // Supply a tied radial order explicitly to exercise the defensive fallback;
    // the public canonicalizer's coordinate tie breaker can avoid this order.
    let mesh = engine::Mesh::build(&points, &[0, 1, 2, 3], (0.5, 0.5), &mut [])
        .unwrap()
        .finish(false)
        .unwrap();
    verify(&points, &mesh, &[], 0.5);
    assert_eq!(mesh.triangles.len(), 2);
    let boundary: BTreeSet<_> = mesh
        .boundary_edges
        .iter()
        .map(|&(a, b)| edge_key(a, b))
        .collect();
    assert_eq!(boundary, BTreeSet::from([(0, 2), (0, 3), (1, 2), (1, 3)]));
}

#[test]
fn concave_polygon() {
    let points = [
        (0., 0.),
        (5., 0.),
        (5., 1.),
        (1., 1.),
        (1., 4.),
        (5., 4.),
        (5., 5.),
        (0., 5.),
    ];
    let contours = vec![closed(0, 8)];
    let mesh = Triangulation::build_from_contours(&points, &contours).unwrap();
    verify(&points, &mesh, &edges(&contours), 13.0);
    assert_eq!(
        triangulate_contours(&points, &contours).unwrap(),
        mesh.triangles
    );
}

#[test]
fn holes_nested_islands_and_disconnected_components() {
    let points = [
        (0., 0.),
        (10., 0.),
        (10., 10.),
        (0., 10.),
        (2., 2.),
        (8., 2.),
        (8., 8.),
        (2., 8.),
        (4., 4.),
        (6., 4.),
        (6., 6.),
        (4., 6.),
        (12., 0.),
        (14., 0.),
        (14., 2.),
        (12., 2.),
    ];
    for reverse in [false, true] {
        let mut contours = vec![closed(0, 4), closed(4, 4), closed(8, 4), closed(12, 4)];
        if reverse {
            contours[0].reverse();
            contours[2].reverse();
            contours.reverse();
        }
        let mesh = Triangulation::build_from_contours(&points, &contours).unwrap();
        verify(&points, &mesh, &edges(&contours), 72.0);
    }
}

#[test]
fn point_137_on_fixed_edge_is_split() {
    let mut points = vec![(0., 0.), (100., 0.), (100., 100.), (0., 100.)];
    for i in 0..133 {
        points.push((1.0 + (i % 13) as f32 * 7.0, 1.0 + (i / 13) as f32 * 7.0));
    }
    points.push((50., 0.));
    assert_eq!(points.len(), 138);
    let contour = vec![closed(0, 4)];
    let mesh = Triangulation::build_from_contours(&points, &contour).unwrap();
    verify(&points, &mesh, &edges(&contour), 10000.0);
    let boundary: BTreeSet<_> = mesh
        .boundary_edges
        .iter()
        .map(|&(a, b)| edge_key(a, b))
        .collect();
    assert!(boundary.contains(&(0, 137)));
    assert!(boundary.contains(&(1, 137)));
    assert!(!boundary.contains(&(0, 1)));
}

#[test]
fn touching_hole_and_t_junction() {
    let points = [
        (0., 0.),
        (10., 0.),
        (10., 10.),
        (0., 10.),
        (5., 0.),
        (6., 2.),
        (4., 2.),
    ];
    let contours = vec![closed(0, 4), closed(4, 3)];
    let mesh = Triangulation::build_from_contours(&points, &contours).unwrap();
    verify(&points, &mesh, &edges(&contours), 98.0);
    assert_eq!(mesh.boundary_edges.len(), 8);
}

#[test]
fn coincident_vertices_zero_edges_and_signed_zero() {
    let points = [(0., 0.), (2., 0.), (2., 2.), (0., 2.), (-0., 0.), (2., 0.)];
    let contours = vec![vec![0, 4, 5, 1, 2, 3, 0]];
    let mesh = Triangulation::build_from_contours(&points, &contours).unwrap();
    verify(&points, &mesh, &edges(&contours), 4.0);
    assert!(mesh
        .triangles
        .iter()
        .all(|&(a, b, c)| [a, b, c].iter().all(|&i| i < 4)));
}

#[test]
fn shared_and_overlapping_collinear_boundaries_cancel() {
    let points = [
        (0., 0.),
        (4., 0.),
        (4., 4.),
        (0., 4.),
        (4., 1.),
        (6., 1.),
        (6., 3.),
        (4., 3.),
    ];
    let contours = vec![closed(0, 4), closed(4, 4)];
    let mesh = Triangulation::build_from_contours(&points, &contours).unwrap();
    verify(&points, &mesh, &edges(&contours), 20.0);
    assert!(!mesh
        .boundary_edges
        .iter()
        .any(|&(a, b)| edge_key(a, b) == (4, 7)));
    let doubled = vec![closed(0, 4), closed(0, 4)];
    let empty = Triangulation::build_from_contours(&points, &doubled).unwrap();
    verify(&points, &empty, &[], 0.0);
}

#[test]
fn crossing_constraints_need_an_existing_intersection_vertex() {
    let mut points = vec![(0., 0.), (2., 2.), (0., 2.), (2., 0.)];
    let contours = vec![closed(0, 4)];
    assert_eq!(
        Triangulation::build_from_contours(&points, &contours).unwrap_err(),
        Error::CrossingFixedEdge
    );
    points.push((1., 1.));
    let mesh = Triangulation::build_from_contours(&points, &contours).unwrap();
    verify(&points, &mesh, &edges(&contours), 2.0);
}

#[test]
fn malformed_inputs_return_errors() {
    assert_eq!(triangulate_points(&[]), Err(Error::EmptyInput));
    assert_eq!(
        triangulate_points(&[(0., 0.), (1., 0.)]),
        Err(Error::TooFewPoints)
    );
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(
            triangulate_points(&[(0., 0.), (1., 0.), (bad, 1.)]),
            Err(Error::InvalidInput)
        );
    }
    let points = [(0., 0.), (1., 0.), (0., 1.)];
    assert_eq!(
        triangulate_with_edges(&points, &[(0, 9)]),
        Err(Error::InvalidEdge)
    );
    assert_eq!(
        triangulate_contours(&points, &[vec![0, 1, 2]]),
        Err(Error::OpenContour)
    );
    assert_eq!(
        triangulate_with_edges(&points, &[(0, 1)]),
        Err(Error::OpenContour)
    );
    assert_eq!(
        triangulate_contours(&points, &[vec![9]]),
        Err(Error::InvalidEdge)
    );
}

#[test]
fn degenerate_inputs_are_empty() {
    for points in [
        vec![(1., 1.); 5],
        vec![(0., 0.), (1., 0.), (2., 0.), (3., 0.)],
    ] {
        assert!(triangulate_points(&points).unwrap().is_empty());
        assert!(
            triangulate_contours(&points, &[closed(0, points.len() as u32)])
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn exterior_points_do_not_fill_holes_or_exterior() {
    let points = [
        (0., 0.),
        (2., 0.),
        (2., 2.),
        (0., 2.),
        (-5., -5.),
        (6., 7.),
        (1., 1.),
    ];
    let contours = vec![closed(0, 4)];
    let mesh = Triangulation::build_from_contours(&points, &contours).unwrap();
    verify(&points, &mesh, &edges(&contours), 4.0);
    assert!(mesh
        .triangles
        .iter()
        .all(|&(a, b, c)| ![a, b, c].contains(&4) && ![a, b, c].contains(&5)));
}

#[test]
fn cocircular_grid_is_deterministic() {
    let points: Vec<_> = (0..10)
        .flat_map(|y| (0..10).map(move |x| (x as f32, y as f32)))
        .collect();
    let first = Triangulation::build(&points).unwrap();
    verify(&points, &first, &[], 81.0);
    assert_eq!(first.triangles.len(), 162);
    assert_eq!(
        first.triangles,
        Triangulation::build(&points).unwrap().triangles
    );
}

struct Random(u64);
impl Random {
    fn next(&mut self) -> f32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 40) as f32 / (1u32 << 24) as f32
    }
}

#[test]
fn filtered_predicates_match_exact_signs_across_f32_exponents() {
    let check = |[a, b, c, d]: [Point; 4]| {
        assert_eq!(
            predicates::orient(a, b, c).partial_cmp(&0.0),
            predicates::orient_exact(a, b, c).partial_cmp(&0.0)
        );
        assert_eq!(
            predicates::incircle(a, b, c, d).partial_cmp(&0.0),
            predicates::incircle_exact(a, b, c, d).partial_cmp(&0.0)
        );
    };
    let mut random = Random(918273);
    for _ in 0..10_000 {
        let mut coordinate = || {
            random.next();
            // All finite signs/exponents/mantissas, including subnormals.
            let mut bits = (random.0 >> 32) as u32;
            if bits & 0x7f800000 == 0x7f800000 {
                bits ^= 0x00800000;
            }
            f32::from_bits(bits)
        };
        check(std::array::from_fn(|_| (coordinate(), coordinate())));
    }
    for exponent in [-140, -100, -50, 0, 50, 100, 125] {
        let s = 2.0f32.powi(exponent);
        for delta in [-1, 0, 1] {
            let near = f32::from_bits((s.to_bits() as i64 + delta) as u32);
            check([(0., 0.), (s, 0.), (0., s), (s, near)]);
            check([(0., 0.), (s, s), (s * 2., near * 2.), (-s, s)]);
        }
    }
}

#[test]
fn predicates_match_independent_integer_determinants() {
    // Integer coordinates bounded by 1024 keep these determinants far inside
    // i128. Powers of two preserve the exact geometry across f32's range,
    // including subnormals, without sharing any expansion arithmetic helpers.
    let check = |points: [(i128, i128); 4]| {
        let [a, b, c, d] = points;
        let orientation = (a.0 - c.0) * (b.1 - c.1) - (a.1 - c.1) * (b.0 - c.0);
        let (ax, ay) = (a.0 - d.0, a.1 - d.1);
        let (bx, by) = (b.0 - d.0, b.1 - d.1);
        let (cx, cy) = (c.0 - d.0, c.1 - d.1);
        let circle = (ax * ax + ay * ay) * (bx * cy - by * cx)
            + (bx * bx + by * by) * (cx * ay - cy * ax)
            + (cx * cx + cy * cy) * (ax * by - ay * bx);
        for exponent in [-149, -100, -20, 0, 20, 100, 116] {
            let scale = 2.0f64.powi(exponent) as f32;
            let [a, b, c, d] = points.map(|(x, y)| (x as f32 * scale, y as f32 * scale));
            assert_eq!(
                predicates::orient(a, b, c).partial_cmp(&0.0),
                Some(orientation.cmp(&0))
            );
            assert_eq!(
                predicates::orient(b, a, c).partial_cmp(&0.0),
                Some((-orientation).cmp(&0))
            );
            assert_eq!(
                predicates::incircle(a, b, c, d).partial_cmp(&0.0),
                Some(circle.cmp(&0))
            );
            assert_eq!(
                predicates::incircle(b, a, c, d).partial_cmp(&0.0),
                Some((-circle).cmp(&0))
            );
            assert_eq!(
                predicates::orient_exact(a, b, c).partial_cmp(&0.0),
                Some(orientation.cmp(&0))
            );
            assert_eq!(
                predicates::incircle_exact(a, b, c, d).partial_cmp(&0.0),
                Some(circle.cmp(&0))
            );
        }
    };
    // Exact collinearity, cocircularity, repeated points and both circle sides.
    check([(1, 1), (12, 12), (24, 24), (48, 48)]);
    check([(5, 0), (0, 5), (-5, 0), (0, -5)]);
    check([(5, 0), (0, 5), (-5, 0), (0, -4)]);
    check([(5, 0), (0, 5), (-5, 0), (0, -6)]);
    check([(1, 1); 4]);
    let mut random = Random(238941);
    for _ in 0..1000 {
        let mut coordinate = || (random.next() * 2049.0) as i128 - 1024;
        check(std::array::from_fn(|_| (coordinate(), coordinate())));
    }
}

#[test]
fn randomized_star_shaped_polygons() {
    for seed in 0..150 {
        let mut random = Random(seed);
        let count = 8 + seed as usize % 40;
        let points: Vec<_> = (0..count)
            .map(|i| {
                let angle = i as f64 * std::f64::consts::TAU / count as f64;
                let radius = 2.0 + random.next() as f64 * 8.0;
                ((radius * angle.cos()) as f32, (radius * angle.sin()) as f32)
            })
            .collect();
        let contours = vec![closed(0, count as u32)];
        let mesh = Triangulation::build_from_contours(&points, &contours)
            .unwrap_or_else(|error| panic!("seed {seed}: {error:?}"));
        verify(
            &points,
            &mesh,
            &edges(&contours),
            polygon_area(&points, &contours[0]),
        );
        assert_eq!(mesh.triangles.len(), count - 2);
    }
}

#[test]
fn randomized_point_clouds_with_known_hull() {
    for seed in 0..50 {
        let mut random = Random(seed);
        let mut points = vec![(0., 0.), (10., 0.), (10., 10.), (0., 10.)];
        points.extend((0..50).map(|_| (random.next() * 10., random.next() * 10.)));
        let mesh = Triangulation::build(&points).unwrap();
        verify(&points, &mesh, &[], 100.0);
    }
}

#[test]
fn shuffled_constraints_and_vertex_order() {
    let points = [
        (0., 0.),
        (8., 0.),
        (8., 8.),
        (0., 8.),
        (2., 2.),
        (6., 2.),
        (6., 6.),
        (2., 6.),
    ];
    let source = edges(&[closed(0, 4), closed(4, 4)]);
    for shift in 0..points.len() {
        let shuffled: Vec<_> = (0..points.len())
            .map(|i| points[(i + shift) % points.len()])
            .collect();
        let mut constraints: Vec<_> = source
            .iter()
            .map(|&(a, b)| {
                (
                    ((b as usize + points.len() - shift) % points.len()) as u32,
                    ((a as usize + points.len() - shift) % points.len()) as u32,
                )
            })
            .collect();
        constraints.reverse();
        let mesh = Triangulation::build_with_edges(&shuffled, &constraints).unwrap();
        verify(&shuffled, &mesh, &constraints, 48.0);
    }
}

#[test]
fn nearly_collinear_and_extreme_scales() {
    for scale in [1e-30f32, 1e-10, 1., 1e10, 1e30] {
        let points = [
            (0., 0.),
            (scale, 0.),
            (scale, scale * 1e-5),
            (0., scale * 1e-5),
            (scale * 0.5, scale * 1e-6),
        ];
        let mesh = Triangulation::build(&points).unwrap();
        assert_eq!(mesh.triangles.len(), 4);
        let expected = points[1].0 as f64 * points[2].1 as f64;
        assert!((area(&points, &mesh) / expected - 1.0).abs() < 1e-6);
        for &(a, b, c) in &mesh.triangles {
            assert!(cross(points[a as usize], points[b as usize], points[c as usize]) > 0.0);
        }
    }
}

#[test]
fn randomized_constrained_clouds() {
    for seed in 0..300 {
        let mut random = Random(seed + 1234);
        let count = 10 + seed as usize % 30;
        let mut points: Vec<_> = (0..count)
            .map(|i| {
                let angle = i as f64 * std::f64::consts::TAU / count as f64;
                let radius = 3.0 + random.next() as f64 * 7.0;
                ((radius * angle.cos()) as f32, (radius * angle.sin()) as f32)
            })
            .collect();
        points.extend((0..40).map(|_| ((random.next() - 0.5) * 20., (random.next() - 0.5) * 20.)));
        let contours = vec![closed(0, count as u32)];
        let mesh = Triangulation::build_from_contours(&points, &contours)
            .unwrap_or_else(|error| panic!("seed {seed}: {error:?}"));
        verify(
            &points,
            &mesh,
            &edges(&contours),
            polygon_area(&points, &contours[0]),
        );
    }
}

#[test]
fn many_holes_with_dense_collinear_boundaries() {
    let mut points = vec![(0., 0.), (100., 0.), (100., 100.), (0., 100.)];
    let mut contours = vec![closed(0, 4)];
    for row in 0..8 {
        for column in 0..8 {
            let (x, y) = (2.0 + column as f32 * 12., 2.0 + row as f32 * 12.);
            let start = points.len() as u32;
            points.extend([
                (x, y),
                (x + 4., y),
                (x + 8., y),
                (x + 8., y + 8.),
                (x + 4., y + 8.),
                (x, y + 8.),
            ]);
            contours.push(closed(start, 6));
        }
    }
    let mesh = Triangulation::build_from_contours(&points, &contours).unwrap();
    verify(&points, &mesh, &edges(&contours), 10000.0 - 64.0 * 64.0);
}

#[test]
fn a_constrained_diagonal_is_preserved_even_when_not_delaunay() {
    let points = [(0., 0.), (4., 0.), (3., 2.), (0., 4.)];
    let unconstrained = Triangulation::build(&points).unwrap();
    let uses_ac = unconstrained
        .triangles
        .iter()
        .any(|&(a, b, c)| [a, b, c].contains(&0) && [a, b, c].contains(&2));
    let diagonal = if uses_ac { (1, 3) } else { (0, 2) };
    let mut constraints = edges(&[closed(0, 4)]);
    // An internal constraint has even boundary multiplicity, so it does not
    // change the fill region but must still survive Delaunay legalization.
    constraints.extend([diagonal, diagonal]);
    let mesh = Triangulation::build_with_edges(&points, &constraints).unwrap();
    assert!(mesh
        .triangles
        .iter()
        .all(|&(a, b, c)| [a, b, c].contains(&diagonal.0) && [a, b, c].contains(&diagonal.1)));
    verify(
        &points,
        &mesh,
        &constraints,
        polygon_area(&points, &closed(0, 4)),
    );
}

#[test]
fn near_edge_vertices_are_not_snapped() {
    let points = [
        (0., 0.),
        (1., 0.),
        (1., 1.),
        (0., 1.),
        (0.5, f32::from_bits(1)),
    ];
    let contours = vec![closed(0, 4)];
    let mesh = Triangulation::build_from_contours(&points, &contours).unwrap();
    assert_eq!(mesh.triangles.len(), 4);
    assert_eq!(mesh.boundary_edges.len(), 4);
    assert!(mesh.boundary_edges.iter().all(|&(a, b)| a != 4 && b != 4));
    verify(&points, &mesh, &edges(&contours), 1.0);
}

#[test]
fn absent_or_collapsed_contours_have_no_fill() {
    let points = [(0., 0.), (1., 0.), (1., 1.), (0., 1.)];
    for contours in [vec![], vec![vec![]], vec![vec![0]], vec![vec![0, 0]]] {
        assert!(Triangulation::build_from_contours(&points, &contours)
            .unwrap()
            .triangles
            .is_empty());
    }
    // The edge API explicitly selects the hull when there are no constraints.
    assert_eq!(triangulate_with_edges(&points, &[]).unwrap().len(), 2);
}

#[test]
fn randomized_internal_constraint_fans() {
    for seed in 0..200 {
        let mut random = Random(seed + 789);
        let mut points = vec![(0., 0.), (10., 0.), (10., 10.), (0., 10.), (5., 5.)];
        points.extend((0..35).map(|_| (random.next() * 10., random.next() * 10.)));
        let mut constraints = edges(&[closed(0, 4)]);
        for i in 5..25 {
            constraints.extend([(4, i), (i, 4)]);
        }
        let mesh = Triangulation::build_with_edges(&points, &constraints)
            .unwrap_or_else(|error| panic!("fan seed {seed}: {error:?}"));
        verify(&points, &mesh, &constraints, 100.0);
        // Seed 11 produces a crossed triangle strip containing a whole vertex
        // fan. Recovery must preserve that interior vertex, not just the strip's
        // outer boundary. All input vertices here lie in the filled region.
        let used: BTreeSet<_> = mesh
            .triangles
            .iter()
            .flat_map(|&(a, b, c)| [a, b, c])
            .collect();
        assert_eq!(used.len(), points.len());
        let mesh_edges: BTreeSet<_> = mesh
            .triangles
            .iter()
            .flat_map(|&(a, b, c)| [edge_key(a, b), edge_key(b, c), edge_key(c, a)])
            .collect();
        for i in 5..25 {
            assert!(mesh_edges.contains(&(4, i)));
        }
    }
}

#[test]
#[ignore = "long-running randomized constraint recovery stress test"]
fn stress_ten_thousand_constraint_fans() {
    for seed in 0..10_000 {
        let mut random = Random(seed + 912345);
        let mut points = vec![(0., 0.), (10., 0.), (10., 10.), (0., 10.), (5., 5.)];
        points.extend((0..50).map(|_| (random.next() * 10., random.next() * 10.)));
        let mut constraints = edges(&[closed(0, 4)]);
        for i in 5..35 {
            constraints.extend([(4, i), (i, 4)]);
        }
        let mesh = Triangulation::build_with_edges(&points, &constraints)
            .unwrap_or_else(|error| panic!("stress seed {seed}: {error:?}"));
        assert!((area(&points, &mesh) - 100.0).abs() < 1e-6);
        let used: BTreeSet<_> = mesh
            .triangles
            .iter()
            .flat_map(|&(a, b, c)| [a, b, c])
            .collect();
        assert_eq!(used.len(), points.len(), "stress seed {seed} lost a vertex");
        let mesh_edges: BTreeSet<_> = mesh
            .triangles
            .iter()
            .flat_map(|&(a, b, c)| [edge_key(a, b), edge_key(b, c), edge_key(c, a)])
            .collect();
        for i in 5..35 {
            assert!(mesh_edges.contains(&(4, i)));
        }
    }
}
