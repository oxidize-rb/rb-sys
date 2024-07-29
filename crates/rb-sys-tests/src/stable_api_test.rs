use std::ffi::{c_int, c_long};

use rb_sys::{StableApiDefinition, RUBY_FIXNUM_MAX, RUBY_FIXNUM_MIN};
use rb_sys_test_helpers::rstring as gen_rstring;

macro_rules! parity_test {
  (name: $name:ident, func: $func:ident, data_factory: $data_factory:expr $(, expected: $expected:expr)?) => {
      #[rb_sys_test_helpers::ruby_test]
      fn $name() {
          use rb_sys::stable_api;
          let data = $data_factory;

          #[allow(unused)]
          let rust_result = unsafe { stable_api::get_default().$func(data) };
          #[allow(unused_unsafe)]
          let compiled_c_result = unsafe { stable_api::get_compiled().$func(data) };

          assert_eq!(
              compiled_c_result, rust_result,
              "compiled_c was {:?}, rust was {:?}",
              compiled_c_result, rust_result
          );

          $(
              assert_eq!($expected, rust_result);
          )?
      }
  };
}

macro_rules! ruby_eval {
  ($expr:literal $(, $arg:expr)*) => {{
      unsafe {
          let mut state = 0;
          let formatted_expr = format!($expr $(, $arg)*);
          let c_string = std::ffi::CString::new(formatted_expr).unwrap();
          let ret = rb_sys::rb_eval_string_protect(c_string.as_ptr(), &mut state as _);

          if state != 0 {
              let mut err_string = rb_sys::rb_inspect(rb_sys::rb_errinfo());
              rb_sys::rb_set_errinfo(rb_sys::Qnil as _);
              let err_string = rb_sys::rb_string_value_cstr(&mut err_string);
              let err_string = std::ffi::CStr::from_ptr(err_string);
              let err_string = err_string.to_str().unwrap();
              panic!("Ruby error: {}", err_string);
          }

          ret
      }
  }};
}

parity_test!(
    name: test_rstring_len_basic,
    func: rstring_len,
    data_factory: {
      gen_rstring!("foo")
    }
);

parity_test!(
    name: test_rstring_len_long,
    func: rstring_len,
    data_factory: {
      gen_rstring!(include_str!("../../../Cargo.lock"))
    }
);

parity_test!(
    name: test_rstring_ptr_basic,
    func: rstring_ptr,
    data_factory: {
      gen_rstring!("foo")
    }
);

parity_test!(
  name: test_rstring_ptr_evaled_basic,
  func: rstring_ptr,
  data_factory: {
    ruby_eval!("'foo'")
  }
);

parity_test!(
  name: test_rstring_len_evaled_basic,
  func: rstring_len,
  data_factory: {
    ruby_eval!("'foo'")
  }
);

parity_test!(
  name: test_rstring_len_evaled_shared,
  func: rstring_len,
  data_factory: {
    ruby_eval!("'foo' + 'bar' + ('a' * 12)")
  }
);

parity_test!(
  name: test_rstring_ptr_evaled_empty,
  func: rstring_ptr,
  data_factory: {
    ruby_eval!("''")
  }
);

parity_test!(
    name: test_rstring_ptr_long,
    func: rstring_ptr,
    data_factory: {
      gen_rstring!(include_str!("../../../Cargo.lock"))
    }
);

parity_test!(
    name: test_rarray_len_basic,
    func: rarray_len,
    data_factory: {
      let ary = unsafe { rb_sys::rb_ary_new() };
      unsafe { rb_sys::rb_ary_push(ary, gen_rstring!("foo")) };
      ary
    }
);

parity_test!(
    name: test_rarray_len_evaled_basic,
    func: rarray_len,
    data_factory: {
      ruby_eval!("[2, nil, :foo]")
    }
);

parity_test!(
    name: test_rarray_len_evaled_empty,
    func: rarray_len,
    data_factory: {
      ruby_eval!("[]")
    }
);

parity_test!(
    name: test_rarray_len_long,
    func: rarray_len,
    data_factory: {
      let ary = unsafe { rb_sys::rb_ary_new() };
      for _ in 0..1000 {
        unsafe { rb_sys::rb_ary_push(ary, rb_sys::Qnil as _) };
      }
      ary
    }
);

parity_test!(
    name: test_rarray_const_ptr_basic,
    func: rarray_const_ptr,
    data_factory: {
      let ary = unsafe { rb_sys::rb_ary_new() };
      unsafe { rb_sys::rb_ary_push(ary, gen_rstring!("foo")) };
      ary
    }
);

parity_test!(
    name: test_rarray_const_ptr_evaled_basic,
    func: rarray_const_ptr,
    data_factory: {
      ruby_eval!("[2, nil, :foo]")
    }
);

parity_test!(
    name: test_rarray_const_ptr_long,
    func: rarray_const_ptr,
    data_factory: {
      let ary = unsafe { rb_sys::rb_ary_new() };
      for _ in 0..1000 {
        unsafe { rb_sys::rb_ary_push(ary, gen_rstring!("foo")) };
      }
      ary
    }
);

parity_test!(
    name: test_special_const_p_for_bool,
    func: special_const_p,
    data_factory: {
      rb_sys::Qtrue as _
    }
);

parity_test!(
    name: test_special_const_p_for_string,
    func: special_const_p,
    data_factory: {
      gen_rstring!("foo")
    }
);

parity_test!(
    name: test_special_const_p_for_static_sym,
    func: special_const_p,
    data_factory: {
      ruby_eval!(":foo")
    }
);

parity_test!(
    name: test_special_const_p_for_symbol,
    func: special_const_p,
    data_factory: {
      ruby_eval!("'foo'.to_sym")
    }
);

parity_test!(
    name: test_builtin_type_for_string,
    func: builtin_type,
    data_factory: {
      gen_rstring!("foo")
    }
);

parity_test!(
    name: test_builtin_type_for_array,
    func: builtin_type,
    data_factory: {
      ruby_eval!("[]")
    }
);

parity_test!(
    name: test_builtin_type_for_hash,
    func: builtin_type,
    data_factory: {
      ruby_eval!("{{foo: 'bar'}}")
    }
);

parity_test!(
    name: test_builtin_type_for_file,
    func: builtin_type,
    data_factory: {
      ruby_eval!("File.open('Cargo.toml')")
    }
);

parity_test!(
    name: test_builtin_type_for_symbol,
    func: builtin_type,
    data_factory: {
      ruby_eval!("'foosymmmm'.to_sym")
    }
);

parity_test! (
    name: test_rb_nil_p_for_nil,
    func: nil_p,
    data_factory: {
      rb_sys::Qnil as _
    }
);

parity_test! (
    name: test_rb_nil_p_for_false,
    func: nil_p,
    data_factory: {
      rb_sys::Qfalse as _
    }
);

parity_test! (
    name: test_rb_nil_p_for_string,
    func: nil_p,
    data_factory: {
      gen_rstring!("foo")
    }
);

parity_test! (
    name: test_rb_fixnum_p_for_fixnum,
    func: fixnum_p,
    data_factory: {
      ruby_eval!("1")
    },
    expected: true
);

parity_test! (
    name: test_rb_fixnum_p_for_string,
    func: fixnum_p,
    data_factory: {
      gen_rstring!("foo")
    },
    expected: false
);

parity_test! (
    name: test_rb_static_sym_p_for_static_sym,
    func: static_sym_p,
    data_factory: {
      let interned = unsafe { rb_sys::rb_intern2("new_sym".as_ptr() as _, 7) };
      unsafe { rb_sys::rb_id2sym(interned) }
    },
    expected: true
);

parity_test! (
    name: test_rb_static_sym_p_for_regular_sym,
    func: static_sym_p,
    data_factory: {
      ruby_eval!("'bar'.to_sym")
    },
    expected: false
);

// flonum tests
parity_test! (
    name: test_rb_flonum_p_for_flonum,
    func: flonum_p,
    data_factory: {
      ruby_eval!("1.0")
    },
    expected: true
);

parity_test! (
    name: test_rb_flonum_p_false_for_fixnum,
    func: flonum_p,
    data_factory: {
      ruby_eval!("1")
    },
    expected: false
);

parity_test! (
    name: test_rb_test_for_true,
    func: rb_test,
    data_factory: {
      rb_sys::Qtrue as _
    },
    expected: true
);

parity_test! (
    name: test_rb_test_for_false,
    func: rb_test,
    data_factory: {
      rb_sys::Qfalse as _
    },
    expected: false
);

parity_test! (
    name: test_rb_test_for_nil,
    func: rb_test,
    data_factory: {
      rb_sys::Qnil as _
    },
    expected: false
);

parity_test! (
    name: test_rb_test_for_fixnum,
    func: rb_test,
    data_factory: {
      ruby_eval!("1")
    },
    expected: true
);

// tests for dynamic_sym_p
parity_test! (
    name: test_rb_dynamic_sym_p_for_dynamic_sym,
    func: dynamic_sym_p,
    data_factory: {
      ruby_eval!("'footestingdynsym'.to_sym")
    },
    expected: true
);

parity_test! (
    name: test_rb_dynamic_sym_p_for_static_sym,
    func: dynamic_sym_p,
    data_factory: {
      let interned = unsafe { rb_sys::rb_intern2("new_sym".as_ptr() as _, 7) };
      unsafe { rb_sys::rb_id2sym(interned) }
    },
    expected: false
);

parity_test! (
    name: test_rb_symbol_p_for_dynamic_sym,
    func: symbol_p,
    data_factory: {
      ruby_eval!("'foodyn'.to_sym")
    },
    expected: true
);

parity_test! (
    name: test_rb_symbol_p_for_static_sym,
    func: symbol_p,
    data_factory: {
      let interned = unsafe { rb_sys::rb_intern2("new_sym".as_ptr() as _, 7) };
      unsafe { rb_sys::rb_id2sym(interned) }
    },
    expected: true
);

parity_test! (
    name: test_rb_float_type_p_for_flonum,
    func: float_type_p,
    data_factory: {
      ruby_eval!("1.0")
    },
    expected: true
);

parity_test! (
    name: test_rb_float_type_p_for_fixnum,
    func: float_type_p,
    data_factory: {
      ruby_eval!("1")
    },
    expected: false
);

// tests for rb_type
parity_test! (
    name: test_rb_type_for_fixnum,
    func: rb_type,
    data_factory: {
      ruby_eval!("1")
    },
    expected: rb_sys::ruby_value_type::RUBY_T_FIXNUM
);

parity_test! (
    name: test_rb_type_for_float,
    func: rb_type,
    data_factory: {
      ruby_eval!("1.0")
    },
    expected: rb_sys::ruby_value_type::RUBY_T_FLOAT
);

parity_test! (
    name: test_rb_type_for_symbol,
    func: rb_type,
    data_factory: {
      ruby_eval!("'foo'.to_sym")
    },
    expected: rb_sys::ruby_value_type::RUBY_T_SYMBOL
);

parity_test! (
    name: test_rb_type_for_string,
    func: rb_type,
    data_factory: {
      gen_rstring!("foo")
    },
    expected: rb_sys::ruby_value_type::RUBY_T_STRING
);

parity_test! (
    name: test_rb_type_for_array,
    func: rb_type,
    data_factory: {
      ruby_eval!("[]")
    },
    expected: rb_sys::ruby_value_type::RUBY_T_ARRAY
);

parity_test! (
    name: test_rb_type_for_hash,
    func: rb_type,
    data_factory: {
      ruby_eval!("{{foo: 'bar'}}")
    },
    expected: rb_sys::ruby_value_type::RUBY_T_HASH
);

parity_test! (
    name: test_rb_type_for_file,
    func: rb_type,
    data_factory: {
      ruby_eval!("File.open('Cargo.toml')")
    },
    expected: rb_sys::ruby_value_type::RUBY_T_FILE
);

parity_test! (
    name: test_rb_type_for_nil,
    func: rb_type,
    data_factory: {
      rb_sys::Qnil as _
    },
    expected: rb_sys::ruby_value_type::RUBY_T_NIL
);

parity_test! (
    name: test_rb_type_for_true,
    func: rb_type,
    data_factory: {
      rb_sys::Qtrue as _
    },
    expected: rb_sys::ruby_value_type::RUBY_T_TRUE
);

// tests for integer_type_p (include bigint too)
parity_test! (
    name: test_rb_integer_type_p_for_fixnum,
    func: integer_type_p,
    data_factory: {
      ruby_eval!("1")
    },
    expected: true
);

parity_test! (
    name: test_rb_integer_type_p_for_bignum,
    func: integer_type_p,
    data_factory: {
      ruby_eval!("11111111111111111111111111111111111111111111111111111111111111111111111111111111111111")
    },
    expected: true
);

parity_test!(
    name: test_rb_integer_type_p_for_float,
    func: integer_type_p,
    data_factory: {
      ruby_eval!("1.0")
    },
    expected: false
);

parity_test!(
  name: test_int2fix_positive,
  func: int2fix,
  data_factory: { 42 },
  expected: ruby_eval!("42")
);

parity_test!(
  name: test_int2fix_negative,
  func: int2fix,
  data_factory: { -42 },
  expected: ruby_eval!("-42")
);

parity_test!(
  name: test_int2fix_zero,
  func: int2fix,
  data_factory: { 0 },
  expected: ruby_eval!("0")
);

parity_test!(
  name: test_int2num_fixnum,
  func: int2num,
  data_factory: { 42 },
  expected: ruby_eval!("42")
);

parity_test!(
  name: test_int2num_bignum_positive,
  func: int2num,
  data_factory: { i32::MAX },
  expected: ruby_eval!("2147483647")
);

parity_test!(
  name: test_int2num_bignum_negative,
  func: int2num,
  data_factory: { i32::MIN },
  expected: ruby_eval!("-2147483648")
);

parity_test!(
  name: test_fix2long_positive,
  func: fix2long,
  data_factory: { ruby_eval!("42") },
  expected: 42
);

parity_test!(
  name: test_fix2long_negative,
  func: fix2long,
  data_factory: { ruby_eval!("-42") },
  expected: -42
);

parity_test!(
  name: test_fix2long_zero,
  func: fix2long,
  data_factory: { ruby_eval!("0") },
  expected: 0
);

parity_test!(
  name: test_num2long_fixnum,
  func: num2long,
  data_factory: { ruby_eval!("42") },
  expected: 42
);

parity_test!(
  name: test_num2long_bignum,
  func: num2long,
  data_factory: { ruby_eval!("9223372036854775807") },  // i64::MAX
  expected: 9223372036854775807
);

parity_test!(
  name: test_num2long_negative_bignum,
  func: num2long,
  data_factory: { ruby_eval!("-9223372036854775808") },  // i64::MIN
  expected: -9223372036854775808
);

parity_test!(
    name: test_int2fix_max,
    func: int2fix,
    data_factory: { RUBY_FIXNUM_MAX as c_int },
    expected: ruby_eval!("{}", rb_sys::RUBY_FIXNUM_MAX as c_int)
);

parity_test!(
    name: test_int2fix_min,
    func: int2fix,
    data_factory: { RUBY_FIXNUM_MIN as c_int },
    expected: ruby_eval!("{}", rb_sys::RUBY_FIXNUM_MIN as c_int)
);

parity_test!(
    name: test_int2num_max_fixnum,
    func: int2num,
    data_factory: { RUBY_FIXNUM_MAX as c_int },
    expected: ruby_eval!("{}", rb_sys::RUBY_FIXNUM_MAX as c_int)
);

parity_test!(
    name: test_int2num_min_fixnum,
    func: int2num,
    data_factory: { RUBY_FIXNUM_MIN as c_int },
    expected: ruby_eval!("{}", rb_sys::RUBY_FIXNUM_MIN as c_int)
);

parity_test!(
    name: test_int2num_max_int,
    func: int2num,
    data_factory: { c_int::MAX },
    expected: ruby_eval!("2147483647")  // Assuming 32-bit int
);

parity_test!(
    name: test_int2num_min_int,
    func: int2num,
    data_factory: { c_int::MIN },
    expected: ruby_eval!("-2147483648")  // Assuming 32-bit int
);

parity_test!(
    name: test_fix2long_max,
    func: fix2long,
    data_factory: { ruby_eval!("{}", rb_sys::RUBY_FIXNUM_MAX) },
    expected: RUBY_FIXNUM_MAX as c_long
);

parity_test!(
    name: test_fix2long_min,
    func: fix2long,
    data_factory: { ruby_eval!("{}", rb_sys::RUBY_FIXNUM_MIN) },
    expected: RUBY_FIXNUM_MIN as c_long
);

parity_test!(
    name: test_num2long_max_fixnum,
    func: num2long,
    data_factory: { ruby_eval!("{}", rb_sys::RUBY_FIXNUM_MAX) },
    expected: RUBY_FIXNUM_MAX as c_long
);

parity_test!(
    name: test_num2long_min_fixnum,
    func: num2long,
    data_factory: { ruby_eval!("{}", rb_sys::RUBY_FIXNUM_MIN) },
    expected: RUBY_FIXNUM_MIN as c_long
);

parity_test!(
    name: test_num2long_max_long,
    func: num2long,
    data_factory: { ruby_eval!("9223372036854775807") },  // Assuming 64-bit long
    expected: c_long::MAX
);

parity_test!(
    name: test_num2long_min_long,
    func: num2long,
    data_factory: { ruby_eval!("-9223372036854775808") },  // Assuming 64-bit long
    expected: c_long::MIN
);
