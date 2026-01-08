//! Implementation of Ruby macros.
//!
//! Since macros are rely on the C preprocessor, or defined as `inline` C
//! functions, they are not available when linking libruby. In order to use the
//! libruby macros from Rust, `rb-sys` implements them using the following
//! strategies:
//!
//! 1. For stable versions of Ruby, the macros are implemented as Rust functions
//! 2. For ruby-head, the macros are implemented as C functions that are linked
//!    into the crate.

#![allow(rustdoc::broken_intra_doc_links)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use crate::rb_data_type_t;
use crate::ruby_value_type;
use crate::stable_api::get_default as api;
use crate::StableApiDefinition;
use crate::VALUE;
use std::ffi::c_void;
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
pub fn TEST(obj: VALUE) -> bool {
    api().rb_test(obj)
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
pub fn NIL_P(obj: VALUE) -> bool {
    api().nil_p(obj)
}

/// Checks if the given object is a so-called Fixnum.
///
/// - @param[in]  obj    An arbitrary ruby object.
/// - @retval     true   `obj` is a Fixnum.
/// - @retval     false  Anything else.
/// - @note       Fixnum was  a thing  in the  20th century, but  it is  rather an implementation detail today.
#[inline(always)]
pub fn FIXNUM_P(obj: VALUE) -> bool {
    api().fixnum_p(obj)
}

/// Checks if the given object is a static symbol.
///
/// - @param[in]  obj    An arbitrary ruby object.
/// - @retval     true   `obj` is a static symbol
/// - @retval     false  Anything else.
/// - @see        RB_DYNAMIC_SYM_P()
/// - @see        RB_SYMBOL_P()
/// - @note       These days  there are static  and dynamic symbols, just  like we once had Fixnum/Bignum back in the old days.
#[inline(always)]
pub fn STATIC_SYM_P(obj: VALUE) -> bool {
    api().static_sym_p(obj)
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
pub unsafe fn RARRAY_CONST_PTR(obj: VALUE) -> *const VALUE {
    api().rarray_const_ptr(obj)
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
pub unsafe fn RARRAY_LEN(obj: VALUE) -> c_long {
    api().rarray_len(obj)
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
pub unsafe fn RSTRING_LEN(obj: VALUE) -> c_long {
    api().rstring_len(obj)
}

/// Get the backend storage of a Ruby string.
///
/// ### Safety
///
/// This function is unsafe because it dereferences a raw pointer and returns
/// raw pointers to Ruby memory. The caller must ensure that the pointer stays live
/// for the duration of usage the the underlying array (by either GC marking or
/// keeping the RArray on the stack).
///
/// - @param[in]  a  An object of ::RString.
/// - @return     Its backend storage
#[inline(always)]
pub unsafe fn RSTRING_PTR(obj: VALUE) -> *const c_char {
    api().rstring_ptr(obj)
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
pub fn FLONUM_P(obj: VALUE) -> bool {
    api().flonum_p(obj)
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
pub fn IMMEDIATE_P(obj: VALUE) -> bool {
    api().immediate_p(obj)
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
pub fn SPECIAL_CONST_P(obj: VALUE) -> bool {
    api().special_const_p(obj)
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
    api().builtin_type(obj)
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
    api().integer_type_p(obj)
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
    api().dynamic_sym_p(obj)
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
    api().symbol_p(obj)
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
    api().rb_type(value)
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
#[inline(always)]
#[cfg(ruby_engine = "mri")] // truffleruby provides its own implementation
pub unsafe fn RB_TYPE_P(obj: VALUE, ty: ruby_value_type) -> bool {
    api().type_p(obj, ty)
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
#[inline(always)]
pub unsafe fn RB_FLOAT_TYPE_P(obj: VALUE) -> bool {
    api().float_type_p(obj)
}

/// Checks if the given object is an RTypedData.
///
/// @param[in]  obj    Object in question.
/// @retval     true   It is an RTypedData.
/// @retval     false  It isn't an RTypedData.
///
/// # Safety
/// This function is unsafe because it could dereference a raw pointer when
/// accessing the underlying data structure.
#[inline(always)]
pub unsafe fn RTYPEDDATA_P(obj: VALUE) -> bool {
    api().rtypeddata_p(obj)
}

/// Gets the data type information from an RTypedData object.
///
/// @param[in]  obj    An RTypedData object.
/// @return     Pointer to the rb_data_type_t structure for this object.
///
/// # Safety
/// This function is unsafe because it dereferences a raw pointer to get
/// access to the underlying data type. The caller must ensure the object
/// is a valid RTypedData.
#[inline(always)]
pub unsafe fn RTYPEDDATA_TYPE(obj: VALUE) -> *const rb_data_type_t {
    api().rtypeddata_type(obj)
}

/// Gets the data pointer from an RTypedData object.
///
/// @param[in]  obj    An RTypedData object.
/// @return     Pointer to the wrapped C struct.
///
/// # Safety
/// This function is unsafe because it dereferences a raw pointer to get
/// access to the underlying data. The caller must ensure the object
/// is a valid RTypedData.
#[inline(always)]
pub unsafe fn RTYPEDDATA_GET_DATA(obj: VALUE) -> *mut c_void {
    api().rtypeddata_get_data(obj)
}

/// Checks if the bignum is positive.
///
/// @param[in]  b      An object of RBignum.
/// @retval     false  `b` is less than zero.
/// @retval     true   Otherwise.
///
/// # Safety
/// This function is unsafe because it could dereference a raw pointer when
/// accessing the underlying bignum structure.
#[inline(always)]
pub unsafe fn RBIGNUM_POSITIVE_P(b: VALUE) -> bool {
    api().bignum_positive_p(b)
}

/// Checks if the bignum is negative.
///
/// @param[in]  b      An object of RBignum.
/// @retval     true   `b` is less than zero.
/// @retval     false  Otherwise.
///
/// # Safety
/// This function is unsafe because it could dereference a raw pointer when
/// accessing the underlying bignum structure.
#[inline(always)]
pub unsafe fn RBIGNUM_NEGATIVE_P(b: VALUE) -> bool {
    api().bignum_negative_p(b)
}

/// Convert ID to Symbol (akin to `ID2SYM` or `RB_ID2SYM`).
///
/// Converts an internal ID to its corresponding Symbol VALUE.
/// This is a safe operation - just bit manipulation for static symbols.
///
/// @param[in]  id     An ID value.
/// @return     The Symbol VALUE corresponding to the ID.
#[inline(always)]
pub fn ID2SYM(id: crate::ID) -> VALUE {
    api().id2sym(id)
}

/// Alias for ID2SYM for compatibility with Ruby naming conventions.
#[inline(always)]
pub fn RB_ID2SYM(id: crate::ID) -> VALUE {
    api().id2sym(id)
}

/// Convert Symbol to ID (akin to `SYM2ID` or `RB_SYM2ID`).
///
/// Converts a Symbol VALUE to its internal ID representation.
///
/// @param[in]  obj    A Symbol VALUE.
/// @return     The ID corresponding to the Symbol.
///
/// # Safety
/// - `obj` must be a valid Symbol VALUE
/// - For dynamic symbols, this may access the heap
#[inline(always)]
pub unsafe fn SYM2ID(obj: VALUE) -> crate::ID {
    api().sym2id(obj)
}

/// Alias for SYM2ID for compatibility with Ruby naming conventions.
///
/// # Safety
/// - `obj` must be a valid Symbol VALUE
/// - For dynamic symbols, this may access the heap
#[inline(always)]
pub unsafe fn RB_SYM2ID(obj: VALUE) -> crate::ID {
    api().sym2id(obj)
}

/// Convert Fixnum to long (akin to `FIX2LONG`).
///
/// Extracts the integer value from a Fixnum VALUE by performing an arithmetic right shift.
///
/// # Safety
/// - `obj` must be a valid Fixnum VALUE (checked with FIXNUM_P)
/// - Behavior is undefined if called on non-Fixnum values
#[inline(always)]
pub unsafe fn FIX2LONG(obj: VALUE) -> std::os::raw::c_long {
    api().fix2long(obj)
}

/// Convert Fixnum to unsigned long (akin to `FIX2ULONG`).
///
/// Extracts the unsigned integer value from a Fixnum VALUE.
///
/// # Safety
/// - `obj` must be a valid positive Fixnum VALUE
/// - Behavior is undefined for negative fixnums
#[inline(always)]
pub unsafe fn FIX2ULONG(obj: VALUE) -> std::os::raw::c_ulong {
    api().fix2ulong(obj)
}

/// Convert long to Fixnum (akin to `LONG2FIX`).
///
/// Creates a Fixnum VALUE from a long integer.
///
/// # Safety
/// - `val` must be within the valid Fixnum range (use FIXABLE to check)
/// - Behavior is undefined if value is out of range
#[inline(always)]
pub unsafe fn LONG2FIX(val: std::os::raw::c_long) -> VALUE {
    api().long2fix(val)
}

/// Check if long value can be represented as Fixnum (akin to `FIXABLE`).
///
/// Returns true if the value fits within the Fixnum range.
#[inline(always)]
pub fn FIXABLE(val: std::os::raw::c_long) -> bool {
    api().fixable(val)
}

/// Check if unsigned long value can be represented as positive Fixnum (akin to `POSFIXABLE`).
///
/// Returns true if the unsigned value fits within the positive Fixnum range.
#[inline(always)]
pub fn POSFIXABLE(val: std::os::raw::c_ulong) -> bool {
    api().posfixable(val)
}

/// Convert Ruby Integer to long (akin to `NUM2LONG`).
///
/// Converts any Ruby Integer (Fixnum or Bignum) to a C long.
/// May raise a RangeError exception if the value is out of range.
///
/// # Safety
/// - `obj` must be a valid Integer VALUE
/// - May call into Ruby runtime and potentially raise exceptions
/// - May trigger garbage collection
#[inline(always)]
pub unsafe fn NUM2LONG(obj: VALUE) -> std::os::raw::c_long {
    api().num2long(obj)
}

/// Convert Ruby Integer to unsigned long (akin to `NUM2ULONG`).
///
/// Converts any Ruby Integer (Fixnum or Bignum) to a C unsigned long.
/// May raise a RangeError exception if the value is out of range or negative.
///
/// # Safety
/// - `obj` must be a valid Integer VALUE
/// - May call into Ruby runtime and potentially raise exceptions
/// - May trigger garbage collection
#[inline(always)]
pub unsafe fn NUM2ULONG(obj: VALUE) -> std::os::raw::c_ulong {
    api().num2ulong(obj)
}

/// Convert long to Ruby Integer (akin to `LONG2NUM`).
///
/// Creates a Ruby Integer (Fixnum or Bignum) from a C long.
/// Uses Fixnum if possible, otherwise allocates a Bignum.
#[inline(always)]
pub fn LONG2NUM(val: std::os::raw::c_long) -> VALUE {
    api().long2num(val)
}

/// Convert unsigned long to Ruby Integer (akin to `ULONG2NUM`).
///
/// Creates a Ruby Integer (Fixnum or Bignum) from a C unsigned long.
/// Uses Fixnum if possible, otherwise allocates a Bignum.
#[inline(always)]
pub fn ULONG2NUM(val: std::os::raw::c_ulong) -> VALUE {
    api().ulong2num(val)
}
