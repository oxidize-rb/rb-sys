use rb_sys::value_type::*;
use rb_sys::*;
use rb_sys_test_helpers::{rstring, ruby_test};

#[ruby_test]
fn test_builtin_type_p() {
    let val = rstring!("foo");

    assert_eq!(unsafe { RB_BUILTIN_TYPE(val) }, RUBY_T_STRING);
}

#[ruby_test]
fn test_rb_integer_type_p() {
    let int = unsafe { rb_num2fix(1) };
    let big = unsafe { rb_int2big(9999999) };

    assert!(unsafe { RB_INTEGER_TYPE_P(int) });
    assert!(unsafe { RB_INTEGER_TYPE_P(big) });
    assert!(!unsafe { RB_INTEGER_TYPE_P(Qnil as VALUE) });
}

#[ruby_test]
fn test_rb_dynamic_sym_p() {
    unsafe {
        let id = rb_intern_str(rstring!("foostatic"));
        let static_sym = rb_id2sym(id);
        let sym = rb_to_symbol(rstring!("foobardynamic"));

        assert!(!RB_DYNAMIC_SYM_P(static_sym));
        assert!(RB_DYNAMIC_SYM_P(sym));
    }
}

#[ruby_test]
fn test_rb_symbol_p() {
    unsafe {
        let id = rb_intern_str(rstring!("foo"));
        let static_sym = rb_id2sym(id);
        let sym = rb_to_symbol(rstring!("foobar"));

        assert!(RB_SYMBOL_P(static_sym));
        assert!(RB_SYMBOL_P(sym));
    }
}

#[ruby_test]
fn test_rb_type_p() {
    unsafe {
        assert!(RB_TYPE_P(rstring!("foo"), RUBY_T_STRING));
        assert!(RB_TYPE_P(rb_to_symbol(rstring!("foo")), RUBY_T_SYMBOL));
        assert!(RB_TYPE_P(Qnil as _, RUBY_T_NIL));
        assert!(RB_TYPE_P(Qtrue as _, RUBY_T_TRUE));
        assert!(RB_TYPE_P(Qfalse as _, RUBY_T_FALSE));
    }
}
