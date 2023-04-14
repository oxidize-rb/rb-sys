#![allow(rustdoc::broken_intra_doc_links)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
//! Definitions for the compiled Ruby macros.
//!
//! Since macros are rely on the C preprocessor, they are not automatically
//! available to Rust. This module compiles a tiny snippet of C code that is
//! used to generate the Ruby macros, so they can be used in Rust.

#[cfg(ruby_gte_3_0)]
use crate::ruby_rarray_consts::RARRAY_EMBED_LEN_SHIFT;
#[cfg(all(ruby_lt_3_0, ruby_gt_2_4))]
use crate::ruby_rarray_flags::RARRAY_EMBED_LEN_SHIFT;
#[cfg(ruby_lte_2_4)]
use crate::RARRAY_EMBED_LEN_SHIFT;

#[cfg(ruby_gt_2_4)]
use crate::ruby_rarray_flags::{RARRAY_EMBED_FLAG, RARRAY_EMBED_LEN_MASK};
#[cfg(ruby_lte_2_4)]
use crate::{RARRAY_EMBED_FLAG, RARRAY_EMBED_LEN_MASK};

use crate::{
    value_type, Qnil, FIXNUM_FLAG, FLONUM_FLAG, FLONUM_MASK, IMMEDIATE_MASK, RB_TYPE_P,
    SYMBOL_FLAG, VALUE,
};

/// Emulates Ruby's "if" statement.
///
/// @param[in]  obj    An arbitrary ruby object.
/// @retval     false  `obj` is either ::RUBY_Qfalse or ::RUBY_Qnil.
/// @retval     true   Anything else.
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
    (obj.into() & !(Qnil as VALUE)) != 0
}

/// Checks if the given object is nil.
///
/// @param[in]  obj    An arbitrary ruby object.
/// @retval     true   `obj` is ::RUBY_Qnil.
/// @retval     false  Anything else.
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
    obj.into() == (Qnil as VALUE)
}

/// Checks if the given object is a so-called Fixnum.
///
/// @param[in]  obj    An arbitrary ruby object.
/// @retval     true   `obj` is a Fixnum.
/// @retval     false  Anything else.
/// @note       Fixnum was  a thing  in the  20th century, but  it is  rather an
///             implementation detail today.
#[inline(always)]
pub fn FIXNUM_P<T: Into<VALUE>>(obj: T) -> bool {
    (obj.into() & FIXNUM_FLAG as VALUE) != 0
}

/// Checks if the given object is a static symbol.
///
/// @param[in]  obj    An arbitrary ruby object.
/// @retval     true   `obj` is a static symbol
/// @retval     false  Anything else.
/// @see        RB_DYNAMIC_SYM_P()
/// @see        RB_SYMBOL_P()
/// @note       These days  there are static  and dynamic symbols, just  like we
///             once had Fixnum/Bignum back in the old days.
pub fn STATIC_SYM_P<T: Into<VALUE>>(obj: T) -> bool {
    (obj.into() & 0xff) == SYMBOL_FLAG as VALUE
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
/// @param[in]  a  An object of ::RArray.
/// @return     Its backend storage.
#[inline(always)]
pub unsafe fn RARRAY_CONST_PTR<T: Into<VALUE>>(obj: T) -> *const VALUE {
    let value: VALUE = obj.into();

    assert!(RB_TYPE_P(value) == value_type::RUBY_T_ARRAY);

    let rbasic = &*(value as *const crate::RBasic);
    let rarray = &*(value as *const crate::RArray);

    if (rbasic.flags & RARRAY_EMBED_FLAG as VALUE) != 0 {
        rarray.as_.ary.as_ptr()
    } else {
        rarray.as_.heap.ptr
    }
}

/// Get the length of a Ruby array.
///
/// ### Safety
///
/// This function is unsafe because it dereferences a raw pointer in order to
/// access internal Ruby memory.
///
/// @param[in]  a  An object of ::RArray.
/// @return     Its length.
#[inline(always)]
pub unsafe fn RARRAY_LEN<T: Into<VALUE>>(obj: T) -> isize {
    let value: VALUE = obj.into();

    assert!(RB_TYPE_P(value) == value_type::RUBY_T_ARRAY);

    let rbasic = &*(value as *const crate::RBasic);
    let rarray = &*(value as *const crate::RArray);

    if (rbasic.flags & RARRAY_EMBED_FLAG as VALUE) != 0 {
        let len = (rbasic.flags >> RARRAY_EMBED_LEN_SHIFT as VALUE)
            & (RARRAY_EMBED_LEN_MASK as VALUE >> RARRAY_EMBED_LEN_SHIFT as VALUE);

        len as _
    } else {
        rarray.as_.heap.len as _
    }
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
pub fn FLONUM_P<T: Into<VALUE>>(obj: T) -> bool {
    #[cfg(ruby_use_flonum = "true")]
    let ret = {
        let obj = obj.into();
        (obj & (FLONUM_MASK as VALUE)) == FLONUM_FLAG as VALUE
    };

    #[cfg(not(ruby_use_flonum = "true"))]
    let ret = false;

    ret
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
    obj.into() & (IMMEDIATE_MASK as VALUE) != 0
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
    let value: VALUE = obj.into();
    let is_immediate = value & (IMMEDIATE_MASK as VALUE) != 0;
    let test = (value & !(Qnil as VALUE)) != 0;

    is_immediate || !test
}

#[cfg(feature = "ruby-macros")]
use crate::ID;
#[cfg(feature = "ruby-macros")]
use std::os::raw::{c_char, c_long};
#[cfg(feature = "ruby-macros")]
extern "C" {

    /// Allocates an instance of ::rb_cSymbol that has the given id.
    ///
    /// @param[in]  id           An id.
    /// @retval     Qfalse  No such id ever existed in the history.
    /// @retval     Otherwise    An allocated ::rb_cSymbol instance.
    #[link_name = "ruby_macros_ID2SYM"]
    pub fn ID2SYM(obj: ID) -> VALUE;

    /// Converts an instance of ::rb_cSymbol into an ::ID.
    ///
    /// @param[in]  obj            An instance of ::rb_cSymbol.
    /// @exception  rb_eTypeError  `obj` is not an instance of ::rb_cSymbol.
    /// @return     An ::ID of the identical symbol.
    #[link_name = "ruby_macros_SYM2ID"]
    pub fn SYM2ID(obj: ID) -> VALUE;

    /// Queries the contents pointer of the string.
    ///
    /// @param[in]  str  String in question.
    /// @return     Pointer to its contents.
    /// @pre        `str` must be an instance of ::RString.
    #[link_name = "ruby_macros_RSTRING_PTR"]
    pub fn RSTRING_PTR(obj: VALUE) -> *mut c_char;

    /// Queries the length of the string.
    ///
    /// @param[in]  str  String in question.
    /// @return     Its length, in bytes.
    /// @pre        `str` must be an instance of ::RString.
    #[link_name = "ruby_macros_RSTRING_LEN"]
    pub fn RSTRING_LEN(obj: VALUE) -> c_long;

    /// Wild  use of  a  C  pointer.  This  function  accesses  the backend  storage
    /// directly.   This is  slower  than  #RARRAY_PTR_USE_TRANSIENT.  It  exercises
    /// extra manoeuvres  to protect our generational  GC.  Use of this  function is
    /// considered archaic.  Use a modern way instead.

    /// @param[in]  ary  An object of ::RArray.
    /// @return     The backend C array.

    /// @internal

    /// That said...  there are  extension libraries  in the wild  who uses  it.  We
    /// cannot but continue supporting.
    #[link_name = "ruby_macros_RARRAY_PTR"]
    pub fn RARRAY_PTR(a: VALUE) -> *const VALUE;
}
