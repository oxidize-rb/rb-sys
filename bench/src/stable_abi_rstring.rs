use criterion::Criterion;
use rb_sys_test_helpers::eval;
use std::hint::black_box;

use crate::StableApiBenchExt;

pub fn run(c: &mut Criterion) {
    let strings = [
        ("8", eval!("'aaaaaaaa'")),
        ("64", eval!("'a' * 64")),
        ("256", eval!("'a' * 256")),
    ];

    c.bench_abi_function(
        "rstring to str (unchecked)",
        strings.iter().copied(),
        |api, string| unsafe {
            let ptr = api.rstring_ptr(black_box(string));
            let len = api.rstring_len(black_box(string));
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr as _, len as _))
        },
    );

    c.bench_abi_function(
        "rstring to str (checked)",
        strings.iter().copied(),
        |api, string| unsafe {
            let ptr = api.rstring_ptr(black_box(string));
            let len = api.rstring_len(black_box(string));
            std::str::from_utf8(std::slice::from_raw_parts(ptr as _, len as _)).unwrap()
        },
    );
}
