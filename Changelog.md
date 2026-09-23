# Changelog

## Unreleased
* Replace `cdt` with a standalone constrained Delaunay triangulation module using native `f32` points and `u32` indices.
* Split constraints at collinear input vertices, fixing the former `PointOnFixedEdge` failure and keeping extruded sides aligned with caps.
* Add geometry/property tests, complex boundary regressions and font/extrusion integration tests.
* Correct indexed text-section offsets when triangulation leaves unused duplicate vertices.
* Add an isolated release benchmark comparing the replacement with `cdt`, including the old conversion overhead.
* Optimize triangulation with a radial hull sweep, flat half-edges, local legalization, packed constraint endings and walk-based segment splitting; see `benches/triangulation-compare/RESULTS.md` for measured comparisons.
* **Breaking:** `GlyphTriangulationError` now wraps `meshtext::triangulation::Error`; triangle ordering can change.
* Upgrade all direct dependencies to their latest stable releases, including `glam` 0.33.8, `pollster` 1.0.1 and `wgpu` 30.0.1.
* Adapt the rendering examples to the wgpu 30 instance, surface and presentation APIs.
* **Breaking:** Public bounding-box vectors now use `glam` 0.33 types.
* **Breaking:** Exposed font types, metadata methods and parser errors now use skrifa.
* Replace `ttf-parser` and `owned_ttf_parser` with `skrifa` for both font modes.
* Preserve `owned` constructors taking `Vec<u8>` with a parsed, movable `OwnedFace`.
* Allow non-static font buffers in borrowed constructors.
* Handle implicitly closed outlines and report outline drawing failures.
---
## Version 0.3.2
* Allow accessing parsed `FontFace`
* Update `glam` dependency
---
## Version 0.3.1
* Add the option to substitute a glyph with a custom mesh in the glyph cache
---
## Version 0.3.0
* Add `owned` feature using [`owned_ttf_parser`](https://crates.io/crates/owned_ttf_parser) for _more flexibility_ when loading fonts
---
## Version 0.2.1
* Add a method for cleaning the internal glyph cache
---
## Version 0.2.0
* Add functionality for generating 2D meshes
* Add benchmarks using [criterion](https://crates.io/crates/criterion)
* Shrink crate size by including only a subset of glyphs in the built-in font
* Fix function signatures
---
## Version 0.1.1
* Add pre-caching functionality
---
