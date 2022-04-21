#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unknown_lints)]
#![allow(deref_nullptr)]
#![warn(unknown_lints)]
#![allow(unaligned_references)]

use std::fmt::Debug;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// An object handle similar to VALUE in the C code. Our methods assume
/// that this is a handle. Sometimes the C code briefly uses VALUE as
/// an unsigned integer type and don't necessarily store valid handles but
/// thankfully those cases are rare and don't cross the FFI boundary.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)] // same size and alignment as simply `usize`
pub struct VALUE(pub usize);

pub type RubyValue = VALUE;
pub type RubyValueType = ruby_value_type;

#[cfg(ruby_dln_check_abi)]
#[macro_export]
macro_rules! ruby_extension {
    () => {
        #[no_mangle]
        #[allow(unused)]
        pub extern "C" fn ruby_abi_version() -> std::os::raw::c_ulonglong {
            use $crate::RUBY_ABI_VERSION;

            RUBY_ABI_VERSION.into()
        }
    };
}

#[cfg(not(ruby_dln_check_abi))]
#[macro_export]
macro_rules! ruby_extension {
    () => {};
}

#[cfg(test)]
mod tests {
    use super::*;

    ruby_extension!();

    #[cfg(unix)]
    #[cfg(ruby_major = "3")]
    #[cfg(ruby_minor = "2")]
    #[test]
    fn test_ruby_abi_version() {
        assert!(ruby_abi_version() == 1)
    }

    #[test]
    fn test_ruby_value_type_debug() {
        assert_eq!(
            format!("nil debug: {:?}", RubyValueType::RUBY_T_NIL),
            "nil debug: RUBY_T_NIL"
        );
    }

    #[cfg(link_ruby)]
    #[test]
    fn basic_smoketest() {
        let str = std::ffi::CString::new("hello").unwrap();
        let ptr = str.as_ptr();

        unsafe {
            ruby_init();
            let rb_string_one = rb_utf8_str_new_cstr(ptr);
            let mut rb_string_two = rb_str_cat(rb_string_one, " world".as_ptr() as *const i8, 6);
            let c_string = rb_string_value_cstr(&mut rb_string_two);
            let result_str = std::ffi::CStr::from_ptr(c_string)
                .to_string_lossy()
                .into_owned();

            assert_eq!(result_str, "hello world");
        }
    }
}
