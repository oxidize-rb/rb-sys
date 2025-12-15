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

#[rb_sys_test_helpers::ruby_test]
fn test_id2sym_and_sym2id_roundtrip() {
    unsafe {
        // Create a symbol from a string
        let id = rb_sys::rb_intern2("test_symbol".as_ptr() as _, 12);

        // Convert to symbol and back
        let sym = rb_sys::ID2SYM(id);
        let id2 = rb_sys::SYM2ID(sym);

        assert_eq!(id, id2);
    }
}

#[rb_sys_test_helpers::ruby_test]
fn test_id2sym_creates_symbol() {
    unsafe {
        let id = rb_sys::rb_intern2("another_symbol".as_ptr() as _, 14);
        let sym = rb_sys::ID2SYM(id);

        // Verify it's actually a symbol by converting back
        let id2 = rb_sys::SYM2ID(sym);
        assert_eq!(id, id2);
    }
}

#[rb_sys_test_helpers::ruby_test]
fn test_sym2id_static_symbol() {
    unsafe {
        // Static symbols are small interned symbols
        let id = rb_sys::rb_intern2("foo".as_ptr() as _, 3);
        let sym = rb_sys::ID2SYM(id);

        // Verify conversion roundtrips
        let id2 = rb_sys::SYM2ID(sym);
        assert_eq!(id, id2);
    }
}

#[rb_sys_test_helpers::ruby_test]
fn test_id2sym_and_sym2id_aliases() {
    unsafe {
        // Test the RB_ prefixed aliases
        let id = rb_sys::rb_intern2("alias_test".as_ptr() as _, 10);
        let sym1 = rb_sys::ID2SYM(id);
        let sym2 = rb_sys::RB_ID2SYM(id);
        assert_eq!(sym1, sym2);

        let id1 = rb_sys::SYM2ID(sym1);
        let id2 = rb_sys::RB_SYM2ID(sym2);
        assert_eq!(id1, id2);
        assert_eq!(id1, id);
    }
}

parity_test!(
    name: test_id2sym_parity,
    func: id2sym,
    data_factory: {
        unsafe { rb_sys::rb_intern2("parity_test".as_ptr() as _, 11) }
    }
);

parity_test!(
    name: test_sym2id_parity,
    func: sym2id,
    data_factory: {
        unsafe {
            let id = rb_sys::rb_intern2("parity_sym".as_ptr() as _, 10);
            rb_sys::ID2SYM(id)
        }
    }
);

// Integer conversion tests - fix2long
parity_test!(
    name: test_fix2long_zero,
    func: fix2long,
    data_factory: { ruby_eval!("0") },
    expected: 0 as std::os::raw::c_long
);

parity_test!(
    name: test_fix2long_one,
    func: fix2long,
    data_factory: { ruby_eval!("1") },
    expected: 1 as std::os::raw::c_long
);

parity_test!(
    name: test_fix2long_negative_one,
    func: fix2long,
    data_factory: { ruby_eval!("-1") },
    expected: -1 as std::os::raw::c_long
);

parity_test!(
    name: test_fix2long_positive_small,
    func: fix2long,
    data_factory: { ruby_eval!("42") },
    expected: 42 as std::os::raw::c_long
);

parity_test!(
    name: test_fix2long_negative_small,
    func: fix2long,
    data_factory: { ruby_eval!("-123") },
    expected: -123 as std::os::raw::c_long
);

parity_test!(
    name: test_fix2long_large_positive,
    func: fix2long,
    data_factory: { ruby_eval!("1073741823") },  // 2^30 - 1
    expected: 1073741823 as std::os::raw::c_long
);

parity_test!(
    name: test_fix2long_large_negative,
    func: fix2long,
    data_factory: { ruby_eval!("-1073741824") },  // -2^30
    expected: -1073741824 as std::os::raw::c_long
);

// 64-bit specific tests (c_long is i64 on Unix)
#[cfg(not(windows))]
parity_test!(
    name: test_fix2long_max_fixnum,
    func: fix2long,
    data_factory: { ruby_eval!("(2 ** 62) - 1") },  // FIXNUM_MAX on 64-bit
    expected: ((1i64 << 62) - 1) as std::os::raw::c_long
);

#[cfg(not(windows))]
parity_test!(
    name: test_fix2long_min_fixnum,
    func: fix2long,
    data_factory: { ruby_eval!("-(2 ** 62)") },  // FIXNUM_MIN on 64-bit
    expected: (-(1i64 << 62)) as std::os::raw::c_long
);

// Windows-specific tests (c_long is i32 on Windows)
#[cfg(windows)]
parity_test!(
    name: test_fix2long_max_fixnum,
    func: fix2long,
    data_factory: { ruby_eval!("(2 ** 30) - 1") },  // FIXNUM_MAX on 32-bit
    expected: ((1i32 << 30) - 1) as std::os::raw::c_long
);

#[cfg(windows)]
parity_test!(
    name: test_fix2long_min_fixnum,
    func: fix2long,
    data_factory: { ruby_eval!("-(2 ** 30)") },  // FIXNUM_MIN on 32-bit
    expected: (-(1i32 << 30)) as std::os::raw::c_long
);

// Integer conversion tests - fix2ulong
parity_test!(
    name: test_fix2ulong_zero,
    func: fix2ulong,
    data_factory: { ruby_eval!("0") },
    expected: 0 as std::os::raw::c_ulong
);

parity_test!(
    name: test_fix2ulong_one,
    func: fix2ulong,
    data_factory: { ruby_eval!("1") },
    expected: 1 as std::os::raw::c_ulong
);

parity_test!(
    name: test_fix2ulong_small,
    func: fix2ulong,
    data_factory: { ruby_eval!("42") },
    expected: 42 as std::os::raw::c_ulong
);

parity_test!(
    name: test_fix2ulong_medium,
    func: fix2ulong,
    data_factory: { ruby_eval!("1000") },
    expected: 1000 as std::os::raw::c_ulong
);

parity_test!(
    name: test_fix2ulong_large,
    func: fix2ulong,
    data_factory: { ruby_eval!("1073741823") },  // 2^30 - 1
    expected: 1073741823 as std::os::raw::c_ulong
);

// 64-bit specific test
#[cfg(not(windows))]
parity_test!(
    name: test_fix2ulong_max_fixnum,
    func: fix2ulong,
    data_factory: { ruby_eval!("(2 ** 62) - 1") },  // FIXNUM_MAX on 64-bit
    expected: ((1u64 << 62) - 1) as std::os::raw::c_ulong
);

// Windows-specific test
#[cfg(windows)]
parity_test!(
    name: test_fix2ulong_max_fixnum,
    func: fix2ulong,
    data_factory: { ruby_eval!("(2 ** 30) - 1") },  // FIXNUM_MAX on 32-bit
    expected: ((1u32 << 30) - 1) as std::os::raw::c_ulong
);

// Integer conversion tests - num2long (handles both fixnum and bignum)
parity_test!(
    name: test_num2long_zero,
    func: num2long,
    data_factory: { ruby_eval!("0") },
    expected: 0 as std::os::raw::c_long
);

parity_test!(
    name: test_num2long_one,
    func: num2long,
    data_factory: { ruby_eval!("1") },
    expected: 1 as std::os::raw::c_long
);

parity_test!(
    name: test_num2long_negative_one,
    func: num2long,
    data_factory: { ruby_eval!("-1") },
    expected: -1 as std::os::raw::c_long
);

parity_test!(
    name: test_num2long_positive_small,
    func: num2long,
    data_factory: { ruby_eval!("999") },
    expected: 999 as std::os::raw::c_long
);

parity_test!(
    name: test_num2long_negative_small,
    func: num2long,
    data_factory: { ruby_eval!("-999") },
    expected: -999 as std::os::raw::c_long
);

parity_test!(
    name: test_num2long_large_positive,
    func: num2long,
    data_factory: { ruby_eval!("2147483647") },  // 2^31 - 1 (i32 max)
    expected: 2147483647 as std::os::raw::c_long
);

parity_test!(
    name: test_num2long_large_negative,
    func: num2long,
    data_factory: { ruby_eval!("-2147483648") },  // -2^31 (i32 min)
    expected: -2147483648 as std::os::raw::c_long
);

// Integer conversion tests - num2ulong
parity_test!(
    name: test_num2ulong_zero,
    func: num2ulong,
    data_factory: { ruby_eval!("0") },
    expected: 0 as std::os::raw::c_ulong
);

parity_test!(
    name: test_num2ulong_one,
    func: num2ulong,
    data_factory: { ruby_eval!("1") },
    expected: 1 as std::os::raw::c_ulong
);

parity_test!(
    name: test_num2ulong_small,
    func: num2ulong,
    data_factory: { ruby_eval!("888") },
    expected: 888 as std::os::raw::c_ulong
);

parity_test!(
    name: test_num2ulong_large,
    func: num2ulong,
    data_factory: { ruby_eval!("4294967295") },  // 2^32 - 1 (u32 max)
    expected: 4294967295 as std::os::raw::c_ulong
);

// long2num/ulong2num parity tests
// These functions take a primitive and return VALUE, so we need a different pattern
macro_rules! parity_test_long2num {
    (name: $name:ident, value: $value:expr) => {
        #[rb_sys_test_helpers::ruby_test]
        fn $name() {
            use rb_sys::stable_api;
            let val: std::os::raw::c_long = $value;

            let rust_result = stable_api::get_default().long2num(val);
            let compiled_c_result = stable_api::get_compiled().long2num(val);

            // Compare by converting back to long
            let rust_roundtrip = unsafe { stable_api::get_default().num2long(rust_result) };
            let compiled_roundtrip =
                unsafe { stable_api::get_compiled().num2long(compiled_c_result) };

            assert_eq!(
                rust_roundtrip, compiled_roundtrip,
                "long2num parity failed for {}: rust={}, compiled={}",
                val, rust_roundtrip, compiled_roundtrip
            );
            assert_eq!(rust_roundtrip, val, "roundtrip failed for {}", val);
        }
    };
}

macro_rules! parity_test_ulong2num {
    (name: $name:ident, value: $value:expr) => {
        #[rb_sys_test_helpers::ruby_test]
        fn $name() {
            use rb_sys::stable_api;
            let val: std::os::raw::c_ulong = $value;

            let rust_result = stable_api::get_default().ulong2num(val);
            let compiled_c_result = stable_api::get_compiled().ulong2num(val);

            // Compare by converting back to ulong
            let rust_roundtrip = unsafe { stable_api::get_default().num2ulong(rust_result) };
            let compiled_roundtrip =
                unsafe { stable_api::get_compiled().num2ulong(compiled_c_result) };

            assert_eq!(
                rust_roundtrip, compiled_roundtrip,
                "ulong2num parity failed for {}: rust={}, compiled={}",
                val, rust_roundtrip, compiled_roundtrip
            );
            assert_eq!(rust_roundtrip, val, "roundtrip failed for {}", val);
        }
    };
}

// long2num parity tests
parity_test_long2num!(name: test_long2num_zero, value: 0);
parity_test_long2num!(name: test_long2num_one, value: 1);
parity_test_long2num!(name: test_long2num_negative_one, value: -1);
parity_test_long2num!(name: test_long2num_small_positive, value: 12345);
parity_test_long2num!(name: test_long2num_small_negative, value: -12345);
parity_test_long2num!(name: test_long2num_large_positive, value: 2147483647); // i32 max
parity_test_long2num!(name: test_long2num_large_negative, value: -2147483648); // i32 min

// 64-bit specific long2num tests
#[cfg(not(windows))]
parity_test_long2num!(name: test_long2num_very_large_positive, value: 4611686018427387903); // 2^62 - 1 (near FIXNUM_MAX)
#[cfg(not(windows))]
parity_test_long2num!(name: test_long2num_very_large_negative, value: -4611686018427387904); // -2^62 (near FIXNUM_MIN)

// Windows-specific long2num tests
#[cfg(windows)]
parity_test_long2num!(name: test_long2num_very_large_positive, value: 1073741823); // 2^30 - 1 (near FIXNUM_MAX on 32-bit)
#[cfg(windows)]
parity_test_long2num!(name: test_long2num_very_large_negative, value: -1073741824); // -2^30 (near FIXNUM_MIN on 32-bit)

// ulong2num parity tests
parity_test_ulong2num!(name: test_ulong2num_zero, value: 0);
parity_test_ulong2num!(name: test_ulong2num_one, value: 1);
parity_test_ulong2num!(name: test_ulong2num_small, value: 54321);
parity_test_ulong2num!(name: test_ulong2num_large, value: 4294967295); // u32 max

// 64-bit specific ulong2num test
#[cfg(not(windows))]
parity_test_ulong2num!(name: test_ulong2num_very_large, value: 4611686018427387903); // 2^62 - 1 (near FIXNUM_MAX)

// Windows-specific ulong2num test
#[cfg(windows)]
parity_test_ulong2num!(name: test_ulong2num_very_large, value: 1073741823); // 2^30 - 1 (near FIXNUM_MAX on 32-bit)

// fixable/posfixable parity tests
#[rb_sys_test_helpers::ruby_test]
fn test_fixable_parity() {
    use rb_sys::stable_api;
    let rust_api = stable_api::get_default();
    let compiled_api = stable_api::get_compiled();

    #[cfg(not(windows))]
    const TEST_VALUES: &[std::os::raw::c_long] = &[
        0,
        1,
        -1,
        100,
        -100,
        1000,
        -1000,
        2147483647,           // i32 max
        -2147483648,          // i32 min
        4611686018427387903,  // 2^62 - 1 (FIXNUM_MAX on 64-bit)
        -4611686018427387904, // -2^62 (FIXNUM_MIN on 64-bit)
        4611686018427387904,  // 2^62 (just above FIXNUM_MAX, not fixable)
        -4611686018427387905, // just below FIXNUM_MIN (not fixable)
    ];

    #[cfg(windows)]
    const TEST_VALUES: &[std::os::raw::c_long] = &[
        0,
        1,
        -1,
        100,
        -100,
        1000,
        -1000,
        1073741823,  // 2^30 - 1 (FIXNUM_MAX on 32-bit)
        -1073741824, // -2^30 (FIXNUM_MIN on 32-bit)
        1073741824,  // 2^30 (just above FIXNUM_MAX, not fixable)
        -1073741825, // just below FIXNUM_MIN (not fixable)
        2147483647,  // i32 max (not fixable)
        -2147483648, // i32 min (not fixable)
    ];

    for &val in TEST_VALUES {
        let rust_result = rust_api.fixable(val);
        let compiled_result = compiled_api.fixable(val);
        assert_eq!(
            rust_result, compiled_result,
            "fixable parity failed for {}: rust={}, compiled={}",
            val, rust_result, compiled_result
        );
    }
}

#[rb_sys_test_helpers::ruby_test]
fn test_posfixable_parity() {
    use rb_sys::stable_api;
    let rust_api = stable_api::get_default();
    let compiled_api = stable_api::get_compiled();

    #[cfg(not(windows))]
    const TEST_VALUES: &[std::os::raw::c_ulong] = &[
        0,
        1,
        100,
        1000,
        2147483647,           // i32 max
        4294967295,           // u32 max
        4611686018427387903,  // 2^62 - 1 (FIXNUM_MAX on 64-bit)
        4611686018427387904,  // 2^62 (just above FIXNUM_MAX)
        9223372036854775807,  // i64 max as u64
        18446744073709551615, // u64 max
    ];

    #[cfg(windows)]
    const TEST_VALUES: &[std::os::raw::c_ulong] = &[
        0, 1, 100, 1000, 1073741823, // 2^30 - 1 (FIXNUM_MAX on 32-bit)
        1073741824, // 2^30 (just above FIXNUM_MAX)
        2147483647, // i32 max
        4294967295, // u32 max
    ];

    for &val in TEST_VALUES {
        let rust_result = rust_api.posfixable(val);
        let compiled_result = compiled_api.posfixable(val);
        assert_eq!(
            rust_result, compiled_result,
            "posfixable parity failed for {}: rust={}, compiled={}",
            val, rust_result, compiled_result
        );
    }
}

#[rb_sys_test_helpers::ruby_test]
fn test_rb_obj_write_basic() {
    unsafe {
        // Create an array to hold a reference
        let ary = rb_sys::rb_ary_new_capa(1);
        rb_sys::rb_ary_push(ary, rb_sys::Qnil as VALUE);
        
        // Create a string to store
        let str = rb_sys::rb_str_new_cstr(b"test\0".as_ptr() as _);
        
        // Get pointer to first element using stable API
        let ptr = stable_api::get_default().rarray_const_ptr(ary) as *mut VALUE;
        
        // Use write barrier to store reference
        let result = rb_sys::RB_OBJ_WRITE(ary, ptr, str);
        
        // Verify the result is the value we wrote
        assert_eq!(result, str);
    }
}

#[rb_sys_test_helpers::ruby_test]
fn test_rb_obj_written_basic() {
    unsafe {
        // Create an array
        let ary = rb_sys::rb_ary_new();
        
        // Create a string
        let str = rb_sys::rb_str_new_cstr(b"test\0".as_ptr() as _);
        
        // Manually write the value (simulating a write that happened elsewhere)
        rb_sys::rb_ary_push(ary, str);
        
        // Inform GC about the write
        let result = rb_sys::RB_OBJ_WRITTEN(ary, rb_sys::Qnil as VALUE, str);
        
        // Verify the result is the value we wrote
        assert_eq!(result, str);
    }
}

#[rb_sys_test_helpers::ruby_test]
fn test_rb_obj_write_multiple_elements() {
    unsafe {
        // Create an array with multiple elements
        let ary = rb_sys::rb_ary_new_capa(3);
        rb_sys::rb_ary_push(ary, rb_sys::Qnil as VALUE);
        rb_sys::rb_ary_push(ary, rb_sys::Qnil as VALUE);
        rb_sys::rb_ary_push(ary, rb_sys::Qnil as VALUE);
        
        // Create strings to store
        let str1 = rb_sys::rb_str_new_cstr(b"first\0".as_ptr() as _);
        let str2 = rb_sys::rb_str_new_cstr(b"second\0".as_ptr() as _);
        let str3 = rb_sys::rb_str_new_cstr(b"third\0".as_ptr() as _);
        
        // Get pointer to array elements using stable API
        let ptr = stable_api::get_default().rarray_const_ptr(ary) as *mut VALUE;
        
        // Write each element with write barrier
        rb_sys::RB_OBJ_WRITE(ary, ptr, str1);
        rb_sys::RB_OBJ_WRITE(ary, ptr.add(1), str2);
        rb_sys::RB_OBJ_WRITE(ary, ptr.add(2), str3);
        
        // Verify all values were written correctly by checking array length
        let len = stable_api::get_default().rarray_len(ary);
        assert_eq!(len, 3);
    }
}
