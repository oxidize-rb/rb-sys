//! Rust implemenations of Ruby preprocessor macros and inlined functions.

use crate::{
    internal::{RArray, RString},
    value_type, RB_TYPE_P, VALUE,
};
use std::ffi::{c_char, c_long};

pub trait Embeddable: Sized + Copy {
    type PointerType;

    unsafe fn from_value(value: VALUE) -> Self;
    unsafe fn is_embedded(self) -> bool;
    unsafe fn embed_len(self) -> c_long;
    unsafe fn heap_len(self) -> c_long;
    unsafe fn embed_ptr(self) -> *const Self::PointerType;
    unsafe fn heap_ptr(self) -> *const Self::PointerType;
    unsafe fn len(self) -> c_long {
        if self.is_embedded() {
            self.embed_len()
        } else {
            self.heap_len()
        }
    }
    unsafe fn ptr(self) -> *const Self::PointerType {
        if self.is_embedded() {
            self.embed_ptr()
        } else {
            self.heap_ptr()
        }
    }
}

#[cfg(ruby_gte_3_0)]
impl<'a> Embeddable for &'a RString {
    type PointerType = c_char;

    unsafe fn from_value(value: VALUE) -> Self {
        assert!(RB_TYPE_P(value) == value_type::RUBY_T_STRING);

        &*(value as *const RString)
    }

    unsafe fn is_embedded(self) -> bool {
        let flags = self.basic.flags;
        let is_heap = (flags & crate::ruby_rstring_flags::RSTRING_NOEMBED as VALUE) != 0;
        !is_heap
    }

    #[cfg(ruby_lt_3_2)]
    unsafe fn embed_len(self) -> c_long {
        #[cfg(all(ruby_gte_3_0, ruby_lt_3_2))]
        use crate::ruby_rstring_consts::RSTRING_EMBED_LEN_SHIFT;
        #[cfg(ruby_lt_3_0)]
        use crate::ruby_rstring_flags::RSTRING_EMBED_LEN_SHIFT;

        let mut f = self.basic.flags;
        f &= crate::ruby_rstring_flags::RSTRING_EMBED_LEN_MASK as VALUE;
        f >>= RSTRING_EMBED_LEN_SHIFT as VALUE;
        f as _
    }

    #[cfg(ruby_gte_3_2)]
    unsafe fn embed_len(self) -> c_long {
        self.as_.embed.len as _
    }

    #[cfg(ruby_gte_3_1)]
    unsafe fn embed_ptr(self) -> *const c_char {
        self.as_.embed.ary.as_ptr() as _
    }

    #[cfg(ruby_lt_3_1)]
    unsafe fn embed_ptr(self) -> *const c_char {
        self.as_.ary.as_ptr() as *const _
    }

    unsafe fn heap_len(self) -> c_long {
        self.as_.heap.len
    }

    unsafe fn heap_ptr(self) -> *const c_char {
        self.as_.heap.ptr
    }
}

impl<'a> Embeddable for &'a RArray {
    type PointerType = VALUE;

    unsafe fn from_value(value: VALUE) -> Self {
        assert!(RB_TYPE_P(value) == value_type::RUBY_T_ARRAY);

        &*(value as *const RArray)
    }

    unsafe fn is_embedded(self) -> bool {
        let flags = self.basic.flags;
        (flags & crate::ruby_rarray_flags::RARRAY_EMBED_FLAG as VALUE) != 0
    }

    unsafe fn embed_len(self) -> c_long {
        let mut f = self.basic.flags;
        f &= crate::ruby_rarray_flags::RARRAY_EMBED_LEN_MASK as VALUE;
        f >>= crate::ruby_rarray_consts::RARRAY_EMBED_LEN_SHIFT as VALUE;
        f as _
    }

    unsafe fn heap_len(self) -> c_long {
        self.as_.heap.len
    }

    unsafe fn embed_ptr(self) -> *const VALUE {
        self.as_.ary.as_ptr()
    }

    unsafe fn heap_ptr(self) -> *const VALUE {
        self.as_.heap.ptr
    }
}
