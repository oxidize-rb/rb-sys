use rb_sys::{rb_gc_guard, rb_str_cat_cstr, rb_str_new_cstr, RSTRING_PTR};
use rb_sys_test_helpers::{rstring_to_string, ruby_test};

#[ruby_test(gc_stress)]
fn test_rb_gc_guarded_ptr_basic() {
    let mut string = unsafe {
        let s = rb_str_new_cstr(" world\0".as_ptr() as _);
        let sptr = RSTRING_PTR(s);
        let t = rb_str_new_cstr("hello,\0".as_ptr() as _);
        rb_gc_guard!(s);
        rb_str_cat_cstr(t, sptr)
    };

    let string = unsafe { rstring_to_string!(string) };
    assert_eq!("hello, world", string);
}
