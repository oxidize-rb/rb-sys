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

#[cfg(not(ruby_eq_3_0))]
compile_error!("This file should only be included in Ruby 3.0 builds");

pub struct Definition;

impl StableApiDefinition for Definition {
    const VERSION_MAJOR: u32 = 3;
    const VERSION_MINOR: u32 = 0;

    #[inline]
    unsafe fn rstring_len(&self, obj: VALUE) -> c_long {
        unsafe {
            assert!(self.type_p(obj, crate::ruby_value_type::RUBY_T_STRING));

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
    unsafe fn rstring_ptr(&self, obj: VALUE) -> *const c_char {
        unsafe {
            assert!(self.type_p(obj, crate::ruby_value_type::RUBY_T_STRING));

            let rstring: &RString = &*(obj as *const RString);
            let flags = rstring.basic.flags;
            let is_heap = (flags & crate::ruby_rstring_flags::RSTRING_NOEMBED as VALUE) != 0;
            let ptr = if !is_heap {
                std::ptr::addr_of!(rstring.as_.ary) as *const _
            } else {
                rstring.as_.heap.ptr
            };

            assert!(!ptr.is_null());

            ptr
        }
    }

    #[inline]
    unsafe fn rarray_len(&self, obj: VALUE) -> c_long {
        unsafe {
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
    }

    #[inline]
    unsafe fn rarray_const_ptr(&self, obj: VALUE) -> *const VALUE {
        unsafe {
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
    }

    #[inline]
    unsafe fn rarray_aref(&self, obj: VALUE, idx: isize) -> VALUE {
        *self.rarray_const_ptr(obj).offset(idx)
    }

    #[inline]
    unsafe fn rarray_aset(&self, obj: VALUE, idx: isize, val: VALUE) {
        let ptr = self.rarray_const_ptr(obj).cast_mut().offset(idx);
        self.rb_obj_write(obj, ptr, val);
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
        let is_immediate = value & (crate::special_consts::IMMEDIATE_MASK as VALUE) != 0;
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

    unsafe fn symbol_p(&self, obj: VALUE) -> bool {
        self.static_sym_p(obj) || self.dynamic_sym_p(obj)
    }

    unsafe fn float_type_p(&self, obj: VALUE) -> bool {
        if self.flonum_p(obj) {
            true
        } else if self.special_const_p(obj) {
            false
        } else {
            self.builtin_type(obj) == value_type::RUBY_T_FLOAT
        }
    }

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
        let rdata = obj as *const RTypedData;
        let typed_flag = (*rdata).typed_flag;
        // Valid typed_flag value for Ruby 3.0 and earlier is only 1
        typed_flag == 1
    }

    #[inline]
    unsafe fn rtypeddata_type(&self, obj: VALUE) -> *const crate::rb_data_type_t {
        debug_ruby_assert_type!(
            obj,
            RUBY_T_DATA,
            "rtypeddata_type called on non-T_DATA object"
        );

        let rdata = obj as *const RTypedData;
        (*rdata).type_
    }

    #[inline]
    unsafe fn rtypeddata_get_data(&self, obj: VALUE) -> *mut c_void {
        debug_ruby_assert_type!(
            obj,
            RUBY_T_DATA,
            "rtypeddata_get_data called on non-T_DATA object"
        );

        // For Ruby 3.0 and lower, simply return the data field
        let rdata = obj as *const RTypedData;
        (*rdata).data
    }

    #[inline]
    fn fix2long(&self, obj: VALUE) -> c_long {
        // Extract the integer value by performing an arithmetic right shift by 1
        (obj as c_long) >> 1
    }

    #[inline]
    fn fix2ulong(&self, obj: VALUE) -> std::os::raw::c_ulong {
        // For positive fixnums, cast to c_long then to c_ulong
        ((obj as c_long) >> 1) as std::os::raw::c_ulong
    }

    #[inline]
    fn long2fix(&self, val: c_long) -> VALUE {
        // Left shift by 1 and OR with FIXNUM_FLAG
        (((val as VALUE) << 1) | crate::FIXNUM_FLAG as VALUE) as VALUE
    }

    #[inline]
    fn fixable(&self, val: c_long) -> bool {
        // Check if value is within Fixnum range
        val >= crate::special_consts::FIXNUM_MIN && val <= crate::special_consts::FIXNUM_MAX
    }

    #[inline]
    fn posfixable(&self, val: std::os::raw::c_ulong) -> bool {
        // Check if unsigned value fits in positive fixnum
        val <= crate::special_consts::FIXNUM_MAX as std::os::raw::c_ulong
    }

    #[inline]
    unsafe fn num2long(&self, obj: VALUE) -> c_long {
        if self.fixnum_p(obj) {
            self.fix2long(obj)
        } else {
            #[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
            if self.type_p(obj, crate::ruby_value_type::RUBY_T_BIGNUM) {
                if let Some(v) = bignum_to_long_fast(obj) {
                    return v;
                }
            }
            crate::rb_num2long(obj)
        }
    }

    #[inline]
    unsafe fn num2ulong(&self, obj: VALUE) -> std::os::raw::c_ulong {
        if self.fixnum_p(obj) {
            self.fix2ulong(obj)
        } else {
            #[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
            if self.type_p(obj, crate::ruby_value_type::RUBY_T_BIGNUM) {
                if let Some(v) = bignum_to_ulong_fast(obj) {
                    return v;
                }
            }
            crate::rb_num2ulong(obj)
        }
    }

    #[inline]
    fn long2num(&self, val: c_long) -> VALUE {
        if self.fixable(val) {
            self.long2fix(val)
        } else {
            unsafe { crate::rb_int2big(val as isize) }
        }
    }

    #[inline]
    fn ulong2num(&self, val: std::os::raw::c_ulong) -> VALUE {
        if self.posfixable(val) {
            self.long2fix(val as c_long)
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
        *slot = young;
        self.rb_obj_written(old, crate::Qundef as VALUE, young)
    }

    #[inline]
    unsafe fn rb_obj_written(&self, old: VALUE, _oldv: VALUE, young: VALUE) -> VALUE {
        if !self.special_const_p(young) {
            crate::rb_gc_writebarrier(old, young);
        }
        old
    }
    #[inline]
    fn fl_able(&self, obj: VALUE) -> bool {
        !self.special_const_p(obj)
    }

    #[inline]
    unsafe fn rstring_end(&self, obj: VALUE) -> *const c_char {
        assert!(self.type_p(obj, crate::ruby_value_type::RUBY_T_STRING));

        let ptr = self.rstring_ptr(obj);
        let len = self.rstring_len(obj);
        ptr.add(len as usize)
    }

    #[inline]
    unsafe fn rdata_ptr(&self, obj: VALUE) -> *mut c_void {
        assert!(self.type_p(obj, RUBY_T_DATA));

        let rdata = obj as *const RTypedData;
        (*rdata).data
    }

    #[inline]
    unsafe fn rb_obj_freeze(&self, obj: VALUE) {
        crate::rb_obj_freeze(obj);
    }

    #[inline]
    unsafe fn rb_obj_promoted(&self, obj: VALUE) -> bool {
        if self.special_const_p(obj) {
            false
        } else {
            self.rb_obj_promoted_raw(obj)
        }
    }

    #[inline]
    unsafe fn rb_obj_promoted_raw(&self, obj: VALUE) -> bool {
        let rbasic = obj as *const crate::RBasic;
        ((*rbasic).flags & crate::ruby_fl_type::RUBY_FL_PROMOTED as VALUE) != 0
    }

    #[inline]
    unsafe fn num2dbl(&self, obj: VALUE) -> std::os::raw::c_double {
        if self.flonum_p(obj) {
            // Fast path: decode Flonum directly
            #[cfg(ruby_use_flonum = "true")]
            {
                if obj != 0x8000000000000002 {
                    let b63 = obj >> 63;
                    let adjusted = ((2 - b63) | (obj & !0x03)) as u64;
                    let rotated = adjusted.rotate_right(3);
                    f64::from_bits(rotated)
                } else {
                    0.0
                }
            }
            #[cfg(not(ruby_use_flonum = "true"))]
            {
                crate::rb_num2dbl(obj)
            }
        } else if self.fixnum_p(obj) {
            // Fast path: convert Fixnum to double
            let long_val = (obj as c_long) >> 1;
            long_val as std::os::raw::c_double
        } else if !self.special_const_p(obj)
            && self.builtin_type(obj) == crate::ruby_value_type::RUBY_T_FLOAT
        {
            // Fast path: heap Float — read RFloat.float_value directly.
            // RFloat = { RBasic basic (2*sizeof(VALUE) bytes); double float_value; }
            // Avoids a dylib call for the common heap-Float case.
            // SAFETY: builtin_type check guarantees obj is a valid heap RFloat pointer.
            #[cfg(not(target_pointer_width = "32"))]
            {
                let float_val_ptr =
                    (obj as *const crate::VALUE).add(2) as *const std::os::raw::c_double;
                *float_val_ptr
            }
            #[cfg(target_pointer_width = "32")]
            {
                crate::rb_num2dbl(obj)
            }
        } else {
            // Slow path: Bignum, coercion (to_f), TypeError, etc.
            crate::rb_num2dbl(obj)
        }
    }

    #[inline(always)]
    fn dbl2num(&self, val: std::os::raw::c_double) -> VALUE {
        #[cfg(ruby_use_flonum = "true")]
        {
            let bits = val.to_bits() as VALUE;
            let exp_bits = (bits >> 60) & 0x7;
            // Flonum-representable: exponent top-3 bits are 011 or 100
            if bits != 0x3000_0000_0000_0000 && (exp_bits == 3 || exp_bits == 4) {
                return (bits.rotate_left(3) & !0x01) | 0x02;
            }
            // +0.0 special case
            if bits == 0 {
                return 0x8000_0000_0000_0002;
            }
        }
        // Out-of-flonum-range or flonum disabled: heap allocate
        unsafe { crate::rb_float_new(val) }
    }

    #[inline(always)]
    unsafe fn rhash_size(&self, obj: VALUE) -> usize {
        // Ruby 3.0 RHash layout (pre-3.3):
        //   struct RHash { RBasic basic; union { st_table *st; ar_table *ar; } as; VALUE ifnone; };
        //
        // AR mode (RUBY_FL_USER3 not set): size is packed in
        //   RBasic.flags bits [USER4..USER7] >> 16 (= FL_USHIFT+4).
        // ST mode (RUBY_FL_USER3 set): dereference the st_table pointer in
        //   RHash.as and read st_table.num_entries directly.
        //
        // SAFETY: caller guarantees obj is a valid T_HASH VALUE.
        #[repr(C)]
        struct RHashPre33 {
            basic: crate::RBasic,
            st: *mut crate::st_table, // union { st_table *st; ar_table *ar; } — same size
            ifnone: VALUE,
        }

        let rhash = obj as *const RHashPre33;
        let flags = (*rhash).basic.flags;

        // RHASH_ST_TABLE_FLAG = FL_USER3 = 32768
        let st_flag = crate::ruby_fl_type::RUBY_FL_USER3 as VALUE;
        if (flags & st_flag) == 0 {
            // AR mode: size encoded in bits [USER4..USER7].
            // RHASH_AR_TABLE_SIZE_MASK = FL_USER4|FL_USER5|FL_USER6|FL_USER7 = 0x000F_0000
            // RHASH_AR_TABLE_SIZE_SHIFT = FL_USHIFT + 4 = 12 + 4 = 16
            let mask: VALUE = (crate::ruby_fl_type::RUBY_FL_USER4 as VALUE)
                | (crate::ruby_fl_type::RUBY_FL_USER5 as VALUE)
                | (crate::ruby_fl_type::RUBY_FL_USER6 as VALUE)
                | (crate::ruby_fl_type::RUBY_FL_USER7 as VALUE);
            // RHASH_AR_TABLE_SIZE_SHIFT = FL_USHIFT + 4 = 12 + 4 = 16 (stable across all Ruby versions)
            let shift = 16u32;
            ((flags & mask) >> shift) as usize
        } else {
            // ST mode: dereference the st_table pointer and read num_entries.
            // SAFETY: the st pointer is valid when RHASH_ST_TABLE_FLAG is set.
            let st = (*rhash).st;
            (*st).num_entries as usize
        }
    }

    #[inline(always)]
    unsafe fn rhash_empty_p(&self, obj: VALUE) -> bool {
        self.rhash_size(obj) == 0
    }

    #[inline]
    unsafe fn encoding_get(&self, obj: VALUE) -> std::os::raw::c_int {
        // Fast path: encoding index is stored inline in the flags when
        // < RUBY_ENCODING_INLINE_MAX (0x7f). Only fall back to the libruby
        // function for out-of-line encodings (rare in practice).
        // Matches CRuby's `ENCODING_GET` inline function semantics.
        let rbasic = obj as *const crate::RBasic;
        let flags = (*rbasic).flags;
        let shift = crate::ruby_encoding_consts::RUBY_ENCODING_SHIFT as u32;
        let inline_max =
            crate::ruby_encoding_consts::RUBY_ENCODING_INLINE_MAX as std::os::raw::c_int;
        let mask = crate::ruby_encoding_consts::RUBY_ENCODING_MASK as VALUE;
        let inline_idx = ((flags & mask) >> shift) as std::os::raw::c_int;
        if inline_idx == inline_max {
            crate::rb_enc_get_index(obj)
        } else {
            inline_idx
        }
    }
}

// SAFETY: RBignum layout is stable across MRI 2.7–master on 64-bit.
// On 64-bit: BDIGIT = u32, BDIGIT_DBL = u64, BIGNUM_EMBED_LEN_MAX = 2.
// RBasic is 16 bytes (flags + klass), union as_ starts at offset 16.
// Embedded digits: as.ary[0..len] are u32 stored at offset 16..24.
// Heap: as.heap.len (usize, offset 16) + as.heap.digits (*u32, offset 24).
//
// Flag constants (from ruby_fl_type):
//   RUBY_FL_USER1 = 8192  = 0x2000  → BIGNUM sign (set = positive)
//   RUBY_FL_USER2 = 16384 = 0x4000  → BIGNUM_EMBED_FLAG (set = embedded)
//   RUBY_FL_USER3 = 32768 = 0x8000  \
//   RUBY_FL_USER4 = 65536 = 0x1_0000  > BIGNUM_EMBED_LEN_MASK (3-bit digit count)
//   RUBY_FL_USER5 = 131072= 0x2_0000  /
//   BIGNUM_EMBED_LEN_SHIFT = RUBY_FL_USHIFT + 3 = 12 + 3 = 15
#[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
#[repr(C)]
struct RBignum {
    basic: crate::RBasic,
    as_: RBignumAs,
}

#[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
#[repr(C)]
union RBignumAs {
    heap: RBignumHeap,
    // BIGNUM_EMBED_LEN_MAX = sizeof(u64)/sizeof(u32) = 2 on 64-bit
    ary: [u32; 2],
}

#[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
#[repr(C)]
#[derive(Copy, Clone)]
struct RBignumHeap {
    len: usize,
    digits: *const u32,
}

/// Fast path: read BDIGIT digits directly from RBignum to convert to i64 (c_long).
/// Returns None if the value overflows i64 or if digits > 2 (heap bignum).
/// Falls back to crate::rb_num2long which raises RangeError for overflow.
#[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
#[inline]
unsafe fn bignum_to_long_fast(obj: VALUE) -> Option<std::os::raw::c_long> {
    let rb = obj as *const RBignum;
    let flags = (*rb).basic.flags;

    // BIGNUM_EMBED_FLAG = RUBY_FL_USER2 = 16384 = 0x4000
    let embed_flag: VALUE = 16384;
    // BIGNUM_EMBED_LEN_MASK = USER3 | USER4 | USER5 = 32768 | 65536 | 131072 = 229376
    let embed_len_mask: VALUE = 229376;
    // BIGNUM_EMBED_LEN_SHIFT = RUBY_FL_USHIFT + 3 = 15
    let embed_len_shift: u32 = 15;
    // RUBY_FL_USER1 = 8192: set means positive
    let sign_flag: VALUE = 8192;
    let positive = (flags & sign_flag) != 0;

    let (len, digits_ptr) = if (flags & embed_flag) != 0 {
        // Embedded: digit count stored in flags[17:15], digits in as_.ary
        let len = ((flags & embed_len_mask) >> embed_len_shift) as usize;
        let digits = (*rb).as_.ary.as_ptr();
        (len, digits)
    } else {
        // Heap: len in as_.heap.len, digits in as_.heap.digits
        let len = (*rb).as_.heap.len;
        let digits = (*rb).as_.heap.digits;
        (len, digits)
    };

    match len {
        0 => Some(0),
        1 => {
            // Single BDIGIT (u32): max 0xFFFF_FFFF = 4294967295 < i64::MAX — always fits
            let d0 = *digits_ptr as u64;
            if positive {
                Some(d0 as std::os::raw::c_long)
            } else {
                Some(-(d0 as i64) as std::os::raw::c_long)
            }
        }
        2 => {
            // Two BDIGITs: check if combined value fits in i64
            let lo = *digits_ptr as u64;
            let hi = *digits_ptr.add(1) as u64;
            let val = lo | (hi << 32);
            if positive {
                if val > i64::MAX as u64 {
                    return None; // overflows i64, fall back to rb_num2long
                }
                Some(val as std::os::raw::c_long)
            } else {
                if val > (i64::MAX as u64) + 1 {
                    return None; // |val| > i64::MIN, fall back
                }
                Some((val as i64).wrapping_neg() as std::os::raw::c_long)
            }
        }
        _ => None, // 3+ digits: doesn't fit in i64
    }
}

/// Fast path: read BDIGIT digits directly from RBignum to convert to u64 (c_ulong).
/// Returns None if the bignum is negative or overflows u64.
/// Falls back to crate::rb_num2ulong which handles errors.
#[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
#[inline]
unsafe fn bignum_to_ulong_fast(obj: VALUE) -> Option<std::os::raw::c_ulong> {
    let rb = obj as *const RBignum;
    let flags = (*rb).basic.flags;

    let embed_flag: VALUE = 16384;
    let embed_len_mask: VALUE = 229376;
    let embed_len_shift: u32 = 15;
    let sign_flag: VALUE = 8192;
    let positive = (flags & sign_flag) != 0;

    // For num2ulong, negative bignums are not an error (Ruby's rb_num2ulong wraps them),
    // so we only fast-path positive values that fit in u64.
    // Negative bignums with > 2 digits always overflow u64 too, so fall back for all negatives.
    if !positive {
        return None; // let rb_num2ulong handle negative bignums (it wraps them)
    }

    let (len, digits_ptr) = if (flags & embed_flag) != 0 {
        let len = ((flags & embed_len_mask) >> embed_len_shift) as usize;
        let digits = (*rb).as_.ary.as_ptr();
        (len, digits)
    } else {
        let len = (*rb).as_.heap.len;
        let digits = (*rb).as_.heap.digits;
        (len, digits)
    };

    match len {
        0 => Some(0),
        1 => {
            let d0 = *digits_ptr as u64;
            Some(d0 as std::os::raw::c_ulong)
        }
        2 => {
            // Two positive BDIGITs always fit in u64
            let lo = *digits_ptr as u64;
            let hi = *digits_ptr.add(1) as u64;
            Some((lo | (hi << 32)) as std::os::raw::c_ulong)
        }
        _ => None, // 3+ digits: may exceed u64, fall back
    }
}
