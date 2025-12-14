use rb_sys::VALUE;
use rb_sys::{rb_gc_guard, rb_str_cat_cstr, rb_str_new_cstr, RSTRING_PTR};
use rb_sys_test_helpers::{rstring_to_string, ruby_test};

#[ruby_test(gc_stress)]
fn test_rb_gc_guarded_ptr_basic() {
    unsafe {
        let s = rb_str_new_cstr(" world\0".as_ptr() as _);
        let sptr = RSTRING_PTR(s);
        let t = rb_str_new_cstr("hello,\0".as_ptr() as _);
        let mut string = rb_str_cat_cstr(t, sptr);
        let result = rstring_to_string!(string);

        let _ = rb_gc_guard!(s);
        let _ = rb_gc_guard!(t);
        let _ = rb_gc_guard!(string);

        assert_eq!("hello, world", result);
    }
}

#[ruby_test(gc_stress)]
fn test_rb_gc_guarded_ptr_vec() {
    for i in 0..42 {
        unsafe {
            let mut vec_of_values: Vec<VALUE> = Default::default();

            // Keep the CStrings alive until after rb_str_new_cstr uses them
            let cstr1 = std::ffi::CString::new(format!("hello world{i}")).unwrap();
            let s1 = rb_str_new_cstr(cstr1.as_ptr());
            let s1 = rb_gc_guard!(s1);
            vec_of_values.push(s1);

            let cstr2 = std::ffi::CString::new(format!("hello world{i}")).unwrap();
            let s2 = rb_str_new_cstr(cstr2.as_ptr());
            let s2 = rb_gc_guard!(s2);
            vec_of_values.push(s2);

            let cstr3 = std::ffi::CString::new(format!("hello world{i}")).unwrap();
            let s3 = rb_str_new_cstr(cstr3.as_ptr());
            let s3 = rb_gc_guard!(s3);
            vec_of_values.push(s3);

            let ptr = &vec_of_values.as_ptr();
            let len = &vec_of_values.len();

            let rarray = rb_sys::rb_ary_new_from_values(*len as _, *ptr);
            let rarray = rb_gc_guard!(rarray);

            let inspected = rb_sys::rb_inspect(rarray);
            let mut inspected = rb_gc_guard!(inspected);
            let result = rstring_to_string!(inspected);

            assert_eq!(
                result,
                format!("[\"hello world{i}\", \"hello world{i}\", \"hello world{i}\"]")
            );
        }
    }
}
