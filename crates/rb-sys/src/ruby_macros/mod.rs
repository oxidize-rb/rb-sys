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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_nil_p() {
        assert!(unsafe { NIL_P(Qnil as u64) });
    }

    #[test]
    fn test_rb_test() {
        assert!(!unsafe { RB_TEST(Qnil as u64) });
    }

    #[cfg(feature = "link-ruby")]
    #[test]
    fn test_symbol_p() {
        unsafe { ruby_init() };
        let sym = unsafe { ID2SYM(rb_intern("foo\0".as_ptr() as *const i8)) };

        assert!(unsafe { SYMBOL_P(sym) });
    }

    #[cfg(feature = "link-ruby")]
    #[test]
    fn test_integer_type_p() {
        let int = unsafe { rb_num2fix(1) };

        assert!(unsafe { RB_INTEGER_TYPE_P(int) });
    }

    #[cfg(feature = "link-ruby")]
    #[test]
    fn test_rb_float_type_p() {
        let float = unsafe { rb_float_new(1.0) };

        assert!(unsafe { RB_FLOAT_TYPE_P(float) });
    }
}
