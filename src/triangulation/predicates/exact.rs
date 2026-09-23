//! Exact determinant signs for finite f32 coordinates, using short expansions.
//!
//! An expansion stores nonoverlapping f64 components, smallest first. Its sum
//! is exact; the last nonzero component determines its sign. See README.md for
//! the range/capacity proofs and the mathematical references.

#![forbid(unsafe_code)]

use super::super::Point;

/// Rounded sum and its exact rounding error (Knuth's TwoSum).
fn two_sum(left: f64, right: f64) -> (f64, f64) {
    let sum = left + right;
    let right_virtual = sum - left;
    let left_virtual = sum - right_virtual;
    let right_error = right - right_virtual;
    let left_error = left - left_virtual;
    (sum, left_error + right_error)
}

struct Expansion<const N: usize> {
    components: [f64; N],
    len: usize,
}

impl<const N: usize> Expansion<N> {
    fn new() -> Self {
        Self {
            components: [0.0; N],
            len: 0,
        }
    }

    /// Add one scalar exactly, eliminating zero components. TwoSum propagates
    /// the rounded part upward and leaves each exact residual behind.
    fn add(&mut self, mut value: f64) {
        if value == 0.0 {
            return;
        }
        let mut written = 0;
        for read in 0..self.len {
            let (sum, error) = two_sum(value, self.components[read]);
            if error != 0.0 {
                self.components[written] = error;
                written += 1;
            }
            value = sum;
        }
        if value != 0.0 {
            self.components[written] = value;
            written += 1;
        }
        self.len = written;
    }

    /// The fused operation recovers the exact product's residual. Our f32
    /// determinant bounds exclude f64 overflow and underflow in both terms.
    fn add_product(&mut self, left: f64, right: f64) {
        let product = left * right;
        let error = left.mul_add(right, -product);
        self.add(error);
        self.add(product);
    }

    fn leading(&self) -> f64 {
        self.components[..self.len].last().copied().unwrap_or(0.0)
    }
}

fn orientation(a: Point, b: Point, c: Point) -> Expansion<6> {
    let mut determinant = Expansion::new();
    // Each product of two f32 coordinates has at most 48 significant bits,
    // so it is exact in f64. Six scalar additions require at most six slots.
    for (p, q) in [(a, b), (b, c), (c, a)] {
        determinant.add(f64::from(p.0) * f64::from(q.1));
        determinant.add(-f64::from(p.1) * f64::from(q.0));
    }
    determinant
}

/// Only the sign (including exact zero) is part of the predicate contract.
pub(in super::super) fn orient(a: Point, b: Point, c: Point) -> f64 {
    orientation(a, b, c).leading()
}

/// Expand det([x, y, x²+y², 1]) along the squared-length column. Each of four
/// cofactors has at most six components, multiplied by two exact coordinate
/// squares; each product contributes at most two terms: 4 * 6 * 2 * 2 = 96.
pub(in super::super) fn incircle(a: Point, b: Point, c: Point, d: Point) -> f64 {
    let mut determinant = Expansion::<96>::new();
    for (point, minor, sign) in [
        (a, [b, c, d], 1.0),
        (b, [a, c, d], -1.0),
        (c, [a, b, d], 1.0),
        (d, [a, b, c], -1.0),
    ] {
        let cofactor = orientation(minor[0], minor[1], minor[2]);
        for coordinate in [point.0, point.1] {
            let square = f64::from(coordinate) * f64::from(coordinate) * sign;
            if square != 0.0 {
                for &component in &cofactor.components[..cofactor.len] {
                    determinant.add_product(component, square);
                }
            }
        }
    }
    determinant.leading()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_sum_keeps_rounding_tails_and_cancellation() {
        let half_ulp = 2.0f64.powi(-53);
        assert_eq!(two_sum(1.0, half_ulp), (1.0, half_ulp));
        assert_eq!(two_sum(half_ulp, 1.0), (1.0, half_ulp));
        assert_eq!(two_sum(1.0, -1.0), (0.0, 0.0));
        let tiny = 2.0f64.powi(-298);
        let large = 2.0f64.powi(256);
        assert_eq!(two_sum(large, tiny), (large, tiny));
        assert_eq!(two_sum(-large, tiny), (-large, tiny));
    }

    #[test]
    fn expansion_preserves_small_terms_through_large_cancellation() {
        let tiny = 2.0f64.powi(-596);
        let large = 2.0f64.powi(512);
        for terms in [
            [large, tiny, -large],
            [tiny, large, -large],
            [-large, tiny, large],
        ] {
            let mut sum = Expansion::<3>::new();
            for term in terms {
                sum.add(term);
            }
            assert_eq!(sum.leading(), tiny);
            assert_eq!(sum.len, 1);
            sum.add(-tiny);
            assert_eq!(sum.leading(), 0.0);
            assert_eq!(sum.len, 0);
        }
    }

    #[test]
    fn product_residual_survives_cancellation() {
        let a = 1.0 + 2.0f64.powi(-27);
        let rounded = 1.0 + 2.0f64.powi(-26);
        for sign in [-1.0, 1.0] {
            let mut sum = Expansion::<3>::new();
            sum.add_product(sign * a, a);
            sum.add(-sign * rounded);
            assert_eq!(sum.leading(), sign * 2.0f64.powi(-54));
        }
    }
}