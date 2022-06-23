use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rb_sys::macros::{RB_TYPE_P, RSTRING_LEN};
use rb_sys::ruby_rstring_consts::RSTRING_EMBED_LEN_SHIFT;
use rb_sys::{ruby_rstring_flags, ruby_value_type, Qnil, RBasic, RString, VALUE};

fn vm_init() {
    let var_in_stack_frame = unsafe { std::mem::zeroed() };
    unsafe { rb_sys::ruby_init_stack(var_in_stack_frame) };
    unsafe { rb_sys::ruby_init() };
}

// To test inlining vs. using RSTRING_LEN macro
fn rusty_rb_str_len(str: VALUE) -> usize {
    unsafe {
        let r_basic = std::ptr::NonNull::new_unchecked(str as *mut RBasic);
        let mut f = r_basic.as_ref().flags;
        if (f & ruby_rstring_flags::RSTRING_NOEMBED as VALUE) != 0 {
            let internal: std::ptr::NonNull<RString> =
                std::ptr::NonNull::new_unchecked(str as *mut _);
            let h = internal.as_ref().as_.heap;
            h.len as usize
        } else {
            f &= ruby_rstring_flags::RSTRING_EMBED_LEN_MASK as VALUE;
            f >>= RSTRING_EMBED_LEN_SHIFT as VALUE;
            f as usize
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    vm_init();

    c.bench_function("ruby_macros::RSTRING_LEN", |b| {
        let ptr = std::ffi::CString::new(
            "foo bar baz foo bar baz foo bar baz foo bar baz foo bar baz foo bar baz",
        )
        .unwrap()
        .into_raw();
        let ptr_small = std::ffi::CString::new("").unwrap().into_raw();
        let rstring = unsafe { rb_sys::rb_str_new_cstr(ptr) };
        let rstring_small = unsafe { rb_sys::rb_str_new_cstr(ptr_small) };

        b.iter(|| {
            unsafe { RSTRING_LEN(black_box(rstring)) };
            unsafe { RSTRING_LEN(black_box(rstring_small)) };
        });
    });

    c.bench_function("rusty_rb_str_len", |b| {
        let ptr = std::ffi::CString::new(
            "foo bar baz foo bar baz foo bar baz foo bar baz foo bar baz foo bar baz",
        )
        .unwrap()
        .into_raw();
        let ptr_small = std::ffi::CString::new("").unwrap().into_raw();
        let rstring = unsafe { rb_sys::rb_str_new_cstr(ptr) };
        let rstring_small = unsafe { rb_sys::rb_str_new_cstr(ptr_small) };

        b.iter(|| {
            rusty_rb_str_len(black_box(rstring));
            rusty_rb_str_len(black_box(rstring_small));
        });
    });

    c.bench_function("ruby_macros::RB_TYPE_P", |b| {
        let ptr = std::ffi::CString::new("foo").unwrap().into_raw();
        let rstring = unsafe { rb_sys::rb_str_new_cstr(ptr) };
        let ary = unsafe { rb_sys::rb_ary_new() };

        b.iter(|| unsafe {
            RB_TYPE_P(
                black_box(rstring),
                black_box(ruby_value_type::RUBY_T_STRING),
            );
            RB_TYPE_P(
                black_box(Qnil as _),
                black_box(ruby_value_type::RUBY_T_STRING),
            );
            RB_TYPE_P(black_box(rstring), black_box(ruby_value_type::RUBY_T_ARRAY));
            RB_TYPE_P(black_box(ary), black_box(ruby_value_type::RUBY_T_ARRAY));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
