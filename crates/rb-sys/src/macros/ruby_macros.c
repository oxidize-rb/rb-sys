#include "ruby.h"
#include "stdbool.h"

bool
ruby_macros_RB_TYPE_P(VALUE obj, enum ruby_value_type t) {
  return RB_TYPE_P(obj, (int)t);
};

bool
ruby_macros_RB_INTEGER_TYPE_P(VALUE obj) {
  return RB_INTEGER_TYPE_P(obj);
}

bool
ruby_macros_SYMBOL_P(VALUE obj) {
  return SYMBOL_P(obj);
}

bool
ruby_macros_RB_FLOAT_TYPE_P(VALUE obj) {
  return RB_FLOAT_TYPE_P(obj);
}

bool
ruby_macros_NIL_P(VALUE obj) {
  return RB_NIL_P(obj);
}

bool
ruby_macros_RB_TEST(VALUE obj) {
  return RB_TEST(obj);
}

VALUE
ruby_macros_ID2SYM(ID obj) {
  return ID2SYM(obj);
}

ID
ruby_macros_SYM2ID(VALUE obj) {
  return SYM2ID(obj);
}

char *
ruby_macros_RSTRING_PTR(VALUE obj) {
  return RSTRING_PTR(obj);
}

long
ruby_macros_RSTRING_LEN(VALUE obj) {
  return RSTRING_LEN(obj);
}

long
ruby_macros_RARRAY_LEN(VALUE obj) {
  return RARRAY_LEN(obj);
}