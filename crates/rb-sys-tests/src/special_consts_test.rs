use rb_sys::special_consts::*;
use rb_sys::*;
use rb_sys_test_helpers::{rstring, ruby_test};

#[ruby_test]
fn test_fixnum_p() {
    let int = unsafe { rb_num2fix(1) };
    let big = unsafe { rb_int2big(9999999) };

    assert!(FIXNUM_P(int));
    assert!(!FIXNUM_P(big));
}

#[ruby_test]
fn test_static_sym_p() {
    let id = unsafe { rb_intern_str(rstring!("teststaticsymp")) };
    let sym = unsafe { rb_id2sym(id) };

    assert!(STATIC_SYM_P(sym));
    assert!(!STATIC_SYM_P(Qnil as VALUE));
}

#[ruby_test]
fn test_flonum_p() {
    let flonum = unsafe { rb_float_new(0.0) };

    #[cfg(ruby_use_flonum)]
    assert!(FLONUM_P(flonum));
    #[cfg(not(ruby_use_flonum))]
    assert!(!FLONUM_P(flonum));

    assert!(!FLONUM_P(Qnil as VALUE));
}
