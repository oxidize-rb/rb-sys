use super::StableApiDefinition;
use crate::{
    debug_ruby_assert_type,
    internal::{RArray, RString},
    ruby_value_type::RUBY_T_DATA,
    value_type, ID, VALUE,
};
use std::{
    os::raw::{c_char, c_long},
    ptr::NonNull,
    time::Duration,
};

#[cfg(not(ruby_eq_3_3))]
compile_error!("This file should only be included in Ruby 3.3 builds");

extern "C" {
    fn rb_obj_write(
        old: VALUE,
        slot: *mut VALUE,
        young: VALUE,
        file: *const c_char,
        line: c_long,
    ) -> VALUE;
    fn rb_obj_written(
        old: VALUE,
        oldv: VALUE,
        young: VALUE,
        file: *const c_char,
        line: c_long,
    ) -> VALUE;
}

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
    unsafe fn rtypeddata_p(&self, obj: VALUE) -> bool {
        debug_ruby_assert_type!(obj, RUBY_T_DATA, "rtypeddata_p called on non-T_DATA object");

        // Access the RTypedData struct
        let rdata = obj as *const crate::internal::RTypedData;
        let typed_flag = (*rdata).typed_flag;
        // Valid typed_flag values are 1, 2, or 3
        typed_flag != 0 && typed_flag <= 3
    }

    #[inline]
    unsafe fn rtypeddata_type(&self, obj: VALUE) -> *const crate::rb_data_type_t {
        debug_ruby_assert_type!(
            obj,
            RUBY_T_DATA,
            "rtypeddata_type called on non-T_DATA object"
        );

        let rdata = obj as *const crate::internal::RTypedData;
        (*rdata).type_
    }

    #[inline]
    unsafe fn rtypeddata_get_data(&self, obj: VALUE) -> *mut std::ffi::c_void {
        debug_ruby_assert_type!(
            obj,
            RUBY_T_DATA,
            "rtypeddata_get_data called on non-T_DATA object"
        );

        let rdata = obj as *const crate::internal::RTypedData;
        let typed_flag = (*rdata).typed_flag;
        #[cfg(target_pointer_width = "64")]
        const FLAG: u64 = crate::TYPED_DATA_EMBEDDED as u64;
        #[cfg(target_pointer_width = "32")]
        const FLAG: u32 = crate::TYPED_DATA_EMBEDDED as u32;

        if (typed_flag & FLAG) != 0 {
            // For embedded data, calculate pointer based on struct layout
            // The formula matches Ruby's implementation:
            // embedded_typed_data_size = sizeof(RTypedData) - sizeof(void *)
            const EMBEDDED_TYPED_DATA_SIZE: usize =
                std::mem::size_of::<crate::internal::RTypedData>()
                    - std::mem::size_of::<*mut std::ffi::c_void>();

            // Return address after the header as the data pointer
            (obj as *mut u8).add(EMBEDDED_TYPED_DATA_SIZE) as *mut std::ffi::c_void
        } else {
            // For non-embedded data, return the data field directly
            let rdata = obj as *const crate::internal::RTypedData;
            (*rdata).data
        }
    }

    #[inline]
    fn fix2long(&self, obj: VALUE) -> std::os::raw::c_long {
        // Extract the integer value by performing an arithmetic right shift by 1
        (obj as std::os::raw::c_long) >> 1
    }

    #[inline]
    fn fix2ulong(&self, obj: VALUE) -> std::os::raw::c_ulong {
        // For positive fixnums, cast to c_long then to c_ulong
        ((obj as std::os::raw::c_long) >> 1) as std::os::raw::c_ulong
    }

    #[inline]
    fn long2fix(&self, val: std::os::raw::c_long) -> VALUE {
        // Left shift by 1 and OR with FIXNUM_FLAG
        (((val as VALUE) << 1) | crate::FIXNUM_FLAG as VALUE) as VALUE
    }

    #[inline]
    fn fixable(&self, val: std::os::raw::c_long) -> bool {
        // Check if value is within Fixnum range
        val >= crate::special_consts::FIXNUM_MIN && val <= crate::special_consts::FIXNUM_MAX
    }

    #[inline]
    fn posfixable(&self, val: std::os::raw::c_ulong) -> bool {
        // Check if unsigned value fits in positive fixnum
        val <= crate::special_consts::FIXNUM_MAX as std::os::raw::c_ulong
    }

    #[inline]
    unsafe fn num2long(&self, obj: VALUE) -> std::os::raw::c_long {
        if self.fixnum_p(obj) {
            self.fix2long(obj)
        } else {
            crate::rb_num2long(obj)
        }
    }

    #[inline]
    unsafe fn num2ulong(&self, obj: VALUE) -> std::os::raw::c_ulong {
        if self.fixnum_p(obj) {
            self.fix2ulong(obj)
        } else {
            crate::rb_num2ulong(obj)
        }
    }

    #[inline]
    fn long2num(&self, val: std::os::raw::c_long) -> VALUE {
        if self.fixable(val) {
            self.long2fix(val)
        } else {
            unsafe { crate::rb_int2big(val as isize) }
        }
    }

    #[inline]
    fn ulong2num(&self, val: std::os::raw::c_ulong) -> VALUE {
        if self.posfixable(val) {
            self.long2fix(val as std::os::raw::c_long)
        } else {
            unsafe { crate::rb_uint2big(val as usize) }
        }
    }

    #[inline]
    fn id2sym(&self, id: ID) -> VALUE {
        // Static symbol encoding: (id << RUBY_SPECIAL_SHIFT) | RUBY_SYMBOL_FLAG
        ((id as VALUE) << crate::ruby_special_consts::RUBY_SPECIAL_SHIFT as VALUE)
            | crate::ruby_special_consts::RUBY_SYMBOL_FLAG as VALUE
    }

    #[inline]
    unsafe fn sym2id(&self, obj: VALUE) -> ID {
        if self.static_sym_p(obj) {
            // Static symbol: extract ID from tagged pointer
            (obj >> crate::ruby_special_consts::RUBY_SPECIAL_SHIFT as VALUE) as ID
        } else {
            // Dynamic symbol: call rb_sym2id
            crate::rb_sym2id(obj)
        }
    }

    #[inline]
    unsafe fn rb_obj_write(&self, old: VALUE, slot: *mut VALUE, young: VALUE) -> VALUE {
        rb_obj_write(old, slot, young, core::ptr::null(), 0)
    }

    #[inline]
    unsafe fn rb_obj_written(&self, old: VALUE, oldv: VALUE, young: VALUE) -> VALUE {
        rb_obj_written(old, oldv, young, core::ptr::null(), 0)
    }
}
