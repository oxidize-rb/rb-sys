#include "ruby.h"

#define COMPILED_C_IMPL(name, ret) \
  ret rb_sys_compiled_c_impls_##name(VALUE obj) { \
    return name(obj); \
  }

COMPILED_C_IMPL(RSTRING_LEN, long)
COMPILED_C_IMPL(RSTRING_PTR, char *)
COMPILED_C_IMPL(RARRAY_LEN, long)
COMPILED_C_IMPL(RARRAY_CONST_PTR, const VALUE *)
