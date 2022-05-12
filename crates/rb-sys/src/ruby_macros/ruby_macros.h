#include "ruby.h"

/**
 * Queries if the given object is of given type.
 *
 * @param[in]  obj    An object.
 * @param[in]  t      A type.
 * @retval     true   `obj` is of type `t`.
 * @retval     false  Otherwise.
 *
 * @internal
 *
 * This  function is  a super-duper  hot  path.  Optimised  targeting modern  C
 * compilers and x86_64 architecture.
 */
bool ruby_macros_RB_TYPE_P(VALUE obj, enum ruby_value_type t);

/**
 * Queries if the object is an instance of ::ruby_macros_cInteger.
 *
 * @param[in]  obj    Object in question.
 * @retval     true   It is.
 * @retval     false  It isn't.
 */
bool ruby_macros_RB_INTEGER_TYPE_P(VALUE obj);        /* like RB_TYPE_P(obj, T_FIXNUM) */

/**
 * Queries if the object is an instance of ::ruby_macros_cFloat.
 *
 * @param[in]  obj    Object in question.
 * @retval     true   It is.
 * @retval     false  It isn't.
 */
bool ruby_macros_RB_FLOAT_TYPE_P(VALUE obj); /* like RB_TYPE_P(obj, T_FLOAT) */

/**
 * Queries if the object is an instance of ::ruby_macros_cSymbol.
 *
 * @param[in]  obj    Object in question.
 * @retval     true   It is.
 * @retval     false  It isn't.
 */
bool ruby_macros_SYMBOL_P(VALUE obj);        /* like RB_TYPE_P(obj, T_SYMBOL) */

/**
 * Checks if the given object is nil.
 *
 * @param[in]  obj    An arbitrary ruby object.
 * @retval     true   `obj` is ::RUBY_Qnil.
 * @retval     false  Anything else.
 */
bool ruby_macros_NIL_P(VALUE obj);           /* like RB_TYPE_P(obj, T_NIL) */

/**
 * Emulates Ruby's "if" statement.
 *
 * @param[in]  obj    An arbitrary ruby object.
 * @retval     false  `obj` is either ::RUBY_Qfalse or ::RUBY_Qnil.
 * @retval     true   Anything else.
 *
 * @internal
 *
 * It HAS to be `__attribute__((const))` in  order for clang to properly deduce
 * `__builtin_assume()`.
 */
bool ruby_macros_RB_TEST(VALUE obj); 

/**
 * Allocates an instance of ::rb_cSymbol that has the given id.
 *
 * @param[in]  id           An id.
 * @retval     RUBY_Qfalse  No such id ever existed in the history.
 * @retval     Otherwise    An allocated ::rb_cSymbol instance.
 */
VALUE ruby_macros_ID2SYM(ID obj); 

/**
 * Converts an instance of ::rb_cSymbol into an ::ID.
 *
 * @param[in]  obj            An instance of ::rb_cSymbol.
 * @exception  rb_eTypeError  `obj` is not an instance of ::rb_cSymbol.
 * @return     An ::ID of the identical symbol.
 */
ID ruby_macros_SYM2ID(VALUE obj); 