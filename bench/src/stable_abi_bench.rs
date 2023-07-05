use criterion::{BenchmarkId, Criterion};
use rb_sys::{
    rb_gc_register_mark_object,
    stable_api::{Compiled, StableApiDefinition},
    StableApi, VALUE,
};
use rb_sys_test_helpers::eval;
use std::hint::black_box;

#[inline(always)]
unsafe fn str_new_checked<'a, T: StableApiDefinition>(rstring: VALUE) -> &'a str {
    let ptr = T::rstring_ptr(rstring);
    let len = T::rstring_len(rstring);

    unsafe { std::str::from_utf8(std::slice::from_raw_parts(ptr as _, len as _)) }
        .expect("valid utf8")
}

#[inline(always)]
unsafe fn str_new_unchecked<'a, T: StableApiDefinition>(rstring: VALUE) -> &'a str {
    let ptr = T::rstring_ptr(rstring);
    let len = T::rstring_len(rstring);

    unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr as _, len as _)) }
}

pub fn run(c: &mut Criterion) {
    let strings = [
        (4, eval!("'aaaa'")),
        (64, eval!("'a' * 64")),
        (1024, eval!("'a' * 1024")),
    ];

    for (_size, string) in strings {
        unsafe { rb_gc_register_mark_object(string) };
    }

    {
        let mut group = c.benchmark_group("str from rstring (checked)");

        for (size, string) in strings {
            group.throughput(criterion::Throughput::Bytes(size));

            group.bench_function(BenchmarkId::new("C", size), |b| {
                b.iter(|| unsafe { str_new_checked::<Compiled>(black_box(string)) })
            });

            group.bench_function(BenchmarkId::new("Rust", size), |b| {
                b.iter(|| unsafe { str_new_checked::<StableApi>(black_box(string)) })
            });
        }

        group.finish();
    }

    {
        let mut group = c.benchmark_group("str from rstring (unchecked)");

        for (size, string) in strings {
            group.throughput(criterion::Throughput::Bytes(size));

            group.bench_function(BenchmarkId::new("C", size), |b| {
                b.iter(|| unsafe { str_new_unchecked::<Compiled>(black_box(string)) })
            });

            group.bench_function(BenchmarkId::new("Rust", size), |b| {
                b.iter(|| unsafe { str_new_unchecked::<StableApi>(black_box(string)) })
            });
        }

        group.finish();
    }
}
