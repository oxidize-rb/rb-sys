//! Stable ABI functions which provide access to Ruby internals that
//! is compatible across Ruby versions, and are guaranteed to be not break due
//! to Ruby binary changes.
//!
//! ### Goals
//!
//! 1. To provide access to Ruby internals that are not exposed by the libruby
//!    (i.e. C macros and inline functions).
//! 2. Provide support for Ruby development versions, which can make breaking
//!    changes without semantic versioning. We want to support these versions
//!    to ensure Rust extensions don't prevent the Ruby core team from testing
//!    changes in production.
//!

use crate::VALUE;
use std::ffi::{c_char, c_long};

pub trait StableAbiDefinition {
    /// Get the length of a Ruby string (akin to `RSTRING_LEN`).
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying Ruby data. The caller must ensure that the pointer
    /// is valid.
    unsafe fn rstring_len(obj: VALUE) -> c_long;

    /// Get a pointer to the bytes of a Ruby string (akin to `RSTRING_PTR`).
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying Ruby data. The caller must ensure that the pointer
    /// is valid.
    unsafe fn rstring_ptr(obj: VALUE) -> *const c_char;

    /// Get the length of a Ruby array (akin to `RARRAY_LEN`).
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying Ruby data. The caller must ensure that the pointer
    /// is valid.
    unsafe fn rarray_len(obj: VALUE) -> c_long;

    /// Get a pointer to the elements of a Ruby array (akin to `RARRAY_CONST_PTR`).
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying Ruby data. The caller must ensure that the pointer
    /// is valid.
    unsafe fn rarray_const_ptr(obj: VALUE) -> *const VALUE;
}

#[cfg(any(compiled_stable_abi_available, feature = "compiled-stable-abi"))]
mod compiled;

#[cfg(ruby_eq_2_4)]
#[path = "stable_abi/ruby_2_4.rs"]
mod abi;

#[cfg(ruby_eq_2_7)]
#[path = "stable_abi/ruby_2_7.rs"]
mod abi;

#[cfg(ruby_eq_3_0)]
#[path = "stable_abi/ruby_3_0.rs"]
mod abi;

#[cfg(ruby_eq_3_1)]
#[path = "stable_abi/ruby_3_1.rs"]
mod abi;

#[cfg(ruby_eq_3_2)]
#[path = "stable_abi/ruby_3_2.rs"]
mod abi;

#[cfg(ruby_gt_3_2)]
#[path = "stable_abi/ruby_dev.rs"]
mod abi;

pub use abi::Definition as StableAbi;

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
                use super::StableAbiDefinition;
                let data = $data_factory;

                #[allow(unused)]
                let rust_result = unsafe { super::StableAbi::$func(data) };
                let compiled_c_result = unsafe { super::compiled::Definition::$func(data) };

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
