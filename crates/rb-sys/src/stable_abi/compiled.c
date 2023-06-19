#include "ruby.h"

long
impl_rstring_len(VALUE obj) {
  return RSTRING_LEN(obj);
}

char *
impl_rstring_ptr(VALUE obj) {
  return RSTRING_PTR(obj);
}

long
impl_rarray_len(VALUE obj) {
  return RARRAY_LEN(obj);
}

const VALUE *
impl_rarray_const_ptr(VALUE obj) {
  return RARRAY_CONST_PTR(obj);
}

int
impl_special_const_p(VALUE obj) {
  return SPECIAL_CONST_P(obj);
}

enum ruby_value_type
impl_rb_builtin_type(VALUE obj) {
  return RB_BUILTIN_TYPE(obj);
}

int
impl_rb_nil_p(VALUE obj) {
  return NIL_P(obj);
}

int
impl_rb_fixnum_p(VALUE obj) {
  return FIXNUM_P(obj);
}

int
impl_rb_static_sym_p(VALUE obj) {
  return STATIC_SYM_P(obj);
}
