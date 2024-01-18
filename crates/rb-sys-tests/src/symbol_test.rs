use std::slice;

use rb_sys::{
    rb_funcall, rb_id2sym, rb_intern, rb_utf8_str_new, RSTRING_LEN, RSTRING_PTR, STATIC_SYM_P,
};
use rb_sys_test_helpers::ruby_test;

#[ruby_test]
fn test_creates_a_usable_id() {
    let method_name = unsafe { rb_intern!("reverse") };

    let mystring = unsafe { rb_utf8_str_new("jerrbear".as_ptr() as *mut _, 8) };
    let ret = unsafe { rb_funcall(mystring, method_name, 0) };
    let ptr = unsafe { RSTRING_PTR(ret) as *const u8 };
    let len = unsafe { RSTRING_LEN(ret) } as _;
    let result = unsafe { slice::from_raw_parts(ptr, len) };

    assert_eq!(result, b"raebrrej");
}

#[ruby_test]
fn test_has_repeatable_results() {
    let method_name1 = unsafe { rb_intern!("reverse") };
    let method_name2 = unsafe { rb_intern!("reverse") };

    assert_ne!(method_name1, 0);
    assert_ne!(method_name2, 0);
    assert_eq!(method_name1, method_name2);
}

#[ruby_test]
fn test_non_usascii() {
    let method_name1 = unsafe { rb_intern!("🙈") };
    let method_name2 = unsafe { rb_intern!("🙈") };

    assert_ne!(method_name1, 0);
    assert_ne!(method_name2, 0);
    assert_eq!(method_name1, method_name2);

    let sym1 = unsafe { rb_id2sym(method_name1) };
    let sym2 = unsafe { rb_id2sym(method_name2) };

    assert!(STATIC_SYM_P(sym1));
    assert!(STATIC_SYM_P(sym2));
}
