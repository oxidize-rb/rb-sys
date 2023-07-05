use criterion::criterion_group;

pub mod stable_abi_bench;

criterion_group!(benches, stable_abi_bench::run);
