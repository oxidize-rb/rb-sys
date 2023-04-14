#![allow(rustdoc::broken_intra_doc_links)]
//! Definitions for Ruby's special constants.
//!
//! Makes it easier to reference important Ruby constants, without havign to dig
//! around in bindgen's output.

#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use std::ffi::c_long;

#[cfg(ruby_gte_3_0)]
use crate::ruby_rarray_consts::RARRAY_EMBED_LEN_SHIFT;
#[cfg(ruby_lt_3_0)]
use crate::ruby_rarray_flags::RARRAY_EMBED_LEN_SHIFT;
use crate::ruby_rarray_flags::{RARRAY_EMBED_FLAG, RARRAY_EMBED_LEN_MASK};
use crate::{ruby_special_consts, value_type, RB_TYPE_P, VALUE};

pub const Qfalse: ruby_special_consts = ruby_special_consts::RUBY_Qfalse;
pub const Qtrue: ruby_special_consts = ruby_special_consts::RUBY_Qtrue;
pub const Qnil: ruby_special_consts = ruby_special_consts::RUBY_Qnil;
pub const Qundef: ruby_special_consts = ruby_special_consts::RUBY_Qundef;
pub const IMMEDIATE_MASK: ruby_special_consts = ruby_special_consts::RUBY_IMMEDIATE_MASK;
pub const FIXNUM_FLAG: ruby_special_consts = ruby_special_consts::RUBY_FIXNUM_FLAG;
pub const FLONUM_MASK: ruby_special_consts = ruby_special_consts::RUBY_FLONUM_MASK;
pub const FLONUM_FLAG: ruby_special_consts = ruby_special_consts::RUBY_FLONUM_FLAG;
pub const SYMBOL_FLAG: ruby_special_consts = ruby_special_consts::RUBY_SYMBOL_FLAG;

#[allow(clippy::from_over_into)]
impl Into<VALUE> for ruby_special_consts {
    fn into(self) -> VALUE {
        self as VALUE
    }
}

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

    debug_assert!(RB_TYPE_P(value) == value_type::RUBY_T_ARRAY);

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
pub unsafe fn RARRAY_LEN<T: Into<VALUE>>(obj: T) -> c_long {
    let value: VALUE = obj.into();

    debug_assert!(RB_TYPE_P(value) == value_type::RUBY_T_ARRAY);

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
    (obj.into() & (FLONUM_MASK as VALUE)) == FLONUM_FLAG as VALUE
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
