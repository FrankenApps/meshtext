#[path = "predicates/exact.rs"]
mod exact;

#[cfg(test)]
pub(super) use exact::{incircle as incircle_exact, orient as orient_exact};

use super::Point;

// Widen individual operands exactly, never copy/convert the input point buffer.
// Exact predicates are necessary for nearly collinear/cocircular data.
#[inline]
pub(super) fn orient(a: Point, b: Point, c: Point) -> f64 {
    let (ax, ay) = (a.0 as f64 - c.0 as f64, a.1 as f64 - c.1 as f64);
    let (bx, by) = (b.0 as f64 - c.0 as f64, b.1 as f64 - c.1 as f64);
    let (left, right) = (ax * by, ay * bx);
    let det = left - right;
    if left == 0.0 || right == 0.0 {
        return det;
    }
    // Conservative versions of Shewchuk's first-stage error bounds. All f32
    // inputs widen exactly; their degree-2/4 products cannot overflow/underflow
    // f64. Uncertain signs, including zero, still use the exact predicate.
    if det.abs() > (left.abs() + right.abs()) * (4.0 * f64::EPSILON) {
        return det;
    }
    exact::orient(a, b, c)
}

#[inline]
pub(super) fn incircle(a: Point, b: Point, c: Point, d: Point) -> f64 {
    let (ax, ay) = (a.0 as f64 - d.0 as f64, a.1 as f64 - d.1 as f64);
    let (bx, by) = (b.0 as f64 - d.0 as f64, b.1 as f64 - d.1 as f64);
    let (cx, cy) = (c.0 as f64 - d.0 as f64, c.1 as f64 - d.1 as f64);
    let (aa, bb, cc) = (ax * ax + ay * ay, bx * bx + by * by, cx * cx + cy * cy);
    let (bc, cb, ca, ac, ab, ba) = (bx * cy, cx * by, cx * ay, ax * cy, ax * by, bx * ay);
    let det = aa * (bc - cb) + bb * (ca - ac) + cc * (ab - ba);
    let magnitude =
        aa * (bc.abs() + cb.abs()) + bb * (ca.abs() + ac.abs()) + cc * (ab.abs() + ba.abs());
    if det.abs() > magnitude * (16.0 * f64::EPSILON) {
        return det;
    }
    exact::incircle(a, b, c, d)
}

#[inline]
pub(super) fn on_segment(a: Point, b: Point, p: Point) -> bool {
    p.0 >= a.0.min(b.0)
        && p.0 <= a.0.max(b.0)
        && p.1 >= a.1.min(b.1)
        && p.1 <= a.1.max(b.1)
        && orient(a, b, p) == 0.0
}

#[inline]
pub(super) fn proper_intersection(a: Point, b: Point, c: Point, d: Point) -> bool {
    let opposite = |x: f64, y: f64| (x < 0.0 && y > 0.0) || (x > 0.0 && y < 0.0);
    opposite(orient(a, b, c), orient(a, b, d)) && opposite(orient(c, d, a), orient(c, d, b))
}
