use rb_sys_test_helpers::rstring as gen_rstring;

macro_rules! parity_test {
    (name: $name:ident, func: $func:ident, data_factory: $data_factory:expr) => {
        #[rb_sys_test_helpers::ruby_test]
        fn $name() {
            use rb_sys::stable_abi::*;
            let data = $data_factory;

            #[allow(unused)]
            let rust_result = unsafe { StableAbi::$func(data) };
            #[allow(unused_unsafe)]
            let compiled_c_result = unsafe { Compiled::$func(data) };

            assert_eq!(
                compiled_c_result, rust_result,
                "compiled_c was {:?}, rust was {:?}",
                compiled_c_result, rust_result
            );
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
    name: test_rb_builtin_type_for_string,
    func: rb_builtin_type,
    data_factory: {
      gen_rstring!("foo")
    }
);

parity_test!(
    name: test_rb_builtin_type_for_array,
    func: rb_builtin_type,
    data_factory: {
      ruby_eval!("[]")
    }
);

parity_test!(
    name: test_rb_builtin_type_for_hash,
    func: rb_builtin_type,
    data_factory: {
      ruby_eval!("{foo: 'bar'}")
    }
);

parity_test!(
    name: test_rb_builtin_type_for_file,
    func: rb_builtin_type,
    data_factory: {
      ruby_eval!("File.open('Cargo.toml')")
    }
);

parity_test!(
    name: test_rb_builtin_type_for_symbol,
    func: rb_builtin_type,
    data_factory: {
      ruby_eval!("'foosymmmm'.to_sym")
    }
);

parity_test! (
    name: test_rb_rb_nil_p_for_nil,
    func: rb_nil_p,
    data_factory: {
      rb_sys::Qnil as _
    }
);

parity_test! (
    name: test_rb_rb_nil_p_for_false,
    func: rb_nil_p,
    data_factory: {
      rb_sys::Qfalse as _
    }
);

parity_test! (
    name: test_rb_rb_nil_p_for_string,
    func: rb_nil_p,
    data_factory: {
      gen_rstring!("foo")
    }
);

parity_test! (
    name: test_rb_rb_fixnum_p_for_fixnum,
    func: rb_fixnum_p,
    data_factory: {
      ruby_eval!("1")
    }
);

parity_test! (
    name: test_rb_rb_fixnum_p_for_string,
    func: rb_fixnum_p,
    data_factory: {
      gen_rstring!("foo")
    }
);

parity_test! (
    name: test_rb_rb_static_sym_p_for_static_sym,
    func: rb_static_sym_p,
    data_factory: {
      let interned = unsafe { rb_sys::rb_intern2("new_sym".as_ptr() as _, 7) };
      unsafe { rb_sys::rb_id2sym(interned) }
    }
);

parity_test! (
    name: test_rb_rb_static_sym_p_for_regular_sym,
    func: rb_static_sym_p,
    data_factory: {
      ruby_eval!("'bar'.to_sym")
    }
);
