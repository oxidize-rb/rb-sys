use crate::{ruby_value_type, ID, VALUE};

extern "C" {

    /// Queries if the given object is of given type.
    ///
    /// @param[in]  obj    An object.
    /// @param[in]  t      A type.
    /// @retval     true   `obj` is of type `t`.
    /// @retval     false  Otherwise.
    ///
    /// @internal
    ///
    /// This  function is  a super-duper  hot  path.  Optimised  targeting modern  C
    /// compilers and x86_64 architecture.
    #[link_name = "ruby_macros_RB_TYPE_P"]
    pub fn RB_TYPE_P(obj: VALUE, t: ruby_value_type) -> bool;

    /// Queries if the object is an instance of ::ruby_macros_cInteger.
    ///
    /// @param[in]  obj    Object in question.
    /// @retval     true   It is.
    /// @retval     false  It isn't.
    #[link_name = "ruby_macros_RB_INTEGER_TYPE_P"]
    pub fn RB_INTEGER_TYPE_P(obj: VALUE) -> bool;

    /// Queries if the object is an instance of ::ruby_macros_cFloat.
    ///
    /// @param[in]  obj    Object in question.
    /// @retval     true   It is.
    /// @retval     false  It isn't.
    #[link_name = "ruby_macros_RB_FLOAT_TYPE_P"]
    pub fn RB_FLOAT_TYPE_P(obj: VALUE) -> bool;

    /// Queries if the object is an instance of ::ruby_macros_cSymbol.
    ///
    /// @param[in]  obj    Object in question.
    /// @retval     true   It is.
    /// @retval     false  It isn't.
    #[link_name = "ruby_macros_SYMBOL_P"]
    pub fn SYMBOL_P(obj: VALUE) -> bool;

    /// Checks if the given object is nil.
    ///
    /// @param[in]  obj    An arbitrary ruby object.
    /// @retval     true   `obj` is ::Qnil.
    /// @retval     false  Anything else.
    #[link_name = "ruby_macros_NIL_P"]
    pub fn NIL_P(obj: VALUE) -> bool;

    /// Emulates Ruby's "if" statement.
    ///
    /// @param[in]  obj    An arbitrary ruby object.
    /// @retval     false  `obj` is either ::Qfalse or ::Qnil.
    /// @retval     true   Anything else.
    ///
    /// @internal
    ///
    /// It HAS to be `__attribute__((const))` in  order for clang to properly deduce
    /// `__builtin_assume()`.
    #[link_name = "ruby_macros_RB_TEST"]
    pub fn RB_TEST(obj: VALUE) -> bool;

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
    pub fn RSTRING_PTR(obj: VALUE) -> *mut ::libc::c_char;

    /// Queries the length of the string.
    ///
    /// @param[in]  str  String in question.
    /// @return     Its length, in bytes.
    /// @pre        `str` must be an instance of ::RString.
    #[link_name = "ruby_macros_RSTRING_LEN"]
    pub fn RSTRING_LEN(obj: VALUE) -> ::libc::c_long;

    /// Queries the length of the array.
    ///
    /// @param[in]  a  Array in question.
    /// @return     Its number of elements.
    /// @pre        `a` must be an instance of ::RArray.
    #[link_name = "ruby_macros_RARRAY_LEN"]
    pub fn RARRAY_LEN(a: VALUE) -> ::libc::c_long;
}
