use super::StableAbiDefinition;

use crate::ruby_rarray_flags::*;
use crate::ruby_rstring_flags::*;
use crate::{
    internal::{RArray, RString},
    value_type, VALUE,
};
use std::os::raw::{c_char, c_long};

pub struct Definition;

impl StableAbiDefinition for Definition {
    #[inline]
    unsafe fn rstring_len(obj: VALUE) -> c_long {
        assert!(value_type::RB_TYPE_P(obj) == value_type::RUBY_T_STRING);
        let rstring: &RString = &*(obj as *const RString);
        let flags = rstring.basic.flags;
        let is_heap = (flags & RSTRING_NOEMBED as VALUE) != 0;

        if !is_heap {
            let mut f = rstring.basic.flags;
            f &= RSTRING_EMBED_LEN_MASK as VALUE;
            f >>= RSTRING_EMBED_LEN_SHIFT as VALUE;
            f as c_long
        } else {
            rstring.as_.heap.len
        }
    }

    #[inline]
    unsafe fn rstring_ptr(obj: VALUE) -> *const c_char {
        assert!(value_type::RB_TYPE_P(obj) == value_type::RUBY_T_STRING);
        let rstring: &RString = &*(obj as *const RString);

        let flags = rstring.basic.flags;
        let is_heap = (flags & RSTRING_NOEMBED as VALUE) != 0;

        if !is_heap {
            rstring.as_.ary.as_ptr() as *const _
        } else {
            rstring.as_.heap.ptr
        }
    }

    #[inline]
    unsafe fn rarray_len(obj: VALUE) -> c_long {
        assert!(value_type::RB_TYPE_P(obj) == value_type::RUBY_T_ARRAY);
        let rarray: &RArray = &*(obj as *const RArray);

        let flags = rarray.basic.flags;
        let is_embedded = (flags & RARRAY_EMBED_FLAG as VALUE) != 0;

        if is_embedded {
            let mut f = rarray.basic.flags;
            f &= RARRAY_EMBED_LEN_MASK as VALUE;
            f >>= RARRAY_EMBED_LEN_SHIFT as VALUE;
            f as c_long
        } else {
            rarray.as_.heap.len
        }
    }

    #[inline]
    unsafe fn rarray_const_ptr(obj: VALUE) -> *const VALUE {
        assert!(value_type::RB_TYPE_P(obj) == value_type::RUBY_T_ARRAY);
        let rarray: &RArray = &*(obj as *const RArray);

        let flags = rarray.basic.flags;
        let is_embedded = (flags & RARRAY_EMBED_FLAG as VALUE) != 0;

        if is_embedded {
            rarray.as_.ary.as_ptr()
        } else {
            rarray.as_.heap.ptr
        }
    }

    #[inline]
    fn special_const_p(value: VALUE) -> bool {
        let is_immediate = value & (crate::special_consts::IMMEDIATE_MASK as VALUE) != 0;
        let test = (value & !(crate::Qnil as VALUE)) != 0;

        is_immediate || !test
    }

    #[inline]
    unsafe fn rb_builtin_type(obj: VALUE) -> crate::ruby_value_type {
        let rbasic = obj as *const crate::RBasic;
        let ret: u32 = ((*rbasic).flags & crate::ruby_value_type::RUBY_T_MASK as VALUE) as _;

        std::mem::transmute::<_, crate::ruby_value_type>(ret)
    }

    #[inline]
    fn rb_nil_p(obj: VALUE) -> bool {
        obj == (crate::Qnil as VALUE)
    }

    #[inline]
    fn rb_fixnum_p(obj: VALUE) -> bool {
        (obj & crate::FIXNUM_FLAG as VALUE) != 0
    }

    #[inline]
    fn rb_static_sym_p(obj: VALUE) -> bool {
        let mask = !(VALUE::MAX << crate::ruby_special_consts::RUBY_SPECIAL_SHIFT as VALUE);
        (obj & mask) == crate::ruby_special_consts::RUBY_SYMBOL_FLAG as VALUE
    }
}
