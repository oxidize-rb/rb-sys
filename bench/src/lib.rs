use criterion::{criterion_group, BenchmarkId, Criterion};
use rb_sys::rb_gc_register_mark_object;
use rb_sys::stable_api::{get_default, get_fallback};
use rb_sys::StableApiDefinition;

pub mod stable_abi_rarray;
pub mod stable_abi_rstring;

pub trait StableApiBenchExt {
    fn bench_abi_function<O>(
        &mut self,
        name: &str,
        iter: impl Iterator<Item = (&'static str, rb_sys::VALUE)>,
        func: impl FnMut(&'static dyn StableApiDefinition, rb_sys::VALUE) -> O,
    );
}

impl StableApiBenchExt for Criterion {
    fn bench_abi_function<O>(
        &mut self,
        name: &str,
        iter: impl Iterator<Item = (&'static str, rb_sys::VALUE)>,
        mut func: impl FnMut(&'static dyn StableApiDefinition, rb_sys::VALUE) -> O,
    ) {
        if cfg!(not(ruby_lte_3_2)) {
            panic!("This benchmark is only supported on stable Ruby versions, please use a different version of Ruby.");
        }

        let mut group = self.benchmark_group(name);

        group.noise_threshold(0.02);
        group.sample_size(1000);

        for (tag, value) in iter {
            unsafe { rb_gc_register_mark_object(value) };

            group.bench_function(BenchmarkId::new("C", tag), |b| {
                b.iter(|| func(get_fallback(), value))
            });

            group.bench_function(BenchmarkId::new("Rust", tag), |b| {
                b.iter(|| func(get_default(), value))
            });
        }

        group.finish();
    }
}

criterion_group!(benches, stable_abi_rstring::run, stable_abi_rarray::run);
