use std::time::{Duration, Instant};

use criterion::{criterion_group, Criterion};
use meshtext::MeshGenerator;

/// Measures the time required to precache the american
/// alphabet for 3D characters.
///
/// Arguments:
///
/// * `c`: The [Criterion] benchmark manager.
fn precache_benchmark(c: &mut Criterion) {
    let font_data = include_bytes!("../../assets/font/FiraMono-Regular.ttf");

    c.bench_function("precache alphabet", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;

            for _ in 0..iters {
                let mut generator = MeshGenerator::new(font_data);

                let start = Instant::now();
                generator
                    .precache_glyphs(
                        "ABCDEFHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
                        false,
                        None,
                    )
                    .expect("Failed to precache characters.");
                total += start.elapsed();
            }

            total
        });
    });
}

criterion_group!(benches, precache_benchmark);
