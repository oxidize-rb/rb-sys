#include "ruby.h"

long rb_sys_compiled_c_impls_RSTRING_LEN(VALUE obj) {
  long return_value;
  return_value = RSTRING_LEN(obj);
  return return_value;
}

char *rb_sys_compiled_c_impls_RSTRING_PTR(VALUE obj) {
  char *return_value;
  return_value = RSTRING_PTR(obj);
  return return_value;
}

long rb_sys_compiled_c_impls_RARRAY_LEN(VALUE obj) {
  long return_value;
  return_value = RARRAY_LEN(obj);
  return return_value;
}

const VALUE *rb_sys_compiled_c_impls_RARRAY_CONST_PTR(VALUE obj) {
  const VALUE *return_value;
  return_value = RARRAY_CONST_PTR(obj);
  return return_value;
}
