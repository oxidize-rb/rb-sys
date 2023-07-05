use criterion::Criterion;
use rb_sys_test_helpers::eval;
use std::hint::black_box;

use crate::StableApiBenchExt;

pub fn run(c: &mut Criterion) {
    let arrays = [
        ("8", eval!("['a', 'a', 'a', 'a', 'a', 'a', 'a', 'a']")),
        ("64", eval!("['abc'] * 64")),
        ("256", eval!("['abc'] * 256")),
    ];

    c.bench_abi_function(
        "rarray to slice (raw)",
        arrays.iter().copied(),
        |api, string| unsafe {
            let ptr = api.rarray_const_ptr(black_box(string));
            let len = api.rarray_len(black_box(string));
            std::slice::from_raw_parts(ptr as _, len as _)
        },
    );

    c.bench_abi_function(
        "rarray to slice (iter)",
        arrays.iter().copied(),
        |api, string| unsafe {
            let ptr = api.rarray_const_ptr(black_box(string));
            let len = api.rarray_len(black_box(string));
            let slice = std::slice::from_raw_parts(ptr as _, len as _);

            slice.iter().map(|v| *v as usize).sum::<usize>()
        },
    );
}
