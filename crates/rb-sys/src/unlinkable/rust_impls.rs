//! Rust implemenations of Ruby preprocessor macros and inlined functions.

#[cfg(all(
    not(ruby_abi_stable),
    not(feature = "bypass-stable-abi-version-checks")
))]
compile_error!("this module can only be used when in stable Ruby ABI mode, to bypass this check, enable the `bypass-stable-abi-version-checks` feature.");

use std::ffi::{c_char, c_long};

use crate::internal::{RArray, RString};
use crate::ruby_rarray_flags::{RARRAY_EMBED_FLAG, RARRAY_EMBED_LEN_MASK};
use crate::ruby_rstring_flags::RSTRING_NOEMBED;
use crate::{value_type, RB_TYPE_P, VALUE};

#[cfg(any(
    ruby_eq_2_4,
    ruby_eq_2_5,
    ruby_eq_2_6,
    ruby_eq_2_7,
    ruby_eq_3_0,
    ruby_eq_3_1,
    ruby_eq_3_2,
    ruby_eq_3_3,
))]
#[inline(always)]
pub unsafe fn rstring_len(str: VALUE) -> c_long {
    assert!(RB_TYPE_P(str) == value_type::RUBY_T_STRING);

    let rstring = &*(str as *const RString);

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
    ruby_eq_3_3,
))]
#[inline(always)]
pub unsafe fn rstring_ptr(str: VALUE) -> *const c_char {
    let rstring = &*(str as *const RString);

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
    ruby_eq_3_3,
))]
#[inline(always)]
pub unsafe fn rarray_len(value: VALUE) -> c_long {
    assert!(RB_TYPE_P(value) == value_type::RUBY_T_ARRAY);

    #[cfg(ruby_gte_3_0)]
    use crate::ruby_rarray_consts::RARRAY_EMBED_LEN_SHIFT;
    #[cfg(ruby_lt_3_0)]
    use crate::ruby_rarray_flags::RARRAY_EMBED_LEN_SHIFT;

    let rarray = &*(value as *const RArray);

    if is_flag_enabled(rarray.basic.flags as _, RARRAY_EMBED_FLAG as _) {
        let len = (rarray.basic.flags >> RARRAY_EMBED_LEN_SHIFT as VALUE)
            & (RARRAY_EMBED_LEN_MASK as VALUE >> RARRAY_EMBED_LEN_SHIFT as VALUE);

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
    ruby_eq_3_3,
))]
#[inline(always)]
pub unsafe fn rarray_const_ptr(value: VALUE) -> *const VALUE {
    assert!(RB_TYPE_P(value) == value_type::RUBY_T_ARRAY);

    let rarray = &*(value as *const RArray);

    if is_flag_enabled(rarray.basic.flags as _, RARRAY_EMBED_FLAG as _) {
        rarray.as_.ary.as_ptr()
    } else {
        rarray.as_.heap.ptr
    }
}

#[inline(always)]
fn is_flag_enabled(flags: u32, flag: u32) -> bool {
    (flags & flag) != 0
}
