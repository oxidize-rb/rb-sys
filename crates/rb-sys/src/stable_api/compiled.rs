use super::StableApiDefinition;
use crate::{ruby_value_type, timeval, VALUE};
use std::{
    ffi::{c_int, CStr},
    os::raw::{c_char, c_long},
    ptr::NonNull,
    time::Duration,
};

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

    #[link_name = "impl_rbasic_class"]
    fn impl_rbasic_class(obj: VALUE) -> VALUE;

    #[link_name = "impl_frozen_p"]
    fn impl_frozen_p(obj: VALUE) -> bool;

    #[link_name = "impl_special_const_p"]
    fn impl_special_const_p(value: VALUE) -> bool;

    #[link_name = "impl_bignum_positive_p"]
    fn impl_bignum_positive_p(obj: VALUE) -> bool;

    #[link_name = "impl_bignum_negative_p"]
    fn impl_bignum_negative_p(obj: VALUE) -> bool;

    #[link_name = "impl_builtin_type"]
    fn impl_builtin_type(obj: VALUE) -> ruby_value_type;

    #[link_name = "impl_nil_p"]
    fn impl_nil_p(obj: VALUE) -> bool;

    #[link_name = "impl_fixnum_p"]
    fn impl_fixnum_p(obj: VALUE) -> bool;

    #[link_name = "impl_static_sym_p"]
    fn impl_static_sym_p(obj: VALUE) -> bool;

    #[link_name = "impl_flonum_p"]
    fn impl_flonum_p(obj: VALUE) -> bool;

    #[link_name = "impl_immediate_p"]
    fn impl_immediate_p(obj: VALUE) -> bool;

    #[link_name = "impl_rb_test"]
    fn impl_rb_test(obj: VALUE) -> bool;

    #[link_name = "impl_type_p"]
    fn impl_type_p(obj: VALUE, ty: ruby_value_type) -> bool;

    #[link_name = "impl_dynamic_sym_p"]
    fn impl_dynamic_sym_p(obj: VALUE) -> bool;

    #[link_name = "impl_symbol_p"]
    fn impl_symbol_p(obj: VALUE) -> bool;

    #[link_name = "impl_float_type_p"]
    fn impl_float_type_p(obj: VALUE) -> bool;

    #[link_name = "impl_rb_type"]
    fn impl_rb_type(obj: VALUE) -> ruby_value_type;

    #[link_name = "impl_integer_type_p"]
    fn impl_integer_type_p(obj: VALUE) -> bool;

    #[link_name = "impl_rstring_interned_p"]
    fn impl_rstring_interned_p(obj: VALUE) -> bool;

    #[link_name = "impl_thread_sleep"]
    fn impl_thread_sleep(interval: timeval);

    #[link_name = "impl_rstruct_define"]
    fn impl_rstruct_define(name: &CStr, members: &[*const c_char]) -> VALUE;

    #[link_name = "impl_rstruct_get"]
    fn impl_rstruct_get(st: VALUE, idx: c_int) -> VALUE;

    #[link_name = "impl_rstruct_set"]
    fn impl_rstruct_set(st: VALUE, idx: c_int, value: VALUE);

    #[link_name = "impl_rstruct_len"]
    fn impl_rstruct_len(obj: VALUE) -> c_long;
}

pub struct Definition;

impl StableApiDefinition for Definition {
    const VERSION_MAJOR: u32 = 0;
    const VERSION_MINOR: u32 = 0;

    #[inline]
    unsafe fn rstring_len(&self, obj: VALUE) -> std::os::raw::c_long {
        impl_rstring_len(obj)
    }

    #[inline]
    unsafe fn rstring_ptr(&self, obj: VALUE) -> *const std::os::raw::c_char {
        impl_rstring_ptr(obj)
    }

    #[inline]
    unsafe fn rarray_len(&self, obj: VALUE) -> std::os::raw::c_long {
        impl_rarray_len(obj)
    }

    #[inline]
    unsafe fn rarray_const_ptr(&self, obj: VALUE) -> *const VALUE {
        impl_rarray_const_ptr(obj)
    }

    #[inline]
    unsafe fn rbasic_class(&self, obj: VALUE) -> Option<NonNull<VALUE>> {
        NonNull::<VALUE>::new(impl_rbasic_class(obj) as _)
    }

    #[inline]
    unsafe fn frozen_p(&self, obj: VALUE) -> bool {
        impl_frozen_p(obj)
    }

    #[inline]
    fn special_const_p(&self, value: VALUE) -> bool {
        unsafe { impl_special_const_p(value) }
    }

    #[inline]
    unsafe fn bignum_positive_p(&self, obj: VALUE) -> bool {
        impl_bignum_positive_p(obj)
    }

    #[inline]
    unsafe fn bignum_negative_p(&self, obj: VALUE) -> bool {
        impl_bignum_negative_p(obj)
    }

    #[inline]
    unsafe fn builtin_type(&self, obj: VALUE) -> ruby_value_type {
        impl_builtin_type(obj)
    }

    #[inline]
    fn nil_p(&self, obj: VALUE) -> bool {
        unsafe { impl_nil_p(obj) }
    }

    #[inline]
    fn fixnum_p(&self, obj: VALUE) -> bool {
        unsafe { impl_fixnum_p(obj) }
    }

    #[inline]
    fn static_sym_p(&self, obj: VALUE) -> bool {
        unsafe { impl_static_sym_p(obj) }
    }

    #[inline]
    fn flonum_p(&self, obj: VALUE) -> bool {
        unsafe { impl_flonum_p(obj) }
    }

    #[inline]
    fn immediate_p(&self, obj: VALUE) -> bool {
        unsafe { impl_immediate_p(obj) }
    }

    #[inline]
    fn rb_test(&self, obj: VALUE) -> bool {
        unsafe { impl_rb_test(obj) }
    }

    #[inline]
    unsafe fn type_p(&self, obj: VALUE, ty: ruby_value_type) -> bool {
        impl_type_p(obj, ty)
    }

    #[inline]
    unsafe fn dynamic_sym_p(&self, obj: VALUE) -> bool {
        impl_dynamic_sym_p(obj)
    }

    #[inline]
    unsafe fn symbol_p(&self, obj: VALUE) -> bool {
        impl_symbol_p(obj)
    }

    #[inline]
    unsafe fn float_type_p(&self, obj: VALUE) -> bool {
        impl_float_type_p(obj)
    }

    #[inline]
    unsafe fn rb_type(&self, obj: VALUE) -> crate::ruby_value_type {
        impl_rb_type(obj)
    }

    #[inline]
    unsafe fn integer_type_p(&self, obj: VALUE) -> bool {
        impl_integer_type_p(obj)
    }

    #[inline]
    unsafe fn rstring_interned_p(&self, obj: VALUE) -> bool {
        impl_rstring_interned_p(obj)
    }

    #[inline]
    fn thread_sleep(&self, duration: Duration) {
        let seconds = duration.as_secs() as _;
        let microseconds = duration.subsec_micros() as _;

        let time = crate::timeval {
            tv_sec: seconds,
            tv_usec: microseconds,
        };

        unsafe { impl_thread_sleep(time) }
    }

    #[inline]
    fn rstruct_define(&self, name: &CStr, members: &[&CStr]) -> VALUE {
        let mut members: Vec<*const c_char> = members
            .iter()
            .map(|m| m.as_ptr() as *const c_char)
            .collect();
        members.push(std::ptr::null());
        unsafe { impl_rstruct_define(name, members.as_ref()) }
    }

    #[inline]
    unsafe fn rstruct_get(&self, st: VALUE, idx: c_int) -> VALUE {
        impl_rstruct_get(st, idx)
    }

    #[inline]
    unsafe fn rstruct_set(&self, st: VALUE, idx: c_int, value: VALUE) {
        impl_rstruct_set(st, idx, value)
    }

    #[inline]
    unsafe fn rstruct_len(&self, st: VALUE) -> c_long {
        impl_rstruct_len(st)
    }
}
