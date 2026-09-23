use std::collections::VecDeque;

use super::{
    predicates::{incircle, on_segment, orient, proper_intersection},
    Edge, Error, Point, Triangulation,
};

const NONE: u32 = u32::MAX;
const FIXED: u8 = 1;
const ODD: u8 = 2;

/// Flat half-edge storage, with stable edge identities across flips. A triangle
/// is a three-edge cycle; twin links go directly to the neighboring triangle.
#[derive(Clone, Copy)]
struct HalfEdge {
    vertex: u32,
    dst: u32,
    next: u32,
    prev: u32,
    twin: u32,
    flags: u8,
}

pub(super) struct Mesh<'a> {
    points: &'a [Point],
    edges: Vec<HalfEdge>,
    outgoing: Vec<u32>,
    todo: Vec<u32>,
    crossed: VecDeque<u32>,
    pending: Vec<(Edge, bool)>,
}

impl<'a> Mesh<'a> {
    pub(super) fn build(
        points: &'a [Point],
        radial: &[u32],
        center: (f64, f64),
        constraints: &mut [(Edge, bool)],
    ) -> Result<Self, Error> {
        // The nearest points form the seed. If several lie on one line, order
        // that initial line monotonically before constructing its triangle fan.
        let mut reordered = Vec::new();
        let order = if radial.len() >= 3 {
            let seed = (2..radial.len()).find(|&i| {
                orient(
                    points[radial[0] as usize],
                    points[radial[1] as usize],
                    points[radial[i] as usize],
                ) != 0.0
            });
            if let Some(seed) = seed.filter(|&seed| seed > 2) {
                reordered.extend_from_slice(radial);
                sort_lexicographic(points, &mut reordered[..seed]);
                &reordered[..]
            } else {
                radial
            }
        } else {
            radial
        };
        match Self::sweep(points, order, Some(center), constraints) {
            Ok(mesh) => return Ok(mesh),
            Err(Error::InvalidTopology | Error::CrossingFixedEdge) => (),
            Err(error) => return Err(error),
        }
        // A rounded radial distance can tie a future vertex with a seed/hull
        // vertex. Retry with exact coordinate ordering, which also ensures all
        // intermediate vertices exist before any segment is installed.
        let mut lexical = radial.to_vec();
        sort_lexicographic(points, &mut lexical);
        Self::sweep(points, &lexical, None, constraints)
    }

    /// Extend the finite hull and legalize only affected edges. Radial order
    /// uses an angular hint; the lexicographic fallback starts at the previous
    /// hull vertex. Exact visibility tests validate both insertion orders.
    fn sweep(
        points: &'a [Point],
        sorted: &[u32],
        center: Option<(f64, f64)>,
        constraints: &mut [(Edge, bool)],
    ) -> Result<Self, Error> {
        let capacity = sorted.len().checked_mul(6).ok_or(Error::TooManyPoints)?;
        if capacity >= NONE as usize {
            return Err(Error::TooManyPoints);
        }
        let mut mesh = Self {
            points,
            edges: Vec::with_capacity(capacity),
            outgoing: vec![NONE; points.len()],
            todo: Vec::new(),
            crossed: VecDeque::new(),
            pending: Vec::new(),
        };
        if sorted.len() < 3 {
            return Ok(mesh);
        }
        let Some(seed) =
            (2..sorted.len()).find(|&i| mesh.turn(sorted[0], sorted[1], sorted[i]) != 0.0)
        else {
            return Ok(mesh);
        };
        // Use the outgoing scratch array for ranks before constructing edges.
        for (rank, &p) in sorted.iter().enumerate() {
            mesh.outgoing[p as usize] = rank as u32;
        }
        // Start each segment at its newly inserted (hull) endpoint. Its fan is
        // short; starting at an old interior endpoint can visit many triangles.
        for ((a, b), _) in constraints.iter_mut() {
            if mesh.outgoing[*a as usize] < mesh.outgoing[*b as usize] {
                std::mem::swap(a, b);
            }
        }
        if constraints.len() < 64 {
            constraints.sort_unstable_by_key(|&((a, b), _)| {
                (mesh.outgoing[a as usize], mesh.outgoing[b as usize])
            });
        } else {
            // Counting-sort constraint endings, like cdt's packed ending lists.
            // Ranks are dense, so this needs O(vertices + constraints) work.
            let mut offsets = vec![0; sorted.len() + 1];
            for &((a, _), _) in constraints.iter() {
                offsets[mesh.outgoing[a as usize] as usize + 1] += 1;
            }
            for i in 1..offsets.len() {
                offsets[i] += offsets[i - 1];
            }
            let mut ordered = vec![((0, 0), false); constraints.len()];
            for &constraint in constraints.iter() {
                let slot = &mut offsets[mesh.outgoing[constraint.0 .0 as usize] as usize];
                ordered[*slot] = constraint;
                *slot += 1;
            }
            constraints.copy_from_slice(&ordered);
        }
        mesh.outgoing.fill(NONE);
        let p = sorted[seed];
        let positive = mesh.turn(sorted[0], sorted[1], p) > 0.0;
        let mut previous = NONE;
        for pair in sorted[..seed].windows(2) {
            let [a, b] = [pair[0], pair[1]];
            let e = if positive {
                mesh.add(a, b, p)
            } else {
                mesh.add(b, a, p)
            };
            if previous != NONE {
                if positive {
                    mesh.link(previous + 1, e + 2);
                } else {
                    mesh.link(previous + 2, e + 1);
                }
            }
            previous = e;
        }
        let mut hull_next = vec![NONE; points.len()];
        let mut hull_prev = vec![NONE; points.len()];
        let mut hull_edge = vec![NONE; points.len()];
        for (id, edge) in mesh.edges.iter().enumerate() {
            if edge.twin == NONE {
                let a = edge.vertex;
                let b = edge.dst;
                hull_next[a as usize] = b;
                hull_prev[b as usize] = a;
                hull_edge[a as usize] = id as u32;
            }
        }
        let mut hash = vec![
            NONE;
            if center.is_some() {
                (sorted.len() / 2).next_power_of_two().clamp(16, 1024)
            } else {
                0
            }
        ];
        if let Some(center) = center {
            for &v in &sorted[..=seed] {
                let key = bucket(points[v as usize], center, hash.len());
                hash[key] = v;
            }
        }
        let mut next_constraint = 0;
        mesh.install_ready(constraints, &mut next_constraint)?;
        let mut last = p;
        for &p in &sorted[seed + 1..] {
            let visible = |a: u32| mesh.turn(a, hull_next[a as usize], p) < 0.0;
            let mut start = last;
            if let Some(center) = center {
                let key = bucket(points[p as usize], center, hash.len());
                for offset in 0..hash.len() {
                    let candidate = hash[(key + offset) & (hash.len() - 1)];
                    if candidate != NONE && hull_next[candidate as usize] != NONE {
                        start = candidate;
                        break;
                    }
                }
                let initial = start;
                let mut backward = start;
                loop {
                    if visible(start) {
                        break;
                    }
                    if backward != start && visible(backward) {
                        start = backward;
                        break;
                    }
                    start = hull_next[start as usize];
                    backward = hull_prev[backward as usize];
                    if start == initial {
                        // Rounded radial ordering can put a point on or within
                        // the hull. Rebuild using the exact-order sweep.
                        return Err(Error::InvalidTopology);
                    }
                }
            } else if !visible(start) {
                start = hull_prev[start as usize];
                if !visible(start) {
                    return Err(Error::InvalidTopology);
                }
            }
            while visible(hull_prev[start as usize]) {
                start = hull_prev[start as usize];
            }
            let mut end = hull_next[start as usize];
            while visible(end) {
                end = hull_next[end as usize];
            }
            let mut a = start;
            let mut previous = NONE;
            let mut first = NONE;
            while a != end {
                let b = hull_next[a as usize];
                let e = mesh.add(b, a, p);
                mesh.link(e, hull_edge[a as usize]);
                mesh.todo.push(e);
                if previous != NONE {
                    mesh.link(previous + 2, e + 1);
                } else {
                    first = e;
                }
                previous = e;
                if a != start {
                    hull_next[a as usize] = NONE;
                }
                a = b;
            }
            hull_next[start as usize] = p;
            hull_prev[p as usize] = start;
            hull_next[p as usize] = end;
            hull_prev[end as usize] = p;
            hull_edge[start as usize] = first + 1;
            hull_edge[p as usize] = previous + 2;
            mesh.install_ready(constraints, &mut next_constraint)?;
            if let Some(center) = center {
                let key = bucket(points[p as usize], center, hash.len());
                hash[key] = p;
                let key = bucket(points[start as usize], center, hash.len());
                hash[key] = start;
            }
            last = p;
        }
        if next_constraint != constraints.len() {
            return Err(Error::InvalidTopology);
        }
        Ok(mesh)
    }

    fn install_ready(
        &mut self,
        constraints: &[(Edge, bool)],
        index: &mut usize,
    ) -> Result<(), Error> {
        self.pending.clear();
        while let Some(&((a, b), odd)) = constraints.get(*index) {
            if self.outgoing[a as usize] == NONE || self.outgoing[b as usize] == NONE {
                break;
            }
            let mut a = a;
            loop {
                let (e, end) = self.start(a, b)?;
                if let Some(end) = end {
                    self.lock(e, odd);
                    if end == b {
                        break;
                    }
                    a = end;
                } else {
                    self.pending.push(((a, b), odd));
                    break;
                }
            }
            *index += 1;
        }
        self.legalize::<true>();
        for i in 0..self.pending.len() {
            let ((a, b), odd) = self.pending[i];
            self.constrain(a, b, odd)?;
        }
        Ok(())
    }

    #[inline]
    fn edge(&self, e: u32) -> HalfEdge {
        self.edges[e as usize]
    }

    #[inline]
    fn turn(&self, a: u32, b: u32, c: u32) -> f64 {
        orient(
            self.points[a as usize],
            self.points[b as usize],
            self.points[c as usize],
        )
    }

    fn add(&mut self, a: u32, b: u32, c: u32) -> u32 {
        let base = self.edges.len() as u32;
        for (i, vertex) in [a, b, c].into_iter().enumerate() {
            self.edges.push(HalfEdge {
                vertex,
                dst: [b, c, a][i],
                next: base + (i as u32 + 1) % 3,
                prev: base + (i as u32 + 2) % 3,
                twin: NONE,
                flags: 0,
            });
            self.outgoing[vertex as usize] = base + i as u32;
        }
        base
    }

    fn link(&mut self, a: u32, b: u32) {
        self.edges[a as usize].twin = b;
        self.edges[b as usize].twin = a;
        // Hull constraints must also apply to the newly added outside face.
        let flags = self.edges[a as usize].flags | self.edges[b as usize].flags;
        self.edges[a as usize].flags = flags;
        self.edges[b as usize].flags = flags;
    }

    /// Change diagonal ab to cd without reallocating or moving the four outer
    /// edges. Hull references, queued edges and installed constraints stay valid.
    fn flip(&mut self, e: u32) -> [u32; 4] {
        let left = self.edge(e);
        let t = left.twin;
        let right = self.edge(t);
        let (a, b) = (left.vertex, right.vertex);
        let (c, d) = (self.edge(left.prev).vertex, self.edge(right.prev).vertex);
        self.edges[e as usize].vertex = c;
        self.edges[e as usize].dst = d;
        self.edges[t as usize].vertex = d;
        self.edges[t as usize].dst = c;
        self.cycle(e, right.prev, left.next);
        self.cycle(t, left.prev, right.next);
        // Only the flipped diagonal changes its source vertices. Keep all
        // other anchors, especially the new point's boundary-edge anchor.
        if self.outgoing[a as usize] == e {
            self.outgoing[a as usize] = right.next;
        }
        if self.outgoing[b as usize] == t {
            self.outgoing[b as usize] = left.next;
        }
        [left.next, left.prev, right.next, right.prev]
    }

    fn cycle(&mut self, a: u32, b: u32, c: u32) {
        self.edges[a as usize].next = b;
        self.edges[a as usize].prev = c;
        self.edges[b as usize].next = c;
        self.edges[b as usize].prev = a;
        self.edges[c as usize].next = a;
        self.edges[c as usize].prev = b;
    }

    #[inline]
    fn illegal(&self, e: u32) -> bool {
        let left = self.edge(e);
        if left.twin == NONE || left.flags & FIXED != 0 {
            return false;
        }
        let right = self.edge(left.twin);
        let c = self.edge(left.prev).vertex;
        let d = self.edge(right.prev).vertex;
        // A strictly positive sign on a valid CCW mesh implies a convex quad.
        incircle(
            self.points[left.vertex as usize],
            self.points[left.dst as usize],
            self.points[c as usize],
            self.points[d as usize],
        ) > 0.0
    }

    fn legalize<const INSERTING: bool>(&mut self) {
        while let Some(e) = self.todo.pop() {
            if INSERTING {
                self.legalize_insert(e, 0);
            } else if self.illegal(e) {
                let affected = self.flip(e);
                self.todo.extend(affected);
            }
        }
    }

    /// Follow the two edges opposite the inserted vertex directly, as in cdt.
    /// Bound call-stack use; longer chains go onto the ordinary work stack and
    /// are completely processed by the caller (no iteration/work limit).
    fn legalize_insert(&mut self, mut e: u32, depth: u8) {
        while self.illegal(e) {
            let affected = self.flip(e);
            if depth < 32 {
                self.legalize_insert(affected[2], depth + 1);
            } else {
                self.todo.push(affected[2]);
            }
            e = affected[3];
        }
    }

    fn crosses(&self, a: u32, b: u32, e: u32) -> bool {
        let edge = self.edge(e);
        proper_intersection(
            self.points[a as usize],
            self.points[b as usize],
            self.points[edge.vertex as usize],
            self.points[edge.dst as usize],
        )
    }

    /// Find the existing edge ab or the first edge crossed by ab, visiting only
    /// the triangles incident to a. A boundary vertex needs both fan directions.
    fn start(&self, a: u32, b: u32) -> Result<(u32, Option<u32>), Error> {
        let first = self.outgoing[a as usize];
        let mut e = first;
        let mut reverse = false;
        loop {
            let edge = self.edge(e);
            let next = edge.dst;
            let prev = self.edge(edge.prev).vertex;
            if next == b {
                return Ok((e, Some(b)));
            }
            if prev == b {
                return Ok((edge.prev, Some(b)));
            }
            let next_turn = self.turn(a, next, b);
            if next_turn >= 0.0 {
                if next_turn == 0.0
                    && on_segment(
                        self.points[a as usize],
                        self.points[b as usize],
                        self.points[next as usize],
                    )
                {
                    return Ok((e, Some(next)));
                }
                let prev_turn = self.turn(a, prev, b);
                if prev_turn == 0.0
                    && on_segment(
                        self.points[a as usize],
                        self.points[b as usize],
                        self.points[prev as usize],
                    )
                {
                    return Ok((edge.prev, Some(prev)));
                }
                // b is an existing mesh vertex. Once it lies strictly inside
                // this angular wedge, it must be beyond the opposite edge: a
                // valid triangle cannot contain another vertex in its interior.
                if next_turn > 0.0 && prev_turn < 0.0 {
                    return Ok((edge.next, None));
                }
            }
            if !reverse {
                e = self.edge(edge.prev).twin;
                if e == first {
                    break;
                }
                if e == NONE {
                    reverse = true;
                    e = first;
                } else {
                    continue;
                }
            }
            let twin = self.edge(e).twin;
            if twin == NONE {
                break;
            }
            e = self.edge(twin).next;
        }
        Err(Error::InvalidTopology)
    }

    fn lock(&mut self, e: u32, odd: bool) {
        let flags = (self.edge(e).flags | FIXED) ^ if odd { ODD } else { 0 };
        self.edges[e as usize].flags = flags;
        let twin = self.edge(e).twin;
        if twin != NONE {
            self.edges[twin as usize].flags = flags;
        }
    }

    fn constrain(&mut self, mut a: u32, b: u32, odd: bool) -> Result<(), Error> {
        self.crossed.clear();
        while a != b {
            let (mut e, end) = self.start(a, b)?;
            if let Some(end) = end {
                self.lock(e, odd);
                a = end;
                continue;
            }
            // Stop at the next vertex on the segment, not just its final
            // endpoint. This is the PointOnFixedEdge repair, performed as part
            // of the local triangle walk rather than an all-points prepass.
            let end = loop {
                let edge = self.edge(e);
                if edge.flags & FIXED != 0 {
                    return Err(Error::CrossingFixedEdge);
                }
                if edge.twin == NONE {
                    return Err(Error::InvalidTopology);
                }
                self.crossed.push_back(e);
                let other = self.edge(edge.twin);
                let vertex = self.edge(other.prev).vertex;
                if vertex == b
                    || on_segment(
                        self.points[a as usize],
                        self.points[b as usize],
                        self.points[vertex as usize],
                    )
                {
                    break vertex;
                }
                if self.crosses(a, b, other.next) {
                    e = other.next;
                } else if self.crosses(a, b, other.prev) {
                    e = other.prev;
                } else {
                    return Err(Error::InvalidTopology);
                }
            };
            let mut deferred = 0;
            while let Some(e) = self.crossed.pop_front() {
                let edge = self.edge(e);
                let twin = self.edge(edge.twin);
                let c = self.edge(edge.prev).vertex;
                let d = self.edge(twin.prev).vertex;
                if self.turn(c, d, twin.vertex) <= 0.0 || self.turn(d, c, edge.vertex) <= 0.0 {
                    self.crossed.push_back(e);
                    deferred += 1;
                    if deferred >= self.crossed.len() {
                        return Err(Error::InvalidTopology);
                    }
                    continue;
                }
                let affected = self.flip(e);
                self.todo.extend(affected);
                self.todo.push(e);
                deferred = 0;
                if self.crosses(a, end, e) {
                    self.crossed.push_back(e);
                }
            }
            let (e, found) = self.start(a, end)?;
            if found != Some(end) {
                return Err(Error::InvalidTopology);
            }
            self.lock(e, odd);
            self.legalize::<false>();
            a = end;
        }
        Ok(())
    }

    pub(super) fn finish(mut self, constrained: bool) -> Result<Triangulation, Error> {
        const OUTSIDE: u8 = 4;
        const INSIDE: u8 = 8;
        const SEEN: u8 = OUTSIDE | INSIDE;
        if constrained {
            // Reuse the legalization stack and half-edge flag byte. Mark faces
            // on enqueue, so every triangle enters the flood stack only once.
            for e in 0..self.edges.len() {
                let edge = self.edges[e];
                if edge.twin == NONE {
                    let marker = if edge.flags & ODD != 0 {
                        INSIDE
                    } else {
                        OUTSIDE
                    };
                    self.mark_face(e as u32, marker, SEEN)?;
                }
            }
            while let Some(e) = self.todo.pop() {
                let face = self.edge(e);
                let marker = face.flags & SEEN;
                for e in [e, face.next, face.prev] {
                    let edge = self.edge(e);
                    if edge.twin != NONE {
                        let next = if edge.flags & ODD != 0 {
                            marker ^ SEEN
                        } else {
                            marker
                        };
                        self.mark_face(edge.twin, next, SEEN)?;
                    }
                }
            }
        }
        let mut result = Triangulation {
            triangles: Vec::with_capacity(self.edges.len() / 3),
            boundary_edges: Vec::with_capacity(self.points.len()),
        };
        for (i, edge) in self.edges.iter().enumerate() {
            if constrained && edge.flags & INSIDE == 0 {
                continue;
            }
            if (i as u32) < edge.next && (i as u32) < edge.prev {
                result
                    .triangles
                    .push((edge.vertex, edge.dst, self.edge(edge.prev).vertex));
            }
            if (constrained && edge.flags & ODD != 0) || (!constrained && edge.twin == NONE) {
                result.boundary_edges.push((edge.dst, edge.vertex));
            }
        }
        Ok(result)
    }

    #[inline]
    fn mark_face(&mut self, e: u32, marker: u8, mask: u8) -> Result<(), Error> {
        let edge = self.edge(e);
        let old = edge.flags & mask;
        if old == 0 {
            for e in [e, edge.next, edge.prev] {
                self.edges[e as usize].flags |= marker;
            }
            self.todo.push(e);
        } else if old != marker {
            return Err(Error::OpenContour);
        }
        Ok(())
    }
}

// Approximate angular lookup is only a starting hint for an exact hull walk.
fn bucket(p: Point, center: (f64, f64), count: usize) -> usize {
    let (x, y) = (p.0 as f64 - center.0, p.1 as f64 - center.1);
    let sum = x.abs() + y.abs();
    let angle = if sum == 0.0 {
        0.0
    } else {
        let fraction = x / sum;
        (if y >= 0.0 {
            1.0 - fraction
        } else {
            3.0 + fraction
        }) * 0.25
    };
    ((angle * count as f64) as usize).min(count - 1)
}

fn sort_lexicographic(points: &[Point], order: &mut [u32]) {
    order.sort_unstable_by(|&a, &b| {
        let (a, b) = (points[a as usize], points[b as usize]);
        a.0.partial_cmp(&b.0)
            .unwrap()
            .then_with(|| a.1.partial_cmp(&b.1).unwrap())
    });
}
