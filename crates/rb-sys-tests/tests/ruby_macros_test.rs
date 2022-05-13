#[path = "./helpers.rs"]
mod helpers;

use helpers::*;
use rb_sys::macros::*;
use rb_sys::*;

#[test]
fn test_nil_p() {
    assert!(unsafe { NIL_P(Qnil as u64) });
}

#[test]
fn test_rb_test() {
    assert!(!unsafe { RB_TEST(Qnil as u64) });
}

#[test]
fn test_symbol_p() {
    eprintln!("👋 About to start ruby vm");
    setup_ruby_vm();

    eprintln!("👋 About to call ID2SYM");
    let name = std::ffi::CString::new("foo").unwrap();
    let sym = unsafe { ID2SYM(rb_intern(name.into_raw())) };

    eprintln!("👋 About to call SYMBOL_P");
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
