use super::StableAbiDefinition;
use crate::VALUE;
use std::os::raw::{c_char, c_long};

#[allow(dead_code)]
extern "C" {
    #[link_name = "impl_rstring_len"]
    fn impl_rstring_len(str: VALUE) -> c_long;

    #[link_name = "impl_rstring_ptr"]
    fn impl_rstring_ptr(str: VALUE) -> *const c_char;

    #[link_name = "impl_rarray_len"]
    fn impl_rarray_len(ary: VALUE) -> c_long;

    #[link_name = "impl_rarray_const_ptr"]
    fn impl_rarray_const_ptr(ary: VALUE) -> *const VALUE;

    #[link_name = "impl_special_const_p"]
    fn impl_special_const_p(value: VALUE) -> bool;
}

pub struct Definition;

impl StableAbiDefinition for Definition {
    #[inline]
    unsafe fn rstring_len(obj: crate::VALUE) -> std::ffi::c_long {
        impl_rstring_len(obj)
    }

    #[inline]
    unsafe fn rstring_ptr(obj: crate::VALUE) -> *const std::ffi::c_char {
        impl_rstring_ptr(obj)
    }

    #[inline]
    unsafe fn rarray_len(obj: crate::VALUE) -> std::ffi::c_long {
        impl_rarray_len(obj)
    }

    #[inline]
    unsafe fn rarray_const_ptr(obj: crate::VALUE) -> *const crate::VALUE {
        impl_rarray_const_ptr(obj)
    }

    #[inline]
    fn special_const_p(value: crate::VALUE) -> bool {
        unsafe { impl_special_const_p(value) }
    }
}
