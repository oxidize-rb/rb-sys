//! Definitions for the compiled Ruby macros.
//!
//! Since macros are rely on the C preprocessor, they are not automatically
//! available to Rust. This module compiles a tiny snippet of C code that is
//! used to generate the Ruby macros, so they can be used in Rust.

use std::os::raw::{c_char, c_long};

use crate::{ID, VALUE};

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

    /// Queries the length of the array.
    ///
    /// @param[in]  a  Array in question.
    /// @return     Its number of elements.
    /// @pre        `a` must be an instance of ::RArray.
    #[link_name = "ruby_macros_RARRAY_LEN"]
    pub fn RARRAY_LEN(a: VALUE) -> c_long;

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
