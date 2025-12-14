use rb_sys::{stable_api, StableApiDefinition, RTYPEDDATA_EMBEDDED_P, RTYPEDDATA_P, VALUE};
use rb_sys_test_helpers::rstring as gen_rstring;

macro_rules! parity_test {
  (name: $name:ident, func: $func:ident, data_factory: $data_factory:expr $(, expected: $expected:expr)?) => {
      #[rb_sys_test_helpers::ruby_test]
      fn $name() {
          use rb_sys::stable_api;
          let data = $data_factory;

          assert_ne!(stable_api::get_default().version(), (0, 0));

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
    ($expr:literal) => {{
        unsafe {
            let mut state = 0;
            let ret =
                rb_sys::rb_eval_string_protect(concat!($expr, "\0").as_ptr() as _, &mut state as _);

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

fn gen_typed_data() -> VALUE {
    ruby_eval!("Time.now")
}

fn gen_embedded_typed_data() -> VALUE {
    ruby_eval!("Time.at(0)")
}

fn gen_non_embedded_typed_data() -> VALUE {
    ruby_eval!("require 'stringio'; StringIO.new('a' * 1000)")
}

fn gen_non_typed_data() -> VALUE {
    ruby_eval!("Object.new")
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
    name: test_rbasic_class_of_array,
    func: rbasic_class,
    data_factory: {
        unsafe { rb_sys::rb_ary_new() as VALUE }
    },
    expected: {
        unsafe { Some(std::ptr::NonNull::new_unchecked(rb_sys::rb_cArray as _)) }
    }
);

parity_test!(
    name: test_rbasic_class_of_array_evaled,
    func: rbasic_class,
    data_factory: {
      ruby_eval!("[]")
    },
    expected: {
      unsafe { Some(std::ptr::NonNull::new_unchecked(ruby_eval!("Array") as *mut VALUE)) }
    }
);

parity_test!(
    name: test_rbasic_frozen_p_not_frozen_obj,
    func: frozen_p,
    data_factory: {
      ruby_eval!("[1]")
    },
    expected: false
);

parity_test!(
    name: test_rbasic_frozen_p_frozen_obj,
    func: frozen_p,
    data_factory: {
      ruby_eval!("[1].freeze")
    },
    expected: true
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
    name: test_bignum_positive_p_evaled,
    func: bignum_positive_p,
    data_factory: {
      ruby_eval!("2 ** 64")
    },
    expected: true
);

parity_test!(
    name: test_bignum_negative_p_evaled,
    func: bignum_negative_p,
    data_factory: {
      ruby_eval!("-(2 ** 64)")
    },
    expected: true
);

parity_test!(
    name: test_bignum_positive_p_for_zero,
    func: bignum_positive_p,
    data_factory: {
      unsafe { rb_sys::rb_int2big(0) }
    },
    expected: true
);

parity_test!(
    name: test_bignum_negative_p_for_zero,
    func: bignum_negative_p,
    data_factory: {
      unsafe { rb_sys::rb_int2big(0) }
    },
    expected: false
);

parity_test!(
    name: test_bignum_positive_p,
    func: bignum_positive_p,
    data_factory: {
      unsafe { rb_sys::rb_int2big(64) }
    },
    expected: true
);

parity_test!(
    name: test_bignum_negative_p,
    func: bignum_negative_p,
    data_factory: {
      unsafe { rb_sys::rb_int2big(-1) }
    },
    expected: true
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
      ruby_eval!("{foo: 'bar'}")
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
      ruby_eval!("{foo: 'bar'}")
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
    name: test_rb_string_interned_p,
    func: rstring_interned_p,
    data_factory: {
        ruby_eval!("'foo'")
    },
    expected: false
);

parity_test!(
    name: test_rb_string_interned_p_frozen_str,
    func: rstring_interned_p,
    data_factory: {
        ruby_eval!("'foo'.freeze")
    },
    expected: true
);

parity_test!(
    name: test_rb_thread_sleep,
    func: thread_sleep,
    data_factory: {
        std::time::Duration::from_millis(100)
    }
);

parity_test! (
    name: test_rtypeddata_p_for_typed_data,
    func: rtypeddata_p,
    data_factory: {
        gen_typed_data()
    },
    expected: true
);

parity_test! (
    name: test_rtypeddata_p_for_regular_data,
    func: rtypeddata_p,
    data_factory: {
        gen_non_typed_data()
    },
    expected: false
);

parity_test! (
    name: test_rtypeddata_p_for_string,
    func: rtypeddata_p,
    data_factory: {
        gen_rstring!("not a typed data")
    },
    expected: false
);

parity_test! (
    name: test_rtypeddata_embedded_p_for_typed_data,
    func: rtypeddata_embedded_p,
    data_factory: {
        gen_typed_data()
    }
);

parity_test! (
    name: test_rtypeddata_type_for_typed_data,
    func: rtypeddata_type,
    data_factory: {
        gen_typed_data()
    }
);

parity_test! (
    name: test_rtypeddata_get_data_for_typed_data,
    func: rtypeddata_get_data,
    data_factory: {
        gen_typed_data()
    }
);

parity_test! (
    name: test_rtypeddata_p_for_large_typed_data,
    func: rtypeddata_p,
    data_factory: {
        gen_non_embedded_typed_data()
    }
);

parity_test! (
    name: test_rtypeddata_embedded_p_for_small_data,
    func: rtypeddata_embedded_p,
    data_factory: {
        gen_embedded_typed_data()
    }
);

parity_test! (
    name: test_rtypeddata_embedded_p_for_large_data,
    func: rtypeddata_embedded_p,
    data_factory: {
        gen_non_embedded_typed_data()
    }
);

parity_test! (
    name: test_rtypeddata_get_data_for_small_data,
    func: rtypeddata_get_data,
    data_factory: {
        gen_embedded_typed_data()
    }
);

parity_test! (
    name: test_rtypeddata_get_data_for_large_data,
    func: rtypeddata_get_data,
    data_factory: {
        gen_non_embedded_typed_data()
    }
);

#[rb_sys_test_helpers::ruby_test]
fn test_rtypeddata_functions_with_usage() {
    let small_time = gen_embedded_typed_data();
    let large_time = gen_non_embedded_typed_data();

    unsafe {
        for obj in [small_time, large_time].iter() {
            assert!(RTYPEDDATA_P(*obj));

            let type_ptr = stable_api::get_default().rtypeddata_type(*obj);
            assert!(!type_ptr.is_null());

            let data_ptr = stable_api::get_default().rtypeddata_get_data(*obj);
            assert!(!data_ptr.is_null());
        }

        let _small_embedded = RTYPEDDATA_EMBEDDED_P(small_time);
        let large_embedded = RTYPEDDATA_EMBEDDED_P(large_time);

        #[cfg(ruby_gte_3_3)]
        assert!(_small_embedded);
        assert!(!large_embedded);
    }
}

// Tests for complex methods: float, hash, encoding

parity_test!(
    name: test_num2dbl_float,
    func: num2dbl,
    data_factory: {
        rb_sys::DBL2NUM(3.14159)
    }
);

parity_test!(
    name: test_num2dbl_fixnum,
    func: num2dbl,
    data_factory: {
        ruby_eval!("42")
    }
);

parity_test!(
    name: test_num2dbl_negative_fixnum,
    func: num2dbl,
    data_factory: {
        ruby_eval!("-100")
    }
);

#[rb_sys_test_helpers::ruby_test]
fn test_dbl2num_and_num2dbl_roundtrip() {
    unsafe {
        let original = 2.71828;
        let float_obj = rb_sys::DBL2NUM(original);
        let recovered = stable_api::get_default().num2dbl(float_obj);

        assert!((original - recovered).abs() < 0.00001);
    }
}

parity_test!(
    name: test_rhash_size_empty,
    func: rhash_size,
    data_factory: {
        ruby_eval!("{}")
    }
);

#[rb_sys_test_helpers::ruby_test]
fn test_rhash_size_with_elements() {
    unsafe {
        let h = ruby_eval!("{1 => 10, 2 => 20}");

        let size = stable_api::get_default().rhash_size(h);
        assert_eq!(size, 2);
    }
}

parity_test!(
    name: test_rhash_empty_p_empty,
    func: rhash_empty_p,
    data_factory: {
        ruby_eval!("{}")
    }
);

#[rb_sys_test_helpers::ruby_test]
fn test_rhash_empty_p_with_elements() {
    unsafe {
        let h = ruby_eval!("{1 => 10}");

        let is_empty = stable_api::get_default().rhash_empty_p(h);
        assert!(!is_empty);
    }
}

parity_test!(
    name: test_encoding_get_utf8_string,
    func: encoding_get,
    data_factory: {
        gen_rstring!("hello world")
    }
);

parity_test!(
    name: test_encoding_get_ascii_string,
    func: encoding_get,
    data_factory: {
        gen_rstring!("abc")
    }
);
