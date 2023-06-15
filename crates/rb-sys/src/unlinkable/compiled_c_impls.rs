//! C implemenations of Ruby preprocessor macros and inlined functions, used
//! when ABI compatibility cannot be guaranteed.

use std::ffi::{c_char, c_long};

use crate::VALUE;

#[allow(dead_code)]
extern "C" {
    #[link_name = "rb_sys_compiled_c_impls_RSTRING_LEN"]
    pub(crate) fn rstring_len(str: VALUE) -> c_long;

    #[link_name = "rb_sys_compiled_c_impls_RSTRING_PTR"]
    pub(crate) fn rstring_ptr(str: VALUE) -> *const c_char;

    #[link_name = "rb_sys_compiled_c_impls_RSTRING_LEN"]
    pub(crate) fn rarray_len(ary: VALUE) -> c_long;

    #[link_name = "rb_sys_compiled_c_impls_RARRAY_CONST_PTR"]
    pub(crate) fn rarray_const_ptr(ary: VALUE) -> *const VALUE;
}
