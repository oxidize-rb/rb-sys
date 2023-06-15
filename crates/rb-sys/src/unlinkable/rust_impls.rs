//! Rust implemenations of Ruby preprocessor macros and inlined functions.

use std::ffi::{c_char, c_long};

use crate::ruby_rarray_consts::RARRAY_EMBED_LEN_SHIFT;
use crate::ruby_rarray_flags::RARRAY_EMBED_FLAG;
use crate::ruby_rarray_flags::RARRAY_EMBED_LEN_MASK;
use crate::ruby_rstring_flags::RSTRING_NOEMBED;
use crate::{value_type, RB_TYPE_P, VALUE};

#[cfg(all(
    not(ruby_abi_stable),
    not(feature = "bypass-stable-abi-version-checks")
))]
compile_error!("this module can only be used when in stable Ruby ABI mode, to bypass this check, enable the `bypass-stable-abi-version-checks` feature.");

#[cfg(any(
    ruby_eq_2_4,
    ruby_eq_2_5,
    ruby_eq_2_6,
    ruby_eq_2_7,
    ruby_eq_3_0,
    ruby_eq_3_1,
    ruby_eq_3_2,
))]
#[inline(always)]
pub unsafe fn rstring_len(str: VALUE) -> c_long {
    assert!(RB_TYPE_P(str) == value_type::RUBY_T_STRING);

    let rstring = &*(str as *const crate::RString);

    if is_flag_enabled(rstring.basic.flags as _, RSTRING_NOEMBED as _) {
        rstring.as_.heap.len as _
    } else {
        rstring.as_.embed.len as _
    }
}

#[cfg(any(
    ruby_eq_2_4,
    ruby_eq_2_5,
    ruby_eq_2_6,
    ruby_eq_2_7,
    ruby_eq_3_0,
    ruby_eq_3_1,
    ruby_eq_3_2,
))]
#[inline(always)]
pub unsafe fn rstring_ptr(str: VALUE) -> *const c_char {
    let rstring = &*(str as *const crate::RString);

    if is_flag_enabled(rstring.basic.flags as _, RSTRING_NOEMBED as _) {
        rstring.as_.heap.ptr as _
    } else {
        rstring.as_.embed.ary.as_ptr() as _
    }
}

#[cfg(any(
    ruby_eq_2_4,
    ruby_eq_2_5,
    ruby_eq_2_6,
    ruby_eq_2_7,
    ruby_eq_3_0,
    ruby_eq_3_1,
    ruby_eq_3_2,
))]
#[inline(always)]
pub unsafe fn rarray_len(value: VALUE) -> c_long {
    assert!(RB_TYPE_P(value) == value_type::RUBY_T_ARRAY);

    let rarray = &*(value as *const crate::RArray);

    if is_flag_enabled(rarray.basic.flags as _, RARRAY_EMBED_FLAG as _) {
        let len = flags_shift(rarray.basic.flags as u32, RARRAY_EMBED_LEN_SHIFT as u32)
            & flags_shift(RARRAY_EMBED_LEN_MASK as u32, RARRAY_EMBED_LEN_SHIFT as u32);

        len as _
    } else {
        rarray.as_.heap.len as _
    }
}

#[cfg(any(
    ruby_eq_2_4,
    ruby_eq_2_5,
    ruby_eq_2_6,
    ruby_eq_2_7,
    ruby_eq_3_0,
    ruby_eq_3_1,
    ruby_eq_3_2,
))]
#[inline(always)]
pub unsafe fn rarray_const_ptr(value: VALUE) -> *const VALUE {
    assert!(RB_TYPE_P(value) == value_type::RUBY_T_ARRAY);

    let rarray = &*(value as *const crate::RArray);

    if is_flag_enabled(rarray.basic.flags as _, RARRAY_EMBED_FLAG as _) {
        rarray.as_.ary.as_ptr()
    } else {
        rarray.as_.heap.ptr
    }
}

/// Be careful to make same assumptions as C code
#[inline(always)]
fn flags_shift(flags: u32, shift: u32) -> u32 {
    (flags >> shift) & 1
}

/// Be careful to make same assumptions as C code
#[inline(always)]
fn is_flag_enabled(flags: u32, flag: u32) -> bool {
    (flags & flag) != 0
}
