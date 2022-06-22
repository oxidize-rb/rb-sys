use rb_sys::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_long;

// NOTICE: This is a low level library. If you are looking to write a gem in
// Rust, you should probably use https://github.com/matsadler/magnus instead.

unsafe extern "C" fn pub_reverse(_klass: RubyValue, mut input: RubyValue) -> RubyValue {
    let ruby_string = CStr::from_ptr(rb_string_value_cstr(&mut input))
        .to_str()
        .unwrap();
    let reversed = ruby_string.to_string().chars().rev().collect::<String>();
    let reversed_cstring = CString::new(reversed).unwrap();
    let size = ruby_string.len() as c_long;

    rb_utf8_str_new(reversed_cstring.as_ptr(), size)
}

#[allow(non_snake_case)]
#[no_mangle]
extern "C" fn Init_rust_reverse() {
    let name = CString::new("RustReverse").unwrap();
    let function_name = CString::new("reverse").unwrap();

    unsafe {
        let klass = rb_define_module(name.as_ptr());
        let callback = std::mem::transmute::<
            unsafe extern "C" fn(RubyValue, RubyValue) -> RubyValue,
            unsafe extern "C" fn() -> RubyValue,
        >(pub_reverse);
        rb_define_module_function(klass, function_name.as_ptr(), Some(callback), 1)
    }
}
