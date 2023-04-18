#include "ruby.h"
#include "stdbool.h"

VALUE
ruby_macros_ID2SYM(ID obj)
{
  return ID2SYM(obj);
}

ID ruby_macros_SYM2ID(VALUE obj)
{
  return SYM2ID(obj);
}

char *
ruby_macros_RSTRING_PTR(VALUE obj)
{
  return RSTRING_PTR(obj);
}

long ruby_macros_RSTRING_LEN(VALUE obj)
{
  return RSTRING_LEN(obj);
}

const VALUE *
ruby_macros_RARRAY_PTR(VALUE ary)
{
  return RARRAY_PTR(ary);
}
