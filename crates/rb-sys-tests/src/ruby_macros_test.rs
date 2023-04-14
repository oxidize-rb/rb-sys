use rb_sys::macros::*;
use rb_sys::*;
use rb_sys_test_helpers::{rstring, ruby_test};
use std::{slice, str};

#[ruby_test]
fn test_rstring_len() {
    let rstr = unsafe { rstring!("foo") };

    assert_eq!(unsafe { RSTRING_LEN(rstr) }, 3);
}

#[ruby_test]
fn test_rarray_len() {
    let rstr: VALUE = unsafe { rstring!("foo") };
    let rarray = unsafe { rb_ary_new() };
    unsafe { rb_ary_push(rarray, rstr) };

    assert_eq!(unsafe { RARRAY_LEN(rarray) }, 1);
}

#[ruby_test]
fn test_rstring_ptr() {
    let rstr = unsafe { rstring!("foo") };

    let rust_str = unsafe {
        let ptr = RSTRING_PTR(rstr);
        let len = RSTRING_LEN(rstr);

        str::from_utf8(slice::from_raw_parts(ptr as _, len as _))
    };

    assert_eq!(rust_str.unwrap(), "foo");
}

#[ruby_test]
fn test_rarray_ptr() {
    let ary = unsafe { rb_ary_new() };
    let foo = unsafe { rstring!("foo") };

    unsafe { rb_ary_push(ary, Qtrue as _) };
    unsafe { rb_ary_push(ary, Qnil as _) };
    unsafe { rb_ary_push(ary, Qfalse as _) };
    unsafe { rb_ary_push(ary, foo) };

    let slice = unsafe {
        let ptr = RARRAY_PTR(ary);
        let len = RARRAY_LEN(ary);

        slice::from_raw_parts(ptr as _, len as _)
    };

    assert_eq!(slice, [Qtrue as _, Qnil as _, Qfalse as _, foo]);
}
