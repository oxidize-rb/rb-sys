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
