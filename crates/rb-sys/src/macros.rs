//! Implementation of Ruby macros.
//!
//! Since macros are rely on the C preprocessor, or defined as `inline` C
//! functions, they are not available when linking libruby. In order to use the
//! libruby macros from Rust, `rb-sys` implements them using the following
//! strategies:
//!
//! 1. Some macros are implemented in Rust, as inline functions. Using these
//!    does not require compiling C code, and can be used in Rust code without the
//!    `ruby-macros` feature.
//! 2. The rest are implemented in C code  that exports the macros as functions
//!    that can be used in Rust. This requires the `ruby-macros` feature.

#![allow(rustdoc::broken_intra_doc_links)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use crate::ruby_value_type;
use crate::stable_api::StableApi;
use crate::{stable_api::StableApiDefinition, VALUE};
use std::os::raw::{c_char, c_long};

/// Emulates Ruby's "if" statement.
///
/// - @param[in]  obj    An arbitrary ruby object.
/// - @retval     false  `obj` is either ::RUBY_Qfalse or ::RUBY_Qnil.
/// - @retval     true   Anything else.
///
/// ```
/// use rb_sys::special_consts::*;
///
/// assert!(!TEST(Qfalse));
/// assert!(!TEST(Qnil));
/// assert!(TEST(Qtrue));
/// ```
#[inline(always)]
pub fn TEST<T: Into<VALUE>>(obj: T) -> bool {
    StableApi::rb_test(obj.into())
}

/// Checks if the given object is nil.
///
/// - @param[in]  obj    An arbitrary ruby object.
/// - @retval     true   `obj` is ::RUBY_Qnil.
/// - @retval     false  Anything else.
///
/// ### Example
///
/// ```
/// use rb_sys::special_consts::*;
///
/// assert!(NIL_P(Qnil));
/// assert!(!NIL_P(Qtrue));
/// ```
#[inline(always)]
pub fn NIL_P<T: Into<VALUE>>(obj: T) -> bool {
    StableApi::nil_p(obj.into())
}

/// Checks if the given object is a so-called Fixnum.
///
/// - @param[in]  obj    An arbitrary ruby object.
/// - @retval     true   `obj` is a Fixnum.
/// - @retval     false  Anything else.
/// - @note       Fixnum was  a thing  in the  20th century, but  it is  rather an
///             implementation detail today.
#[inline(always)]
pub fn FIXNUM_P<T: Into<VALUE>>(obj: T) -> bool {
    StableApi::fixnum_p(obj.into())
}

/// Checks if the given object is a static symbol.
///
/// - @param[in]  obj    An arbitrary ruby object.
/// - @retval     true   `obj` is a static symbol
/// - @retval     false  Anything else.
/// - @see        RB_DYNAMIC_SYM_P()
/// - @see        RB_SYMBOL_P()
/// - @note       These days  there are static  and dynamic symbols, just  like we
///             once had Fixnum/Bignum back in the old days.
pub fn STATIC_SYM_P<T: Into<VALUE>>(obj: T) -> bool {
    StableApi::static_sym_p(obj.into())
}

/// Get the backend storage of a Ruby array.
///
/// ### Safety
///
/// This function is unsafe because it dereferences a raw pointer and returns
/// raw pointers to Ruby memory. The caller must ensure that the pointer stays live
/// for the duration of usage the the underlying array (by either GC marking or
/// keeping the RArray on the stack).
///
/// - @param[in]  a  An object of ::RArray.
/// - @return     Its backend storage.
#[inline(always)]
pub unsafe fn RARRAY_CONST_PTR<T: Into<VALUE>>(obj: T) -> *const VALUE {
    StableApi::rarray_const_ptr(obj.into())
}

/// Get the length of a Ruby array.
///
/// ### Safety
///
/// This function is unsafe because it dereferences a raw pointer in order to
/// access internal Ruby memory.
///
/// - @param[in]  a  An object of ::RArray.
/// - @return     Its length.
#[inline(always)]
pub unsafe fn RARRAY_LEN<T: Into<VALUE>>(obj: T) -> c_long {
    StableApi::rarray_len(obj.into())
}

/// Get the length of a Ruby string.
///
/// ### Safety
///
/// This function is unsafe because it dereferences a raw pointer in order to
/// access internal Ruby memory.
///
/// - @param[in]  a  An object of ::RString.
/// - @return     Its length.
#[inline(always)]
pub unsafe fn RSTRING_LEN<T: Into<VALUE>>(obj: T) -> c_long {
    StableApi::rstring_len(obj.into())
}

/// Get the backend storage of a Ruby string.
///
/// ### Safety
///
/// This function is unsafe because it dereferences a raw pointer and returns
/// raw pointers to Ruby memory.
///
/// - @param[in]  a  An object of ::RString.
/// - @return     Its backend storage
#[inline(always)]
pub unsafe fn RSTRING_PTR<T: Into<VALUE>>(obj: T) -> *const c_char {
    StableApi::rstring_ptr(obj.into())
}

/// Checks if the given object is a so-called Flonum.
///
/// @param[in]  obj    An arbitrary ruby object.
/// @retval     true   `obj` is a Flonum.
/// @retval     false  Anything else.
/// @see        RB_FLOAT_TYPE_P()
/// @note       These days there are Flonums and non-Flonum floats, just like we
///             once had Fixnum/Bignum back in the old days.
#[inline(always)]
pub fn FLONUM_P<T: Into<VALUE>>(#[allow(unused)] obj: T) -> bool {
    StableApi::flonum_p(obj.into())
}

/// Checks if  the given  object is  an immediate  i.e. an  object which  has no
/// corresponding storage inside of the object space.
///
/// @param[in]  obj    An arbitrary ruby object.
/// @retval     true   `obj` is a Flonum.
/// @retval     false  Anything else.
/// @see        RB_FLOAT_TYPE_P()
/// @note       The concept of "immediate" is purely C specific.
#[inline(always)]
pub fn IMMEDIATE_P<T: Into<VALUE>>(obj: T) -> bool {
    StableApi::immediate_p(obj.into())
}

/// Checks if the given object is of enum ::ruby_special_consts.
///
/// @param[in]  obj    An arbitrary ruby object.
/// @retval     true   `obj` is a special constant.
/// @retval     false  Anything else.
///
/// ### Example
///
/// ```
/// use rb_sys::special_consts::*;
///
/// assert!(SPECIAL_CONST_P(Qnil));
/// assert!(SPECIAL_CONST_P(Qtrue));
/// assert!(SPECIAL_CONST_P(Qfalse));
/// ```
#[inline(always)]
pub fn SPECIAL_CONST_P<T: Into<VALUE>>(obj: T) -> bool {
    StableApi::special_const_p(obj.into())
}

/// Queries the type of the object.
///
/// @param[in]  obj  Object in question.
/// @pre        `obj` must not be a special constant.
/// @return     The type of `obj`.
///
/// # Safety
/// This function is unsafe because it could dereference a raw pointer when
/// attemping to access the underlying [`RBasic`] struct.
#[inline(always)]
pub unsafe fn RB_BUILTIN_TYPE(obj: VALUE) -> ruby_value_type {
    StableApi::builtin_type(obj)
}

/// Queries if the object is an instance of ::rb_cInteger.
///
/// @param[in]  obj    Object in question.
/// @retval     true   It is.
/// @retval     false  It isn't.
///
/// # Safety
/// This function is unsafe because it could dereference a raw pointer when
/// attemping to access the underlying [`RBasic`] struct.
#[inline(always)]
pub unsafe fn RB_INTEGER_TYPE_P(obj: VALUE) -> bool {
    StableApi::integer_type_p(obj)
}

/// Queries if the object is a dynamic symbol.
///
/// @param[in]  obj    Object in question.
/// @retval     true   It is.
/// @retval     false  It isn't.
///
/// # Safety
/// This function is unsafe because it could dereference a raw pointer when
/// attemping to access the underlying [`RBasic`] struct.
#[inline(always)]
pub unsafe fn RB_DYNAMIC_SYM_P(obj: VALUE) -> bool {
    StableApi::dynamic_sym_p(obj)
}

/// Queries if the object is an instance of ::rb_cSymbol.
///
/// @param[in]  obj    Object in question.
/// @retval     true   It is.
/// @retval     false  It isn't.
///
/// # Safety
/// This function is unsafe because it could dereference a raw pointer when
/// attemping to access the underlying [`RBasic`] struct.
#[inline(always)]
pub unsafe fn RB_SYMBOL_P(obj: VALUE) -> bool {
    StableApi::symbol_p(obj)
}

/// Identical to RB_BUILTIN_TYPE(), except it can also accept special constants.
///
/// @param[in]  obj  Object in question.
/// @return     The type of `obj`.
///
/// # Safety
/// This function is unsafe because it could dereference a raw pointer when
/// attemping to access the underlying [`RBasic`] struct.
#[inline(always)]
pub unsafe fn RB_TYPE(value: VALUE) -> ruby_value_type {
    StableApi::rb_type(value)
}

/// Queries if the given object is of given type.
///
/// @param[in]  obj    An object.
/// @param[in]  t      A type.
/// @retval     true   `obj` is of type `t`.
/// @retval     false  Otherwise.
///
/// # Safety
/// This function is unsafe because it could dereference a raw pointer when
/// attemping to access the underlying [`RBasic`] struct.
pub unsafe fn RB_TYPE_P(obj: VALUE, ty: ruby_value_type) -> bool {
    StableApi::type_p(obj, ty)
}

/// Queries if the object is an instance of ::rb_cFloat.
///
/// @param[in]  obj    Object in question.
/// @retval     true   It is.
/// @retval     false  It isn't.
///
/// # Safety
/// This function is unsafe because it could dereference a raw pointer when
/// attemping to access the underlying [`RBasic`] struct.
pub unsafe fn RB_FLOAT_TYPE_P(obj: VALUE) -> bool {
    StableApi::float_type_p(obj)
}
