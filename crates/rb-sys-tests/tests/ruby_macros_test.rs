use rb_sys::macros::*;
use rb_sys::*;
use std::{slice, str};

macro_rules! rstring {
    ($s:expr) => {
        unsafe { rb_str_new($s.as_ptr() as _, $s.len() as _) }
    };
}

#[test]
fn test_nil_p() {
    assert!(unsafe { NIL_P(Qnil as u64) });
}

#[test]
fn test_rb_test() {
    assert!(!unsafe { RB_TEST(Qnil as u64) });
}

#[cfg(not(windows_broken_vm_init_3_1))]
#[test]
fn test_symbol_p() {
    let name = std::ffi::CString::new("foo").unwrap();
    let ptr = name.as_ptr();
    let symbol = unsafe { rb_intern(ptr) };
    let sym = unsafe { ID2SYM(symbol) };

    assert!(unsafe { SYMBOL_P(sym) });
}

#[test]
fn test_integer_type_p() {
    let int = unsafe { rb_num2fix(1) };

    assert!(unsafe { RB_INTEGER_TYPE_P(int) });
}

#[test]
fn test_rb_float_type_p() {
    let float = unsafe { rb_float_new(1.0) };

    assert!(unsafe { RB_FLOAT_TYPE_P(float) });
}

#[cfg(not(windows_broken_vm_init_3_1))]
#[test]
fn test_rstring_len() {
    let rstr = rstring!("foo");

    assert_eq!(unsafe { RSTRING_LEN(rstr) }, 3);
}

#[cfg(not(windows_broken_vm_init_3_1))]
#[test]
fn test_rarray_len() {
    let rstr: VALUE = rstring!("foo");
    let rarray = unsafe { rb_ary_new() };
    unsafe { rb_ary_push(rarray, rstr) };

    assert_eq!(unsafe { RARRAY_LEN(rarray) }, 1);
}

#[cfg(not(windows_broken_vm_init_3_1))]
#[test]
fn test_rstring_ptr() {
    let rstr = rstring!("foo");

    let rust_str = unsafe {
        let ptr = RSTRING_PTR(rstr);
        let len = RSTRING_LEN(rstr);

        str::from_utf8(slice::from_raw_parts(ptr as _, len as _))
    };

    assert_eq!(rust_str.unwrap(), "foo");
}

#[cfg(not(windows_broken_vm_init_3_1))]
#[test]
fn test_rarray_ptr() {
    let ary = unsafe { rb_ary_new() };
    let foo = rstring!("foo");

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
