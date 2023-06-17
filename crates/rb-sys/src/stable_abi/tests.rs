use crate as rb_sys;
use rb_sys_test_helpers::rstring as gen_rstring;

macro_rules! parity_test {
    (name: $name:ident, func: $func:ident, data_factory: $data_factory:expr) => {
        #[rb_sys_test_helpers::ruby_test]
        fn $name() {
            use super::StableAbiDefinition;
            let data = $data_factory;

            #[allow(unused)]
            let rust_result = unsafe { super::StableAbi::$func(data) };
            #[allow(unused_unsafe)]
            let compiled_c_result = unsafe { super::Compiled::$func(data) };

            assert_eq!(
                compiled_c_result, rust_result,
                "compiled_c was {:?}, rust was {:?}",
                compiled_c_result, rust_result
            );
        }
    };
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
      gen_rstring!(include_str!("../../../../Cargo.lock"))
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
    let mut state = 0;
    let ret = unsafe { rb_sys::rb_eval_string_protect("'foo'\0".as_ptr() as _, &mut state as _) };
    assert_eq!(state, 0);
    ret
  }
);

parity_test!(
  name: test_rstring_len_evaled_basic,
  func: rstring_len,
  data_factory: {
    let mut state = 0;
    let ret = unsafe { rb_sys::rb_eval_string_protect("'foo'\0".as_ptr() as _, &mut state as _) };
    assert_eq!(state, 0);
    ret
  }
);

parity_test!(
  name: test_rstring_len_evaled_shared,
  func: rstring_len,
  data_factory: {
    let mut state = 0;
    let ret = unsafe { rb_sys::rb_eval_string_protect("'foo' + 'bar' + ('a' * 12)\0".as_ptr() as _, &mut state as _) };
    let ret = unsafe { rb_sys::rb_str_new_shared(ret) };
    unsafe { rb_sys::rb_str_cat_cstr(ret, "baz\0".as_ptr() as _)};
    assert_eq!(state, 0);
    ret
  }
);

parity_test!(
  name: test_rstring_ptr_evaled_empty,
  func: rstring_ptr,
  data_factory: {
    let mut state = 0;
    let ret = unsafe { rb_sys::rb_eval_string_protect("''\0".as_ptr() as _, &mut state as _) };
    assert_eq!(state, 0);
    ret
  }
);

parity_test!(
    name: test_rstring_ptr_long,
    func: rstring_ptr,
    data_factory: {
      gen_rstring!(include_str!("../../../../Cargo.lock"))
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
      let mut state = 0;
      let ret = unsafe { rb_sys::rb_eval_string_protect("[2, nil, :foo]\0".as_ptr() as _, &mut state as _) };
      assert_eq!(state, 0);
      ret
    }
);

parity_test!(
    name: test_rarray_len_evaled_empty,
    func: rarray_len,
    data_factory: {
      let mut state = 0;
      let ret = unsafe { rb_sys::rb_eval_string_protect("[]\0".as_ptr() as _, &mut state as _) };
      assert_eq!(state, 0);
      ret
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
      let mut state = 0;
      let ret = unsafe { rb_sys::rb_eval_string_protect("[2, nil, :foo]\0".as_ptr() as _, &mut state as _) };
      assert_eq!(state, 0);
      ret
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
