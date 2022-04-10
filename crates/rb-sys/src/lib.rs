#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(deref_nullptr)]
#![allow(unaligned_references)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub type RubyValue = VALUE;

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
#[cfg(link_ruby)]
mod tests {
    use super::*;

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
}
