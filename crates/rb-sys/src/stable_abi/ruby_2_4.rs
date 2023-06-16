use super::StableAbiDefinition;
use crate::{
    internal::{RArray, RString},
    value_type::{self},
    VALUE,
};
use std::ffi::{c_char, c_long};

const RARRAY_EMBED_FLAG: u32 = 1 << 13;
const RARRAY_EMBED_LEN_SHIFT: u32 = 15;
const RARRAY_EMBED_LEN_MASK: u32 = crRUBY_FL_USER3 | RUBY_FL_USER4;
const RUBY_FL_USHIFT: u32 = 12;
const RUBY_FL_USER3: u32 = 1 << (RUBY_FL_USHIFT as u32 + 3);
const RUBY_FL_USER4: u32 = 1 << (RUBY_FL_USHIFT as u32 + 4);

pub struct Definition;

impl StableAbiDefinition for Definition {
    unsafe fn rstring_len(obj: VALUE) -> c_long {
        assert!(value_type::RB_TYPE_P(obj) == value_type::RUBY_T_STRING);
        let rstring: &RString = &*(obj as *const RString);
        let flags = rstring.basic.flags;
        let is_heap = (flags & RSTRING_NOEMBED as VALUE) != 0;

        if !is_heap {
            use RSTRING_EMBED_LEN_SHIFT;

            let mut f = rstring.basic.flags;
            f &= RSTRING_EMBED_LEN_MASK as VALUE;
            f >>= RSTRING_EMBED_LEN_SHIFT as VALUE;
            f as c_long
        } else {
            rstring.as_.heap.len
        }
    }

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
}
