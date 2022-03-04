extern crate rb_sys;

use rb_sys::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_long;

#[rb_extern]
fn pub_reverse(_klass: RubyValue, mut input: RubyValue) -> RubyValue {
    let ruby_string = unsafe {
        CStr::from_ptr(rb_string_value_cstr(&mut input))
            .to_str()
            .unwrap()
    };
    let reversed = ruby_string.to_string().chars().rev().collect::<String>();
    let reversed_cstring = CString::new(reversed).unwrap();
    let size = ruby_string.len() as c_long;

    unsafe { rb_utf8_str_new(reversed_cstring.as_ptr(), size) }
}

#[rb_extension_init]
fn Init_rust_ruby_example() {
    let name = CString::new("RustRubyExample").unwrap();
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
