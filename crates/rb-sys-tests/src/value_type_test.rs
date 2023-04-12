use rb_sys::value_type::*;
use rb_sys::*;
use rb_sys_env::test_helpers::with_ruby_vm;

#[test]
fn test_builtin_type_p() {
    with_ruby_vm(|| unsafe {
        let val = rstring!("foo");

        assert_eq!(RB_BUILTIN_TYPE(val), RUBY_T_STRING);
    })
}

#[test]
fn test_rb_integer_type_p() {
    with_ruby_vm(|| unsafe {
        let int = rb_num2fix(1);
        let big = rb_int2big(9999999);

        assert!(RB_INTEGER_TYPE_P(int));
        assert!(RB_INTEGER_TYPE_P(big));
        assert!(!RB_INTEGER_TYPE_P(Qnil as VALUE));
    })
}

#[test]
fn test_rb_dynamic_sym_p() {
    with_ruby_vm(|| unsafe {
        let id = rb_intern_str(rstring!("foo"));
        let static_sym = rb_id2sym(id);
        let sym = rb_to_symbol(rstring!("foobar"));

        assert!(!RB_DYNAMIC_SYM_P(static_sym));
        assert!(RB_DYNAMIC_SYM_P(sym));
    });
}

#[test]
fn test_rb_symbol_p() {
    with_ruby_vm(|| unsafe {
        let id = rb_intern_str(rstring!("foo"));
        let static_sym = rb_id2sym(id);
        let sym = rb_to_symbol(rstring!("foobar"));

        assert!(RB_SYMBOL_P(static_sym));
        assert!(RB_SYMBOL_P(sym));
    })
}

#[test]
fn test_rb_type_p() {
    with_ruby_vm(|| unsafe {
        assert_eq!(RB_TYPE_P(rstring!("foo")), RUBY_T_STRING);
        assert_eq!(RB_TYPE_P(rb_to_symbol(rstring!("foo"))), RUBY_T_SYMBOL);
        assert_eq!(RB_TYPE_P(Qnil), RUBY_T_NIL);
        assert_eq!(RB_TYPE_P(Qtrue), RUBY_T_TRUE);
        assert_eq!(RB_TYPE_P(Qfalse), RUBY_T_FALSE);
    });
}
