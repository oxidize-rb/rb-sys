//! Definitions for Ruby's special constants.
//!
//! Makes it easier to reference important Ruby constants, without havign to dig
//! around in bindgen's output.

#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use std::convert::TryInto;

use crate::{
    ruby_value_type::{self, RUBY_T_MASK},
    Qfalse, Qnil, Qtrue, Qundef, RBasic, FIXNUM_P, FLONUM_P, SPECIAL_CONST_P, STATIC_SYM_P, VALUE,
};

pub use ruby_value_type::*;

/// Queries the type of the object.
///
/// @param[in]  obj  Object in question.
/// @pre        `obj` must not be a special constant.
/// @return     The type of `obj`.
#[inline(always)]
pub unsafe fn RB_BUILTIN_TYPE(obj: VALUE) -> ruby_value_type {
    debug_assert!(!SPECIAL_CONST_P(obj));

    let rbasic = obj as *const RBasic;

    let ret = (*rbasic).flags & RUBY_T_MASK as VALUE;
    let ret: u32 = ret.try_into().unwrap();

    std::mem::transmute::<_, ruby_value_type>(ret)
}

/// Queries if the object is an instance of ::rb_cInteger.
///
/// @param[in]  obj    Object in question.
/// @retval     true   It is.
/// @retval     false  It isn't.
#[inline(always)]
pub unsafe fn RB_INTEGER_TYPE_P(obj: VALUE) -> bool {
    if FIXNUM_P(obj) {
        true
    } else if SPECIAL_CONST_P(obj) {
        false
    } else {
        RB_BUILTIN_TYPE(obj) == RUBY_T_BIGNUM
    }
}

/// Queries if the object is a dynamic symbol.
///
/// @param[in]  obj    Object in question.
/// @retval     true   It is.
/// @retval     false  It isn't.
#[inline(always)]
pub unsafe fn RB_DYNAMIC_SYM_P(obj: VALUE) -> bool {
    if SPECIAL_CONST_P(obj) {
        false
    } else {
        RB_BUILTIN_TYPE(obj) == RUBY_T_SYMBOL
    }
}

/// Queries if the object is an instance of ::rb_cSymbol.
///
/// @param[in]  obj    Object in question.
/// @retval     true   It is.
/// @retval     false  It isn't.
#[inline(always)]
pub unsafe fn RB_SYMBOL_P(obj: VALUE) -> bool {
    return STATIC_SYM_P(obj) || RB_DYNAMIC_SYM_P(obj);
}

/// Identical to RB_BUILTIN_TYPE(), except it can also accept special constants.
///
/// @param[in]  obj  Object in question.
/// @return     The type of `obj`.
#[inline(always)]
pub unsafe fn RB_TYPE_P<T: Into<VALUE>>(value: T) -> ruby_value_type {
    let obj = value.into();

    if !SPECIAL_CONST_P(obj) {
        return RB_BUILTIN_TYPE(obj);
    } else if obj == Qfalse as VALUE {
        return RUBY_T_FALSE;
    } else if obj == Qnil as VALUE {
        return RUBY_T_NIL;
    } else if obj == Qtrue as VALUE {
        return RUBY_T_TRUE;
    } else if obj == Qundef as VALUE {
        return RUBY_T_UNDEF;
    } else if FIXNUM_P(obj) {
        return RUBY_T_FIXNUM;
    } else if STATIC_SYM_P(obj) {
        return RUBY_T_SYMBOL;
    } else {
        debug_assert!(FLONUM_P(obj));
        return RUBY_T_FLOAT;
    }
}

/**
 * Queries if the object is an instance of ::rb_cFloat.
 *
 * @param[in]  obj    Object in question.
 * @retval     true   It is.
 * @retval     false  It isn't.
 */
pub unsafe fn RB_FLOAT_TYPE_P(obj: VALUE) -> bool {
    if FLONUM_P(obj) {
        return true;
    } else if SPECIAL_CONST_P(obj) {
        return false;
    } else {
        return RB_BUILTIN_TYPE(obj) == RUBY_T_FLOAT;
    }
}
