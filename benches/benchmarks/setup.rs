use criterion::{criterion_group, Criterion};
use meshtext::MeshGenerator;

/// Measures the time required to instantiate a new
/// [MeshGenerator].
///
/// Arguments:
///
/// * `c`: The [Criterion] benchmark manager.
fn setup_benchmark(c: &mut Criterion) {
    let font_data = include_bytes!("../../assets/font/FiraMono-Regular.ttf");

    c.bench_function("setup", |b| {
        b.iter(|| {
            let _generator = MeshGenerator::new(font_data);
        });
    });
}

criterion_group!(benches, setup_benchmark);
