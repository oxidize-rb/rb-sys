use criterion::{black_box, BenchmarkId, Criterion, Throughput};
use rb_sys::rb_utf8_str_new;
use rb_sys_test_helpers::eval;

pub fn run(c: &mut Criterion) {
    let items = (2..=12).map(|i| 2usize.pow(i));

    let mut group = c.benchmark_group("baselines (rb_utf8_str_new)");

    for len in items {
        group.throughput(Throughput::Bytes(len as _));

        group.bench_with_input(BenchmarkId::from_parameter(len), &len, |b, &size| {
            let data = "a".repeat(size);
            let batch_size = 1000;
            let mut iterations = 0;

            b.iter_batched(
                || {
                    iterations += 1;

                    if iterations % batch_size == 0 {
                        eval!("GC.start(full_mark: true, immediate_sweep: true)");
                    }
                    data.clone()
                },
                |data| unsafe {
                    let string = black_box(data);
                    rb_utf8_str_new(string.as_ptr() as _, string.len() as _)
                },
                criterion::BatchSize::NumIterations(batch_size),
            );
        });
    }
    group.finish();
}
