use rb_sys::special_consts::*;
use rb_sys::*;
use rb_sys_env::test_helpers::with_ruby_vm;

#[test]
fn test_fixnum_p() {
    with_ruby_vm(|| unsafe {
        let int = rb_num2fix(1);
        let big = rb_int2big(9999999);

        assert!(FIXNUM_P(int));
        assert!(!FIXNUM_P(big));
    })
}

#[test]
fn test_static_sym_p() {
    with_ruby_vm(|| unsafe {
        let id = rb_intern_str(rstring!("foo"));
        let sym = rb_id2sym(id);

        assert!(STATIC_SYM_P(sym));
        assert!(!STATIC_SYM_P(Qnil));
    })
}

#[test]
fn test_flonum_p() {
    with_ruby_vm(|| unsafe {
        let flonum = rb_float_new(0.0);

        #[cfg(ruby_use_flonum)]
        assert!(FLONUM_P(flonum));
        #[cfg(not(ruby_use_flonum))]
        assert!(!FLONUM_P(flonum));

        assert!(!FLONUM_P(Qnil));
    });
}
