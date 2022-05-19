use rb_sys::*;

#[cfg(not(windows_broken_vm_init_3_1))]
#[test]
fn basic_smoke_test() {
    let cstr = std::ffi::CString::new("hello").unwrap();
    let str = cstr.into_raw();

    let cworld = std::ffi::CString::new(" world").unwrap();
    let world = cworld.into_raw();

    unsafe {
        let rb_string_one = rb_utf8_str_new_cstr(str);
        let mut rb_string_two = rb_str_cat(rb_string_one, world, 6);
        let c_string = rb_string_value_cstr(&mut rb_string_two);

        let result_str = std::ffi::CStr::from_ptr(c_string)
            .to_string_lossy()
            .into_owned();

        assert_eq!(result_str, "hello world");
    }
}
