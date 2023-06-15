#include "ruby.h"

long
rb_sys_compiled_c_impls_RSTRING_LEN(VALUE obj) {
  return RSTRING_LEN(obj);
}

char *
rb_sys_compiled_c_impls_RSTRING_PTR(VALUE obj) {
  return RSTRING_PTR(obj);
}

long
rb_sys_compiled_c_impls_RARRAY_LEN(VALUE obj) {
  return RARRAY_LEN(obj);
}

const VALUE *
rb_sys_compiled_c_impls_RARRAY_CONST_PTR(VALUE obj) {
  return RARRAY_CONST_PTR(obj);
}
