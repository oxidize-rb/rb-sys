use criterion::Criterion;
use rb_sys::{rb_ary_new_capa, rb_ary_push, rb_funcall, rb_intern2};
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
        "rarray to slice (iter_minimal)",
        arrays.iter().copied(),
        |api, string| unsafe {
            let ptr = api.rarray_const_ptr(black_box(string));
            let len = api.rarray_len(black_box(string));
            let slice = std::slice::from_raw_parts(ptr as _, len as _);

            slice.iter().map(|v| *v as usize).sum::<usize>()
        },
    );

    c.bench_abi_function(
        "rarray to slice (iter_realistic)",
        arrays.iter().copied(),
        |api, string| unsafe {
            let ptr = api.rarray_const_ptr(black_box(string));
            let len = api.rarray_len(black_box(string));
            let slice = std::slice::from_raw_parts(ptr as _, len as _);
            let output = rb_ary_new_capa(slice.len() as _);
            let upcase = rb_intern2("size".as_ptr() as *mut _, 4);

            for v in slice {
                rb_ary_push(output, rb_funcall(*v, upcase, 0));
            }

            output
        },
    );
}
