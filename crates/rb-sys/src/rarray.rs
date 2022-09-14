//! Helpers for Ruby's rarray.h
//!
//! Makes it easier to reference to use the rarray.h API.

#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

#[cfg(version_gte_3_0 = "true")]
use crate::ruby_rarray_consts::RARRAY_EMBED_LEN_SHIFT;
#[cfg(version_gte_3_0 = "false")]
use crate::ruby_rarray_flags::{RARRAY_EMBED_LEN_SHIFT, RARRAY_TRANSIENT_FLAG};
use crate::{
    debug_assert_ruby_type, rb_gc_writebarrier_unprotect, refute_flag,
    ruby_fl_type::{RUBY_FL_USER12, RUBY_FL_USER14},
    ruby_rarray_flags::{RARRAY_EMBED_FLAG, RARRAY_EMBED_LEN_MASK, RARRAY_TRANSIENT_FLAG},
    ruby_xmalloc2, RArray, RBasic, RGENGC_WB_PROTECTED_ARRAY, RUBY_T_ARRAY, USE_TRANSIENT_HEAP,
    VALUE,
};
use std::{convert::TryInto, mem::size_of, os::raw::c_long, ptr::copy_nonoverlapping};

pub const RARRAY_SHARED_ROOT_FLAG: u32 = RUBY_FL_USER12 as _;
pub const RARRAY_PTR_IN_USE_FLAG: u32 = RUBY_FL_USER14 as _;
pub const RARRAY_SHARED_FLAG: u32 = crate::ruby_fl_type::RUBY_ELTS_SHARED as _;

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
    debug_assert_ruby_type!(obj, RUBY_T_ARRAY);

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

/// Wild  use of  a  C  pointer.  This  function  accesses  the backend  storage
/// directly.   This is  slower  than  #RARRAY_PTR_USE_TRANSIENT.  It  exercises
/// extra manoeuvres  to protect our generational  GC.  Use of this  function is
/// considered archaic.  Use a modern way instead.
///
/// @param[in]  ary  An object of ::RArray.
/// @return     The backend C array.
///
/// @internal
///
/// That said...  there are  extension libraries  in the wild  who uses  it.  We
/// cannot but continue supporting.
///
/// # Safety
/// This function is unsafe because it dereferences a raw pointer to access the
/// underlying [`RArray`] struct.
#[inline(always)]
pub unsafe fn RARRAY_PTR(obj: VALUE) -> *const VALUE {
    debug_assert_ruby_type!(obj, RUBY_T_ARRAY);

    if RGENGC_WB_PROTECTED_ARRAY == 1 {
        rb_gc_writebarrier_unprotect(obj);
    }

    let rarray = obj as *mut RArray;
    let rbasic = obj as *mut RBasic;
    let fl = (*rbasic).flags;

    // Implements https://github.com/ruby/ruby/blob/cfb9624460a295e4e1723301486d89058c228e07/array.c#L456
    if USE_TRANSIENT_HEAP == 1 && (fl & RARRAY_TRANSIENT_FLAG as VALUE) != 0 {
        refute_flag!(fl, RARRAY_SHARED_ROOT_FLAG);

        let old_ptr = (*rarray).as_.heap.ptr as *const VALUE;
        let capa = (*rarray).as_.heap.aux.capa;

        refute_flag!(fl, RARRAY_SHARED_FLAG as VALUE | RARRAY_EMBED_FLAG as VALUE);
        refute_flag!(fl, RARRAY_PTR_IN_USE_FLAG);

        let new_ptr = ruby_xmalloc2(capa.try_into().unwrap(), size_of::<VALUE>() as _);

        (*rbasic).flags &= !(RARRAY_TRANSIENT_FLAG as VALUE);
        copy_nonoverlapping(old_ptr, new_ptr as _, capa.try_into().unwrap());
        (*rarray).as_.heap.ptr = new_ptr as _;
    }

    if (fl & RARRAY_EMBED_FLAG as VALUE) != 0 {
        &(*rarray).as_.ary as *const VALUE
    } else {
        (*rarray).as_.heap.ptr as *const VALUE
    }
}
