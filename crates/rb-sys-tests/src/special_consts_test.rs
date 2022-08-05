use rb_sys::special_consts::*;
use rb_sys::*;

#[cfg(not(windows_broken_vm_init_3_1))]
#[test]
fn test_fixnum_p() {
    unsafe {
        let int = rb_num2fix(1);
        let big = rb_int2big(9999999);

        assert!(FIXNUM_P(int));
        assert!(!FIXNUM_P(big));
    }
}

#[cfg(not(windows_broken_vm_init_3_1))]
#[test]
fn test_static_sym_p() {
    unsafe {
        let id = rb_intern_str(rstring!("foo"));
        let sym = rb_id2sym(id);

        assert!(STATIC_SYM_P(sym));
        assert!(!STATIC_SYM_P(Qnil));
    }
}

#[cfg(not(windows_broken_vm_init_3_1))]
#[test]
fn test_flonum_p() {
    unsafe {
        let flonum = rb_float_new(0.0);

        assert!(FLONUM_P(flonum));
        assert!(!FLONUM_P(Qnil));
    }
}
