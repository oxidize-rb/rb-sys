use rb_sys::macros::*;
use rb_sys::*;
use std::{slice, str};

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
    let cstr = std::ffi::CString::new("foo").unwrap();
    let rstring: VALUE = unsafe { rb_str_new_cstr(cstr.as_ptr()) };

    assert_eq!(unsafe { RSTRING_LEN(rstring) }, 3);
}

#[cfg(not(windows_broken_vm_init_3_1))]
#[test]
fn test_rarray_len() {
    let cstr = std::ffi::CString::new("foo").unwrap();
    let rstring: VALUE = unsafe { rb_str_new_cstr(cstr.as_ptr()) };
    let rarray = unsafe { rb_ary_new() };
    unsafe { rb_ary_push(rarray, rstring) };

    assert_eq!(unsafe { RARRAY_LEN(rarray) }, 1);
}

#[cfg(not(windows_broken_vm_init_3_1))]
#[test]
fn test_rstring_ptr() {
    let cstr = std::ffi::CString::new("foo").unwrap();
    let rstring: VALUE = unsafe { rb_str_new_cstr(cstr.as_ptr()) };

    let rust_str = unsafe {
        let ptr = RSTRING_PTR(rstring);
        let len = RSTRING_LEN(rstring);

        str::from_utf8(slice::from_raw_parts(ptr as _, len as _))
    };

    assert_eq!(rust_str.unwrap(), "foo");
}
