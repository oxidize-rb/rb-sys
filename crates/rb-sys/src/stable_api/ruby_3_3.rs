use super::StableApiDefinition;
use crate::{
    internal::{RArray, RString},
    ruby_fl_type, ruby_fl_ushift, value_type, VALUE,
};
use std::{
    ffi::{c_int, CStr},
    os::raw::{c_char, c_long},
    ptr::NonNull,
    time::Duration,
};

const RSTRUCT_EMBED_LEN_MASK: VALUE = (ruby_fl_type::RUBY_FL_USER7 as VALUE)
    | (ruby_fl_type::RUBY_FL_USER6 as VALUE)
    | (ruby_fl_type::RUBY_FL_USER5 as VALUE)
    | (ruby_fl_type::RUBY_FL_USER4 as VALUE)
    | (ruby_fl_type::RUBY_FL_USER3 as VALUE)
    | (ruby_fl_type::RUBY_FL_USER2 as VALUE)
    | (ruby_fl_type::RUBY_FL_USER1 as VALUE);

const RSTRUCT_EMBED_LEN_SHIFT: VALUE = (ruby_fl_ushift::RUBY_FL_USHIFT as VALUE) + 1;

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct HeapStructData {
    pub(crate) len: std::ffi::c_long,
    pub(crate) ptr: *const VALUE,
}

#[repr(C)]
pub(crate) union RStructUnion {
    pub(crate) heap: HeapStructData,
    // Match the C array[1] layout
    pub(crate) ary: [VALUE; 1],
}

#[repr(C)]
pub struct RStruct {
    pub(crate) basic: crate::RBasic,
    pub(crate) as_: RStructUnion,
}

#[cfg(not(ruby_eq_3_3))]
compile_error!("This file should only be included in Ruby 3.3 builds");

pub struct Definition;

impl StableApiDefinition for Definition {
    const VERSION_MAJOR: u32 = 3;
    const VERSION_MINOR: u32 = 3;

    #[inline]
    unsafe fn rstring_len(&self, obj: VALUE) -> c_long {
        assert!(self.type_p(obj, crate::ruby_value_type::RUBY_T_STRING));

        let rstring: &RString = &*(obj as *const RString);
        rstring.len
    }

    #[inline]
    unsafe fn rstring_ptr(&self, obj: VALUE) -> *const c_char {
        assert!(self.type_p(obj, crate::ruby_value_type::RUBY_T_STRING));

        let rstring: &RString = &*(obj as *const RString);
        let flags = rstring.basic.flags;
        let is_heap = (flags & crate::ruby_rstring_flags::RSTRING_NOEMBED as VALUE) != 0;
        let ptr = if !is_heap {
            std::ptr::addr_of!(rstring.as_.embed.ary) as *const _
        } else {
            rstring.as_.heap.ptr
        };

        assert!(!ptr.is_null());

        ptr
    }

    #[inline]
    unsafe fn rarray_len(&self, obj: VALUE) -> c_long {
        assert!(self.type_p(obj, value_type::RUBY_T_ARRAY));

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

    #[inline]
    unsafe fn rarray_const_ptr(&self, obj: VALUE) -> *const VALUE {
        assert!(self.type_p(obj, value_type::RUBY_T_ARRAY));

        let rarray: &RArray = &*(obj as *const RArray);
        let flags = rarray.basic.flags;
        let is_embedded = (flags & crate::ruby_rarray_flags::RARRAY_EMBED_FLAG as VALUE) != 0;
        let ptr = if is_embedded {
            std::ptr::addr_of!(rarray.as_.ary) as *const _
        } else {
            rarray.as_.heap.ptr
        };

        assert!(!ptr.is_null());

        ptr
    }

    #[inline]
    unsafe fn rbasic_class(&self, obj: VALUE) -> Option<NonNull<VALUE>> {
        let rbasic = obj as *const crate::RBasic;

        NonNull::<VALUE>::new((*rbasic).klass as _)
    }

    #[inline]
    unsafe fn frozen_p(&self, obj: VALUE) -> bool {
        if self.special_const_p(obj) {
            true
        } else {
            let rbasic = obj as *const crate::RBasic;
            ((*rbasic).flags & crate::ruby_fl_type::RUBY_FL_FREEZE as VALUE) != 0
        }
    }

    #[inline]
    unsafe fn bignum_positive_p(&self, obj: VALUE) -> bool {
        let rbasic = obj as *const crate::RBasic;

        ((*rbasic).flags & crate::ruby_fl_type::RUBY_FL_USER1 as VALUE) != 0
    }

    #[inline]
    fn special_const_p(&self, value: VALUE) -> bool {
        let is_immediate = (value) & (crate::special_consts::IMMEDIATE_MASK as VALUE) != 0;
        let test = (value & !(crate::Qnil as VALUE)) != 0;

        is_immediate || !test
    }

    #[inline]
    unsafe fn builtin_type(&self, obj: VALUE) -> crate::ruby_value_type {
        let rbasic = obj as *const crate::RBasic;
        let ret: u32 = ((*rbasic).flags & crate::ruby_value_type::RUBY_T_MASK as VALUE) as _;

        std::mem::transmute::<_, crate::ruby_value_type>(ret)
    }

    #[inline]
    fn nil_p(&self, obj: VALUE) -> bool {
        obj == (crate::Qnil as VALUE)
    }

    #[inline]
    fn fixnum_p(&self, obj: VALUE) -> bool {
        (obj & crate::FIXNUM_FLAG as VALUE) != 0
    }

    #[inline]
    fn static_sym_p(&self, obj: VALUE) -> bool {
        let mask = !(VALUE::MAX << crate::ruby_special_consts::RUBY_SPECIAL_SHIFT as VALUE);
        (obj & mask) == crate::ruby_special_consts::RUBY_SYMBOL_FLAG as VALUE
    }

    #[inline]
    fn flonum_p(&self, obj: VALUE) -> bool {
        #[cfg(ruby_use_flonum = "true")]
        let ret = (obj & crate::FLONUM_MASK as VALUE) == crate::FLONUM_FLAG as VALUE;

        #[cfg(not(ruby_use_flonum = "true"))]
        let ret = false;

        ret
    }

    #[inline]
    fn immediate_p(&self, obj: VALUE) -> bool {
        (obj & crate::special_consts::IMMEDIATE_MASK as VALUE) != 0
    }

    #[inline]
    fn rb_test(&self, obj: VALUE) -> bool {
        (obj & !(crate::Qnil as VALUE)) != 0
    }

    #[inline]
    unsafe fn type_p(&self, obj: VALUE, t: crate::ruby_value_type) -> bool {
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
            self.fixnum_p(obj)
        } else if t == RUBY_T_SYMBOL {
            self.symbol_p(obj)
        } else if t == RUBY_T_FLOAT {
            self.float_type_p(obj)
        } else if self.special_const_p(obj) {
            false
        } else if t == self.builtin_type(obj) {
            true
        } else {
            t == self.rb_type(obj)
        }
    }

    #[inline]
    unsafe fn symbol_p(&self, obj: VALUE) -> bool {
        self.static_sym_p(obj) || self.dynamic_sym_p(obj)
    }

    #[inline]
    unsafe fn float_type_p(&self, obj: VALUE) -> bool {
        if self.flonum_p(obj) {
            true
        } else if self.special_const_p(obj) {
            false
        } else {
            self.builtin_type(obj) == value_type::RUBY_T_FLOAT
        }
    }

    #[inline]
    unsafe fn rb_type(&self, obj: VALUE) -> crate::ruby_value_type {
        use crate::ruby_special_consts::*;
        use crate::ruby_value_type::*;

        if !self.special_const_p(obj) {
            self.builtin_type(obj)
        } else if obj == RUBY_Qfalse as _ {
            RUBY_T_FALSE
        } else if obj == RUBY_Qnil as _ {
            RUBY_T_NIL
        } else if obj == RUBY_Qtrue as _ {
            RUBY_T_TRUE
        } else if obj == RUBY_Qundef as _ {
            RUBY_T_UNDEF
        } else if self.fixnum_p(obj) {
            RUBY_T_FIXNUM
        } else if self.static_sym_p(obj) {
            RUBY_T_SYMBOL
        } else {
            debug_assert!(self.flonum_p(obj));
            RUBY_T_FLOAT
        }
    }

    #[inline]
    unsafe fn dynamic_sym_p(&self, obj: VALUE) -> bool {
        if self.special_const_p(obj) {
            false
        } else {
            self.builtin_type(obj) == value_type::RUBY_T_SYMBOL
        }
    }

    #[inline]
    unsafe fn integer_type_p(&self, obj: VALUE) -> bool {
        if self.fixnum_p(obj) {
            true
        } else if self.special_const_p(obj) {
            false
        } else {
            self.builtin_type(obj) == value_type::RUBY_T_BIGNUM
        }
    }

    #[inline]
    unsafe fn rstring_interned_p(&self, obj: VALUE) -> bool {
        assert!(self.type_p(obj, value_type::RUBY_T_STRING));

        let rstring: &RString = &*(obj as *const RString);
        let flags = rstring.basic.flags;

        (flags & crate::ruby_rstring_flags::RSTRING_FSTR as VALUE) != 0
    }

    #[inline]
    fn thread_sleep(&self, duration: Duration) {
        let seconds = duration.as_secs() as _;
        let microseconds = duration.subsec_micros() as _;

        let time = crate::timeval {
            tv_sec: seconds,
            tv_usec: microseconds,
        };

        unsafe { crate::rb_thread_wait_for(time) }
    }

    #[inline]
    fn rstruct_define(&self, name: &CStr, members: &[&CStr]) -> VALUE {
        let mut members: Vec<*const c_char> = members
            .iter()
            .map(|m| m.as_ptr() as *const c_char)
            .collect();
        members.push(std::ptr::null());
        unsafe { crate::rb_struct_define(name.as_ptr(), members) }
    }

    #[inline]
    unsafe fn rstruct_get(&self, st: VALUE, idx: c_int) -> VALUE {
        let rbasic = st as *const crate::RBasic;
        let rstruct = st as *const RStruct;
        let slice: &[VALUE] = if ((*rbasic).flags & RSTRUCT_EMBED_LEN_MASK as VALUE) != 0 {
            (*rstruct).as_.ary.as_slice()
        } else {
            let ptr = (*rstruct).as_.heap.ptr;
            let len = (*rstruct).as_.heap.len as _;
            std::slice::from_raw_parts(ptr, len)
        };

        slice[idx as usize]
    }

    // TODO: In CRuby this method has a write barrier. What do we do?
    #[inline]
    unsafe fn rstruct_set(&self, st: VALUE, idx: c_int, value: VALUE) {
        let rbasic = st as *const crate::RBasic;
        let rstruct = st as *mut RStruct;
        let slice: &mut [VALUE] = if ((*rbasic).flags & RSTRUCT_EMBED_LEN_MASK as VALUE) != 0 {
            (*rstruct).as_.ary.as_mut_slice()
        } else {
            let ptr = (*rstruct).as_.heap.ptr as *mut _;
            let len = (*rstruct).as_.heap.len as _;
            std::slice::from_raw_parts_mut(ptr, len)
        };

        slice[idx as usize] = value;
    }

    #[inline]
    unsafe fn rstruct_len(&self, st: VALUE) -> c_long {
        let rbasic = st as *const crate::RBasic;
        if ((*rbasic).flags & RSTRUCT_EMBED_LEN_MASK as VALUE) != 0 {
            let mut ret = ((*rbasic).flags & RSTRUCT_EMBED_LEN_MASK as VALUE) as c_long;
            ret >>= RSTRUCT_EMBED_LEN_SHIFT;
            ret
        } else {
            let rstruct = st as *const RStruct;
            (*rstruct).as_.heap.len
        }
    }
}
