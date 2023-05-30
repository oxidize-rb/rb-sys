use rb_sys::VALUE;
use rb_sys_test_helpers::{rstring_to_string, ruby_test, trigger_full_gc};

#[ruby_test]
fn test_rb_gc_guarded_ptr_works() {
    use rb_sys::{rb_gc_guard, rb_str_cat_cstr, rb_str_new_cstr, RSTRING_PTR};

    let mut vec_of_values: Vec<VALUE> = Default::default();

    unsafe {
        let s = rb_str_new_cstr(" world\0".as_ptr() as _);
        let sptr = RSTRING_PTR(s);
        let t = rb_str_new_cstr("hello,\0".as_ptr() as _); // Possible GC invocation
        trigger_full_gc!();
        let u = rb_str_cat_cstr(t, sptr);
        rb_gc_guard!(s); // ensure `s` (and thus `sptr`) do not get GC-ed
        vec_of_values.push(u);

        let s = rb_str_new_cstr(" world\0".as_ptr() as _);
        let sptr = RSTRING_PTR(s);
        let t = rb_str_new_cstr("hello,\0".as_ptr() as _); // Possible GC invocation
        trigger_full_gc!();
        let u = rb_str_cat_cstr(t, sptr);
        rb_gc_guard!(s); // ensure `s` (and thus `sptr`) do not get GC-ed
        vec_of_values.push(u);

        let s = rb_str_new_cstr(" world\0".as_ptr() as _);
        let sptr = RSTRING_PTR(s);
        let t = rb_str_new_cstr("hello,\0".as_ptr() as _); // Possible GC invocation
        trigger_full_gc!();
        let u = rb_str_cat_cstr(t, sptr);
        rb_gc_guard!(s); // ensure `s` (and thus `sptr`) do not get GC-ed
        vec_of_values.push(u);

        let ptr = vec_of_values.as_mut_ptr();
        let len = &vec_of_values.len();

        let rarray = rb_sys::rb_ary_new_from_values(*len as _, ptr);
        let inspected = rstring_to_string!(rb_sys::rb_inspect(rarray));

        assert_eq!(
            inspected,
            "[\"hello, world\", \"hello, world\", \"hello, world\"]"
        );
    }
}
