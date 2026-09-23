# Exact predicate signs for finite f32 coordinates

`exact.rs` is an original, safe Rust implementation of orientation and in-circle
signs. It has no dependencies, heap allocations, unsafe code, generated code, or
vendored routines. It uses the repository's MIT/Apache-2.0 licenses. The parent
`predicates.rs` retains the existing fast filters; only uncertain signs need an
exact expansion. This directory can move with the triangulation module into its
own dependency-free crate.

## Contract and formulas

Inputs are **finite f32 coordinates**, checked by the triangulator before use.
Only the determinant's **sign and exact zero** are meaningful; the returned f64
is not necessarily its magnitude. Positive orientation means counterclockwise.
For counterclockwise `(a,b,c)`, positive in-circle means `d` is inside the circle;
reversing orientation reverses that sign. There is no geometric tolerance.
This is not a replacement for predicates accepting arbitrary f64 inputs.

Orientation accumulates the six products in

```text
ax*by - ay*bx + bx*cy - by*cx + cx*ay - cy*ax.
```

In-circle expands the determinant with rows `[x, y, x²+y², 1]` along its third
column:

```text
 lift(a)*orient(b,c,d) - lift(b)*orient(a,c,d)
+lift(c)*orient(a,b,d) - lift(d)*orient(a,b,c).
```

Each lift is handled as two separate squares. These unshifted formulas avoid
having to represent rounded coordinate differences and their tails. The fast
filters still use translated coordinates and the established conservative error
bounds (`4 * f64::EPSILON` and `16 * f64::EPSILON`). For orientation, a zero
product is also safe: no underflow is possible, so a factor is exactly zero and
the remaining product determines the sign.

## Arithmetic argument

An expansion is a sequence of nonoverlapping f64 components ordered by increasing
magnitude. Their real sum is exact. We use the standard TwoSum error-free
transform and Grow-Expansion operation with zero elimination. TwoSum returns a
rounded sum and its exact residual without needing an operand-magnitude ordering.
Adding a scalar to an expansion preserves its exact sum and nonoverlap; the last
nonzero component determines the sign. The loop writes only at or behind the
component being read, so compaction does not overwrite unread values.

For a product, `product = left * right` and
`error = left.mul_add(right, -product)` recover the two exact components.
Rust's [`f64::mul_add`](https://doc.rust-lang.org/std/primitive.f64.html#method.mul_add)
guarantees a single correctly rounded fused operation. Hardware FMA is not a
correctness requirement. The standard error-free product result applies because
neither the product nor its residual can underflow or overflow in this domain.

The f32 restriction supplies the bounds:

- Every finite coordinate has at most 24 significant binary digits and magnitude
  less than `2^128`. A product of two widened coordinates, including a square,
  has at most 48 significant digits and is **exact in f64** (53 digits).
- Each coordinate is an integer multiple of `2^-149`. Orientation terms and
  expansion components are therefore multiples of `2^-298`; in-circle terms
  and components are multiples of `2^-596`. Rounding and the error-free
  transforms preserve these lattices. Any nonzero residual is at least
  `2^-596`, well above the smallest normal f64, `2^-1022`.
- An orientation has six terms of magnitude less than `2^256`. Its components
  are bounded conservatively by `2^260`. Multiplying one by a coordinate square
  gives magnitude below `2^516`. Even 96 such accumulated terms and the
  TwoSum intermediates stay below the conservative bound `2^530`, far below
  f64 overflow near `2^1024`.
- Grow-Expansion increases length by at most one per scalar addition.
  Orientation needs at most **6 components**. In-circle has four cofactors,
  each with at most six components, multiplied by two squares, each producing
  at most two product components: **4 × 6 × 2 × 2 = 96 components**. Zero
  elimination only reduces these lengths. Storage bounds do not depend on
  cancellation or exponent separation.

These bounds justify the arithmetic hypotheses and fixed capacities for the
whole finite-f32 domain, including subnormals and signed zero. As with standard
floating-point error-free transforms, this assumes IEEE round-to-nearest
arithmetic. Do not enable reassociation/fast-math, replace TwoSum expressions
with fused operations, or lower intermediate precision. The explicit fused
operation in `add_product` is intentional.

The mathematical basis is described in Jonathan Richard Shewchuk's
[Adaptive Precision Floating-Point Arithmetic and Fast Robust Geometric Predicates](https://people.eecs.berkeley.edu/~jrs/papers/robust-predicates.pdf)
(TwoSum and Grow-Expansion). This implementation uses those general arithmetic
ideas, not the old crate's source or its adaptive predicate implementation.

## Verification

Normal `cargo test` includes:

- Unit tests of half-ULP residuals, fused product residuals, zero elimination,
  and cancellation across the full degree-four exponent span.
- All **6,561 quadruples** on a 3-by-3 integer grid at three scales, including
  the smallest subnormal and a near-maximum exponent: **19,683 configurations**.
  Signs are computed independently with i128 determinants.
- **14,656 fixtures** generated using Python arbitrary-precision integers.
  Each f32 is decoded directly from bits to an integer multiple of `2^-149`;
  translated determinants are evaluated with integer arithmetic, with no
  floating-point determinant or expansion routines. Cases cover all finite
  exponent ranges, one-ULP perturbations, exact degeneracies, extremes, and
  signed zero. Both filtered and unfiltered exact signs are checked, as are
  reversed signs under odd permutations.
- The triangulation module's existing scaled-i128 oracle, 10,000 random
  full-exponent predicate cases, and geometry/integration regressions.

The binary fixture contains eight little-endian u32 coordinate bit patterns
followed by two signed sign bytes per record (34 bytes). Python is needed only
to regenerate or audit it, not to build or test the crate:

```sh
python3 src/triangulation/predicates/fixtures/generate.py --check
cargo test --offline
cargo test --offline --features owned
cargo test --offline --lib stress_ten_thousand_constraint_fans -- --ignored
```

Fixture SHA-256:
`345e8c3b3391e06855fd639f2d0fa789e05d145ff66777b86cb1ffb774627eb4`.

An isolated [differential verifier](../../../benches/predicate-verify/README.md)
also compares **1,000,000 cases** against the previous library's adaptive and
exact predicates. Its legacy dependency is outside meshtext's dependency graph.
The integer oracle is independent of that library.

This combines a mathematical range/capacity argument with reproducible tests.
It is not a machine-checked formal proof or exhaustive enumeration of every
finite f32 tuple. Numerical changes should rerun these checks, the constraint
stress test, default/owned integration tests, and the paired performance suite.
