use criterion::criterion_main;

mod benchmarks {
    pub(crate) mod alphabet;
    pub(crate) mod cached;
    pub(crate) mod setup;
}
pub(crate) use benchmarks::*;

criterion_main! {
    alphabet::benches,
    cached::benches,
    setup::benches,
}
