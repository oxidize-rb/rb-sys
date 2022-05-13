#[path = "./helpers.rs"]
mod helpers;

use helpers::setup_ruby_vm;
use rb_sys::*;

#[test]
fn basic_smoke_test() {
    setup_ruby_vm();

    let str = std::ffi::CString::new("hello").unwrap();
    let ptr = str.as_ptr();

    unsafe {
        let rb_string_one = rb_utf8_str_new_cstr(ptr);
        let mut rb_string_two = rb_str_cat(rb_string_one, " world".as_ptr() as *const i8, 6);
        let c_string = rb_string_value_cstr(&mut rb_string_two);
        let result_str = std::ffi::CStr::from_ptr(c_string)
            .to_string_lossy()
            .into_owned();

        assert_eq!(result_str, "hello world");
    }
}
