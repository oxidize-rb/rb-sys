use rb_sys::VALUE;
use rb_sys::{rb_gc_guard, rb_str_cat_cstr, rb_str_new_cstr, RSTRING_PTR};
use rb_sys_test_helpers::{rstring_to_string, ruby_test};

#[ruby_test(gc_stress)]
fn test_rb_gc_guarded_ptr_basic() {
    let string = unsafe {
        let s = rb_str_new_cstr(" world\0".as_ptr() as _);
        let sptr = RSTRING_PTR(s);
        let t = rb_str_new_cstr("hello,\0".as_ptr() as _);
        rb_gc_guard!(s);
        let mut string = rb_str_cat_cstr(t, sptr);
        unsafe { rstring_to_string!(string) }
    };

    assert_eq!("hello, world", string);
}

#[ruby_test(gc_stress)]
fn test_rb_gc_guarded_ptr_vec() {
    for i in 0..100 {
        unsafe {
            let mut vec_of_values: Vec<VALUE> = Default::default();

            let s1 = rb_str_new_cstr(format!("hello world{i}\0").as_ptr() as _);
            vec_of_values.push(s1);

            let s2 = rb_str_new_cstr(format!("hello world{i}\0").as_ptr() as _);
            vec_of_values.push(s2);

            let s3 = rb_str_new_cstr(format!("hello world{i}\0").as_ptr() as _);
            vec_of_values.push(s3);

            let ptr = &vec_of_values.as_ptr();
            let len = &vec_of_values.len();

            let rarray = rb_sys::rb_ary_new_from_values(*len as _, *ptr);
            let mut inspected = rb_sys::rb_inspect(rarray);
            let result = rstring_to_string!(inspected);

            rb_gc_guard!(s1);
            rb_gc_guard!(s2);
            rb_gc_guard!(s3);
            rb_gc_guard!(rarray);
            rb_gc_guard!(inspected);

            assert_eq!(
                result,
                format!("[\"hello world{i}\", \"hello world{i}\", \"hello world{i}\"]")
            );
        }
    }
}
