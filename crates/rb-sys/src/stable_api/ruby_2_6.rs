use super::StableApiDefinition;

use crate::ruby_rarray_flags::*;
use crate::ruby_rstring_flags::*;
use crate::{
    internal::{RArray, RString},
    value_type, VALUE,
};
use std::os::raw::{c_char, c_long};

pub struct Definition;

impl StableApiDefinition for Definition {
    #[inline]
    unsafe fn rstring_len(obj: VALUE) -> c_long {
        assert!(Self::type_p(obj, crate::ruby_value_type::RUBY_T_STRING));
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
        assert!(Self::type_p(obj, crate::ruby_value_type::RUBY_T_STRING));
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
        assert!(Self::type_p(obj, value_type::RUBY_T_ARRAY));
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
        assert!(Self::type_p(obj, value_type::RUBY_T_ARRAY));
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
    unsafe fn builtin_type(obj: VALUE) -> crate::ruby_value_type {
        let rbasic = obj as *const crate::RBasic;
        let ret: u32 = ((*rbasic).flags & crate::ruby_value_type::RUBY_T_MASK as VALUE) as _;

        std::mem::transmute::<_, crate::ruby_value_type>(ret)
    }

    #[inline]
    fn nil_p(obj: VALUE) -> bool {
        obj == (crate::Qnil as VALUE)
    }

    #[inline]
    fn fixnum_p(obj: VALUE) -> bool {
        (obj & crate::FIXNUM_FLAG as VALUE) != 0
    }

    #[inline]
    fn static_sym_p(obj: VALUE) -> bool {
        let mask = !(VALUE::MAX << crate::ruby_special_consts::RUBY_SPECIAL_SHIFT as VALUE);
        (obj & mask) == crate::ruby_special_consts::RUBY_SYMBOL_FLAG as VALUE
    }

    #[inline]
    fn flonum_p(obj: VALUE) -> bool {
        #[cfg(ruby_use_flonum = "true")]
        let ret = (obj & crate::FLONUM_MASK as VALUE) == crate::FLONUM_FLAG as VALUE;

        #[cfg(not(ruby_use_flonum = "true"))]
        let ret = false;

        ret
    }

    #[inline]
    fn immediate_p(obj: VALUE) -> bool {
        (obj & crate::special_consts::IMMEDIATE_MASK as VALUE) != 0
    }

    #[inline]
    fn rb_test(obj: VALUE) -> bool {
        (obj & !(crate::Qnil as VALUE)) != 0
    }

    #[inline]
    unsafe fn type_p(obj: VALUE, t: crate::ruby_value_type) -> bool {
        use crate::ruby_special_consts::*;
        use crate::ruby_value_type::*;

        if t == RUBY_T_TRUE {
            obj == RUBY_Qtrue as _
        } else if t == RUBY_T_FALSE {
            obj == RUBY_Qfalse as _
        } else if t == RUBY_T_NIL {
            obj == RUBY_Qnil as _
        } else if t == RUBY_T_UNDEF {
            obj == RUBY_Qundef as _
        } else if t == RUBY_T_FIXNUM {
            Self::fixnum_p(obj)
        } else if t == RUBY_T_SYMBOL {
            Self::symbol_p(obj)
        } else if t == RUBY_T_FLOAT {
            Self::float_type_p(obj)
        } else if Self::special_const_p(obj) {
            false
        } else if t == Self::builtin_type(obj) {
            true
        } else {
            t == Self::rb_type(obj)
        }
    }

    unsafe fn symbol_p(obj: VALUE) -> bool {
        Self::static_sym_p(obj) || Self::dynamic_sym_p(obj)
    }

    unsafe fn float_type_p(obj: VALUE) -> bool {
        if Self::flonum_p(obj) {
            true
        } else if Self::special_const_p(obj) {
            false
        } else {
            Self::builtin_type(obj) == value_type::RUBY_T_FLOAT
        }
    }

    unsafe fn rb_type(obj: VALUE) -> crate::ruby_value_type {
        use crate::ruby_special_consts::*;
        use crate::ruby_value_type::*;

        if !Self::special_const_p(obj) {
            Self::builtin_type(obj)
        } else if obj == RUBY_Qfalse as _ {
            RUBY_T_FALSE
        } else if obj == RUBY_Qnil as _ {
            RUBY_T_NIL
        } else if obj == RUBY_Qtrue as _ {
            RUBY_T_TRUE
        } else if obj == RUBY_Qundef as _ {
            RUBY_T_UNDEF
        } else if Self::fixnum_p(obj) {
            RUBY_T_FIXNUM
        } else if Self::static_sym_p(obj) {
            RUBY_T_SYMBOL
        } else {
            debug_assert!(Self::flonum_p(obj));
            RUBY_T_FLOAT
        }
    }

    unsafe fn dynamic_sym_p(obj: VALUE) -> bool {
        if Self::special_const_p(obj) {
            false
        } else {
            Self::builtin_type(obj) == value_type::RUBY_T_SYMBOL
        }
    }

    #[inline]
    unsafe fn integer_type_p(obj: VALUE) -> bool {
        if Self::fixnum_p(obj) {
            true
        } else if Self::special_const_p(obj) {
            false
        } else {
            Self::builtin_type(obj) == value_type::RUBY_T_BIGNUM
        }
    }
}
