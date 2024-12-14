#![allow(clippy::manual_c_str_literals)]

use rb_sys::tracking_allocator::ManuallyTracked;
use rb_sys::*;
use std::os::raw::c_long;

// NOTICE: This is a low level library. If you are looking to write a gem in
// Rust, you should probably use https://github.com/matsadler/magnus instead.

unsafe extern "C" fn pub_reverse(_klass: VALUE, input: VALUE) -> VALUE {
    if rb_sys::NIL_P(input) {
        // Just here to test out linking globals on msvc
        rb_raise(rb_eTypeError, "cannot be nil\0".as_ptr() as *const i8);
    }

    let ptr = RSTRING_PTR(input);
    let len = RSTRING_LEN(input);
    let slice = std::slice::from_raw_parts(ptr as *const u8, len as usize);
    let ruby_string = std::str::from_utf8(slice).unwrap();
    let reversed = ruby_string.chars().rev().collect::<String>();

    // Just here to test out the tracking allocator
    let manually_tracked = ManuallyTracked::wrap("foo", 1024);
    assert_eq!(&"foo", manually_tracked.get());

    rb_utf8_str_new(reversed.as_ptr() as _, reversed.len() as c_long)
}

#[allow(non_snake_case)]
#[no_mangle]
extern "C" fn Init_rust_reverse() {
    unsafe {
        let klass = rb_define_module("RustReverse\0".as_ptr() as *const i8);
        let callback = std::mem::transmute::<
            unsafe extern "C" fn(VALUE, VALUE) -> VALUE,
            unsafe extern "C" fn() -> VALUE,
        >(pub_reverse);
        rb_define_module_function(klass, "reverse\0".as_ptr() as _, Some(callback), 1)
    }
}
