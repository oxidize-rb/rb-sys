//! Helpers for Ruby's rarray.h
//!
//! Makes it easier to reference to use the rarray.h API.

#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

#[cfg(version_gte_3_0 = "true")]
use crate::ruby_rarray_consts::RARRAY_EMBED_LEN_SHIFT;
#[cfg(version_gte_3_0 = "false")]
use crate::ruby_rarray_flags::RARRAY_EMBED_LEN_SHIFT;
use crate::{
    ruby_rarray_flags::{RARRAY_EMBED_FLAG, RARRAY_EMBED_LEN_MASK},
    RArray, RBasic, RB_BUILTIN_TYPE, RUBY_T_ARRAY, VALUE,
};
use std::{convert::TryInto, os::raw::c_long};

/// Queries the length of the array.
///
/// @param[in]  a  Array in question.
/// @return     Its number of elements.
/// @pre        `a` must be an instance of ::RArray.
///
/// # Safety
/// This function is unsafe because it dereferences a raw pointer to access the
/// underlying [`RArray`] struct.
#[inline(always)]
pub unsafe fn RARRAY_LEN(obj: VALUE) -> c_long {
    debug_assert!(RB_BUILTIN_TYPE(obj) == RUBY_T_ARRAY);

    let rarray = obj as *const RArray;
    let rbasic = obj as *const RBasic;
    let flags = (*rbasic).flags;

    if flags & RARRAY_EMBED_FLAG as VALUE != 0 {
        let masked = flags & RARRAY_EMBED_LEN_MASK as VALUE;
        let shifted = masked >> RARRAY_EMBED_LEN_SHIFT as VALUE;
        shifted.try_into().unwrap()
    } else {
        (*rarray).as_.heap.len
    }
}
