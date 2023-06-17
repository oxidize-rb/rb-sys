use super::StableAbiDefinition;
use crate::{
    internal::{RArray, RString},
    value_type, VALUE,
};
use std::ffi::{c_char, c_long};

pub struct Definition;

impl StableAbiDefinition for Definition {
    #[inline]
    unsafe fn rstring_len(obj: VALUE) -> c_long {
        unsafe {
            assert!(value_type::RB_TYPE_P(obj) == value_type::RUBY_T_STRING);

            let rstring: &RString = &*(obj as *const RString);
            let flags = rstring.basic.flags;
            let is_heap = (flags & crate::ruby_rstring_flags::RSTRING_NOEMBED as VALUE) != 0;

            if !is_heap {
                use crate::ruby_rstring_consts::RSTRING_EMBED_LEN_SHIFT;

                let mut f = rstring.basic.flags;
                f &= crate::ruby_rstring_flags::RSTRING_EMBED_LEN_MASK as VALUE;
                f >>= RSTRING_EMBED_LEN_SHIFT as VALUE;
                f as c_long
            } else {
                rstring.as_.heap.len
            }
        }
    }

    #[inline]
    unsafe fn rstring_ptr(obj: VALUE) -> *const c_char {
        unsafe {
            assert!(value_type::RB_TYPE_P(obj) == value_type::RUBY_T_STRING);

            let rstring: &RString = &*(obj as *const RString);
            let flags = rstring.basic.flags;
            let is_heap = (flags & crate::ruby_rstring_flags::RSTRING_NOEMBED as VALUE) != 0;

            if !is_heap {
                rstring.as_.ary.as_ptr() as *const _
            } else {
                rstring.as_.heap.ptr
            }
        }
    }

    #[inline]
    unsafe fn rarray_len(obj: VALUE) -> c_long {
        unsafe {
            assert!(value_type::RB_TYPE_P(obj) == value_type::RUBY_T_ARRAY);

            let rarray: &RArray = &*(obj as *const RArray);
            let flags = rarray.basic.flags;
            let is_embedded = (flags & crate::ruby_rarray_flags::RARRAY_EMBED_FLAG as VALUE) != 0;

            if is_embedded {
                let mut f = rarray.basic.flags;
                f &= crate::ruby_rarray_flags::RARRAY_EMBED_LEN_MASK as VALUE;
                f >>= crate::ruby_rarray_consts::RARRAY_EMBED_LEN_SHIFT as VALUE;
                f as c_long
            } else {
                rarray.as_.heap.len
            }
        }
    }

    #[inline]
    unsafe fn rarray_const_ptr(obj: VALUE) -> *const VALUE {
        unsafe {
            assert!(value_type::RB_TYPE_P(obj) == value_type::RUBY_T_ARRAY);
            let rarray: &RArray = &*(obj as *const RArray);

            let flags = rarray.basic.flags;
            let is_embedded = (flags & crate::ruby_rarray_flags::RARRAY_EMBED_FLAG as VALUE) != 0;

            if is_embedded {
                rarray.as_.ary.as_ptr()
            } else {
                rarray.as_.heap.ptr
            }
        }
    }
}