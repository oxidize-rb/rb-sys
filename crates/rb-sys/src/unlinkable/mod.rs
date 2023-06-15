#[cfg(compiled_c_impls_available)]
mod compiled_c_impls;

#[cfg(any(ruby_abi_stable, feature = "bypass-stable-abi-version-checks"))]
mod rust_impls;

pub(crate) mod impls {
    #[cfg(all(compiled_c_impls_available, not(ruby_abi_stable),))]
    pub(crate) use super::compiled_c_impls::*;

    #[cfg(any(ruby_abi_stable))]
    pub(crate) use super::rust_impls::*;
}

pub(crate) use impls::{rarray_const_ptr, rarray_len, rstring_len, rstring_ptr};

#[cfg(test)]
mod tests {
    use crate as rb_sys;
    use rb_sys_test_helpers::rstring as gen_rstring;

    macro_rules! parity_test {
        (
            name: $name:ident,
            func: $func:ident,
            data_factory: $data_factory:expr
        ) => {
            #[rb_sys_test_helpers::ruby_test]
            fn $name() {
                let data = $data_factory;

                #[allow(unused)]
                let rust_result = unsafe { super::rust_impls::$func(data) };
                let compiled_c_result = unsafe { super::compiled_c_impls::$func(data) };

                assert_eq!(compiled_c_result, rust_result);
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
}
