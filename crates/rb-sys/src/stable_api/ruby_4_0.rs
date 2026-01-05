use super::StableApiDefinition;
use crate::{
    debug_ruby_assert_type,
    internal::{RArray, RString, RTypedData},
    ruby_value_type::RUBY_T_DATA,
    value_type, ID, VALUE,
};
use std::{
    ffi::c_void,
    os::raw::{c_char, c_long},
    ptr::NonNull,
    time::Duration,
};

#[cfg(not(ruby_eq_4_0))]
compile_error!("This file should only be included in Ruby 4.0 builds");

pub struct Definition;

impl StableApiDefinition for Definition {
    const VERSION_MAJOR: u32 = 4;
    const VERSION_MINOR: u32 = 0;

    #[inline(always)]
    unsafe fn rstring_len(&self, obj: VALUE) -> c_long {
        debug_ruby_assert_type!(
            obj,
            crate::ruby_value_type::RUBY_T_STRING,
            "rstring_len called on non-T_STRING object"
        );

        let rstring: &RString = &*(obj as *const RString);
        rstring.len
    }

    #[inline(always)]
    unsafe fn rstring_ptr(&self, obj: VALUE) -> *const c_char {
        debug_ruby_assert_type!(
            obj,
            crate::ruby_value_type::RUBY_T_STRING,
            "rstring_ptr called on non-T_STRING object"
        );

        let rstring: &RString = &*(obj as *const RString);
        let flags = rstring.basic.flags;
        let is_heap = (flags & crate::ruby_rstring_flags::RSTRING_NOEMBED as VALUE) != 0;
        if !is_heap {
            std::ptr::addr_of!(rstring.as_.embed.ary) as *const _
        } else {
            rstring.as_.heap.ptr
        }
    }

    #[inline(always)]
    unsafe fn rarray_len(&self, obj: VALUE) -> c_long {
        debug_ruby_assert_type!(
            obj,
            value_type::RUBY_T_ARRAY,
            "rarray_len called on non-T_ARRAY object"
        );

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

    #[inline(always)]
    unsafe fn rarray_const_ptr(&self, obj: VALUE) -> *const VALUE {
        debug_ruby_assert_type!(
            obj,
            value_type::RUBY_T_ARRAY,
            "rarray_const_ptr called on non-T_ARRAY object"
        );

        let rarray: &RArray = &*(obj as *const RArray);
        let flags = rarray.basic.flags;
        let is_embedded = (flags & crate::ruby_rarray_flags::RARRAY_EMBED_FLAG as VALUE) != 0;
        if is_embedded {
            std::ptr::addr_of!(rarray.as_.ary) as *const _
        } else {
            rarray.as_.heap.ptr
        }
    }

    #[inline(always)]
    unsafe fn rbasic_class(&self, obj: VALUE) -> Option<NonNull<VALUE>> {
        let rbasic = obj as *const crate::RBasic;

        NonNull::<VALUE>::new((*rbasic).klass as _)
    }

    #[inline(always)]
    unsafe fn frozen_p(&self, obj: VALUE) -> bool {
        if self.special_const_p(obj) {
            true
        } else {
            let rbasic = obj as *const crate::RBasic;
            ((*rbasic).flags & crate::ruby_fl_type::RUBY_FL_FREEZE as VALUE) != 0
        }
    }

    #[inline(always)]
    fn special_const_p(&self, value: VALUE) -> bool {
        // Checks if immediate (low 3 bits set) OR if it's a "falsy" value (Qnil/Qfalse)
        self.immediate_p(value) || !self.rb_test(value)
    }

    #[inline(always)]
    unsafe fn bignum_positive_p(&self, obj: VALUE) -> bool {
        let rbasic = obj as *const crate::RBasic;

        ((*rbasic).flags & crate::ruby_fl_type::RUBY_FL_USER1 as VALUE) != 0
    }

    #[inline(always)]
    unsafe fn builtin_type(&self, obj: VALUE) -> crate::ruby_value_type {
        let rbasic = obj as *const crate::RBasic;
        let ret: u32 = ((*rbasic).flags & crate::ruby_value_type::RUBY_T_MASK as VALUE) as _;

        std::mem::transmute::<_, crate::ruby_value_type>(ret)
    }

    #[inline(always)]
    fn nil_p(&self, obj: VALUE) -> bool {
        obj == (crate::Qnil as VALUE)
    }

    #[inline(always)]
    fn fixnum_p(&self, obj: VALUE) -> bool {
        (obj & crate::FIXNUM_FLAG as VALUE) != 0
    }

    #[inline(always)]
    fn static_sym_p(&self, obj: VALUE) -> bool {
        const SPECIAL_MASK: VALUE =
            !(VALUE::MAX << crate::ruby_special_consts::RUBY_SPECIAL_SHIFT as VALUE);
        const SYMBOL_FLAG: VALUE = crate::ruby_special_consts::RUBY_SYMBOL_FLAG as VALUE;
        (obj & SPECIAL_MASK) == SYMBOL_FLAG
    }

    #[inline(always)]
    fn flonum_p(&self, obj: VALUE) -> bool {
        #[cfg(ruby_use_flonum = "true")]
        let ret = (obj & crate::FLONUM_MASK as VALUE) == crate::FLONUM_FLAG as VALUE;

        #[cfg(not(ruby_use_flonum = "true"))]
        let ret = false;

        ret
    }

    #[inline(always)]
    fn immediate_p(&self, obj: VALUE) -> bool {
        (obj & crate::special_consts::IMMEDIATE_MASK as VALUE) != 0
    }

    #[inline(always)]
    fn rb_test(&self, obj: VALUE) -> bool {
        (obj & !(crate::Qnil as VALUE)) != 0
    }

    #[inline(always)]
    unsafe fn type_p(&self, obj: VALUE, ty: crate::ruby_value_type) -> bool {
        use crate::ruby_special_consts::*;
        use crate::ruby_value_type::*;

        if !self.special_const_p(obj) {
            self.builtin_type(obj) == ty
        } else if obj == RUBY_Qfalse as _ {
            ty == RUBY_T_FALSE
        } else if obj == RUBY_Qnil as _ {
            ty == RUBY_T_NIL
        } else if obj == RUBY_Qtrue as _ {
            ty == RUBY_T_TRUE
        } else if obj == RUBY_Qundef as _ {
            ty == RUBY_T_UNDEF
        } else if self.fixnum_p(obj) {
            ty == RUBY_T_FIXNUM
        } else if self.static_sym_p(obj) {
            ty == RUBY_T_SYMBOL
        } else if self.flonum_p(obj) {
            ty == RUBY_T_FLOAT
        } else {
            // Optimized: For heap objects, directly compare builtin_type
            // This eliminates the extra check in the original: else if ty == self.builtin_type(obj)
            ty == self.builtin_type(obj)
        }
    }

    #[inline(always)]
    unsafe fn symbol_p(&self, obj: VALUE) -> bool {
        // Partition by heap vs immediate - generates fewer branches than
        // checking static_sym first, since heap/immediate are mutually exclusive.
        if !self.special_const_p(obj) {
            self.builtin_type(obj) == value_type::RUBY_T_SYMBOL
        } else {
            self.static_sym_p(obj)
        }
    }

    #[inline(always)]
    unsafe fn float_type_p(&self, obj: VALUE) -> bool {
        // Partition by heap vs immediate - generates fewer branches than
        // checking flonum first, since heap/immediate are mutually exclusive.
        if !self.special_const_p(obj) {
            self.builtin_type(obj) == value_type::RUBY_T_FLOAT
        } else {
            self.flonum_p(obj)
        }
    }

    #[inline(always)]
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

    #[inline(always)]
    unsafe fn dynamic_sym_p(&self, obj: VALUE) -> bool {
        !self.special_const_p(obj) && self.builtin_type(obj) == value_type::RUBY_T_SYMBOL
    }

    #[inline(always)]
    unsafe fn integer_type_p(&self, obj: VALUE) -> bool {
        // Partition by heap vs immediate - generates fewer branches than
        // checking fixnum first, since heap/immediate are mutually exclusive.
        if !self.special_const_p(obj) {
            self.builtin_type(obj) == value_type::RUBY_T_BIGNUM
        } else {
            self.fixnum_p(obj)
        }
    }

    #[inline(always)]
    unsafe fn rstring_interned_p(&self, obj: VALUE) -> bool {
        debug_ruby_assert_type!(
            obj,
            value_type::RUBY_T_STRING,
            "rstring_interned_p called on non-T_STRING object"
        );

        let rstring: &RString = &*(obj as *const RString);
        let flags = rstring.basic.flags;

        (flags & crate::ruby_rstring_flags::RSTRING_FSTR as VALUE) != 0
    }

    #[inline(always)]
    fn thread_sleep(&self, duration: Duration) {
        let seconds = duration.as_secs() as _;
        let microseconds = duration.subsec_micros() as _;

        let time = crate::timeval {
            tv_sec: seconds,
            tv_usec: microseconds,
        };

        unsafe { crate::rb_thread_wait_for(time) }
    }

    #[inline(always)]
    unsafe fn rtypeddata_p(&self, obj: VALUE) -> bool {
        debug_ruby_assert_type!(obj, RUBY_T_DATA, "rtypeddata_p called on non-T_DATA object");

        // Ruby 4.0: Check RUBY_TYPED_FL_IS_TYPED_DATA flag (RUBY_FL_USERPRIV0)
        let rbasic = obj as *const crate::RBasic;
        ((*rbasic).flags & crate::ruby_fl_type::RUBY_FL_USERPRIV0 as VALUE) != 0
    }

    #[inline(always)]
    unsafe fn rtypeddata_embedded_p(&self, obj: VALUE) -> bool {
        debug_ruby_assert_type!(
            obj,
            RUBY_T_DATA,
            "rtypeddata_embedded_p called on non-T_DATA object"
        );

        // Ruby 4.0: TYPED_DATA_EMBEDDED is bit 0 of the type field
        let rdata = obj as *const RTypedData;
        ((*rdata).type_ & 1) != 0
    }

    #[inline(always)]
    unsafe fn rtypeddata_type(&self, obj: VALUE) -> *const crate::rb_data_type_t {
        debug_ruby_assert_type!(
            obj,
            RUBY_T_DATA,
            "rtypeddata_type called on non-T_DATA object"
        );

        // Ruby 4.0: type field stores pointer with embedded flag in low bit
        // Mask out the embedded flag to get the actual pointer
        let rdata = obj as *const RTypedData;
        ((*rdata).type_ & !1) as *const crate::rb_data_type_t
    }

    #[inline(always)]
    unsafe fn rtypeddata_get_data(&self, obj: VALUE) -> *mut c_void {
        debug_ruby_assert_type!(
            obj,
            RUBY_T_DATA,
            "rtypeddata_get_data called on non-T_DATA object"
        );

        if self.rtypeddata_embedded_p(obj) {
            // For embedded data, calculate pointer based on struct layout
            // The formula matches Ruby's implementation:
            // embedded_typed_data_size = sizeof(RTypedData) - sizeof(void *)
            const EMBEDDED_TYPED_DATA_SIZE: usize =
                std::mem::size_of::<RTypedData>() - std::mem::size_of::<*mut c_void>();

            // Return address after the header as the data pointer
            (obj as *mut u8).add(EMBEDDED_TYPED_DATA_SIZE) as *mut c_void
        } else {
            // For non-embedded data, return the data field directly
            let rdata = obj as *const RTypedData;
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
        (crate::special_consts::FIXNUM_MIN..=crate::special_consts::FIXNUM_MAX).contains(&val)
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

    #[inline(always)]
    fn id2sym(&self, id: ID) -> VALUE {
        // Static symbol encoding: (id << RUBY_SPECIAL_SHIFT) | RUBY_SYMBOL_FLAG
        ((id as VALUE) << crate::ruby_special_consts::RUBY_SPECIAL_SHIFT as VALUE)
            | crate::ruby_special_consts::RUBY_SYMBOL_FLAG as VALUE
    }

    #[inline(always)]
    unsafe fn sym2id(&self, obj: VALUE) -> ID {
        if self.static_sym_p(obj) {
            // Static symbol: extract ID from tagged pointer
            (obj >> crate::ruby_special_consts::RUBY_SPECIAL_SHIFT as VALUE) as ID
        } else {
            // Dynamic symbol: call rb_sym2id
            crate::rb_sym2id(obj)
        }
    }
}
