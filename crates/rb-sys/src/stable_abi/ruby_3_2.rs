use super::StableAbiDefinition;
use crate::{
    internal::{RArray, RString},
    value_type, VALUE,
};
use std::ffi::{c_char, c_long};

pub struct Definition;

impl StableAbiDefinition for Definition {
    // #[cfg(ruby_gte_3_2)]
    // unsafe fn embed_len(self) -> c_long {
    //     self.as_.embed.len as _
    // }

    // #[cfg(ruby_gte_3_1)]
    // unsafe fn embed_ptr(self) -> *const c_char {
    // }

    unsafe fn rstring_len(obj: VALUE) -> c_long {
        assert!(value_type::RB_TYPE_P(obj) == value_type::RUBY_T_STRING);
        let rstring: &RString = &*(obj as *const RString);
        let flags = rstring.basic.flags;
        let is_heap = (flags & crate::ruby_rstring_flags::RSTRING_NOEMBED as VALUE) != 0;

        if !is_heap {
            rstring.as_.embed.len as _
        } else {
            rstring.as_.heap.len
        }
    }

    unsafe fn rstring_ptr(obj: VALUE) -> *const c_char {
        assert!(value_type::RB_TYPE_P(obj) == value_type::RUBY_T_STRING);
        let rstring: &RString = &*(obj as *const RString);

        let flags = rstring.basic.flags;
        let is_heap = (flags & crate::ruby_rstring_flags::RSTRING_NOEMBED as VALUE) != 0;

        if !is_heap {
            rstring.as_.embed.ary.as_ptr() as *const c_char
        } else {
            rstring.as_.heap.ptr
        }
    }

    unsafe fn rarray_len(obj: VALUE) -> c_long {
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

    unsafe fn rarray_const_ptr(obj: VALUE) -> *const VALUE {
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
