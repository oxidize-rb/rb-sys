//! Helpers for testing Ruby from Rust.
//!
//! ## Usage
//!
//! In your `Cargo.toml`:
//!
//! ```toml
//! [dev-dependencies]
//! rb-sys = { version = "0.9", features = ["link-ruby"] }
//! rb-sys-test-helpers = { version = "0.1" }
//! ```
//!
//! In your `tests/*.rs`:
//!
//! ```rust
//! use rb_sys_test_helpers::with_ruby_vm;
//!
//! #[test]
//! fn test_something() {
//!     with_ruby_vm(|| {
//!         // Your test code that needs Ruby goes here...
//!     });
//! }
//! ```
//!

mod ruby_exception;
mod ruby_test_executor;
mod utils;

use rb_sys::{rb_errinfo, rb_intern, rb_set_errinfo, Qnil, VALUE};
use ruby_test_executor::global_executor;
use std::{mem::MaybeUninit, panic::UnwindSafe};

pub use rb_sys_test_helpers_macros::*;
pub use ruby_exception::RubyException;
pub use utils::*;

/// Initializes a Ruby VM, and ensures all tests are run by the same thread and
/// that the Ruby VM is initialized from.
///
/// ### Example
///
/// ```
/// use rb_sys_test_helpers::with_ruby_vm;
/// use core::ffi::CStr;
///
/// with_ruby_vm(|| unsafe {
///     let mut hello = rb_sys::rb_utf8_str_new_cstr("hello \0".as_ptr() as _);
///     rb_sys::rb_str_cat(hello, "world\0".as_ptr() as _, 5);
///     let result = rb_sys::rb_string_value_cstr(&mut hello);
///     let result = CStr::from_ptr(result).to_string_lossy().into_owned();
///
///     assert_eq!(result, "hello world");
/// });
/// ```
pub fn with_ruby_vm<F>(f: F)
where
    F: FnOnce() + UnwindSafe + Send + 'static,
{
    global_executor().run(f)
}

/// Runs a test with GC stress enabled to help find GC bugs.
///
/// ### Example
///
/// ```
/// use rb_sys_test_helpers::{with_gc_stress, with_ruby_vm};
/// use core::ffi::CStr;
///
/// with_ruby_vm(|| unsafe {
///     let hello_world = with_gc_stress(|| unsafe {
///         let mut rstring = rb_sys::rb_utf8_str_new_cstr("hello world\0".as_ptr() as _);
///         let result = rb_sys::rb_string_value_cstr(&mut rstring);
///         CStr::from_ptr(result).to_string_lossy().into_owned()
///     });
///
///    assert_eq!(hello_world, "hello world");
/// });
/// ```
pub fn with_gc_stress<T>(f: impl FnOnce() -> T + std::panic::UnwindSafe) -> T {
    unsafe {
        let stress_intern = rb_intern("stress\0".as_ptr() as _);
        let stress_eq_intern = rb_intern("stress=\0".as_ptr() as _);
        let gc_module = rb_sys::rb_const_get(rb_sys::rb_cObject, rb_intern("GC\0".as_ptr() as _));

        let old_gc_stress = rb_sys::rb_funcall(gc_module, stress_intern, 0);
        rb_sys::rb_funcall(gc_module, stress_eq_intern, 1, rb_sys::Qtrue);
        let result = std::panic::catch_unwind(f);
        rb_sys::rb_funcall(gc_module, stress_eq_intern, 1, old_gc_stress);

        match result {
            Ok(result) => result,
            Err(err) => std::panic::resume_unwind(err),
        }
    }
}

/// Catches a Ruby exception and returns it as a `Result` (using [`rb_sys::rb_protect`]).
///
/// ### Example
///
/// ```
/// use rb_sys_test_helpers::{protect, with_ruby_vm, RubyException};
///
/// with_ruby_vm(|| unsafe {
///     let result: Result<&str, RubyException> = protect(|| {
///         rb_sys::rb_raise(rb_sys::rb_eRuntimeError, "oh no\0".as_ptr() as _);
///         "this will never be returned"
///     });
///
///     assert!(result.is_err());
///     assert!(result.unwrap_err().message().unwrap().contains("oh no"));
/// });
/// ```
pub fn protect<F, T>(f: F) -> Result<T, RubyException>
where
    F: FnMut() -> T + std::panic::UnwindSafe,
{
    unsafe extern "C" fn ffi_closure<T, F: FnMut() -> T>(args: VALUE) -> VALUE {
        let args: *mut (Option<*mut F>, Option<MaybeUninit<T>>) = args as _;
        let args = &mut *args;
        let (func, outbuf) = args;
        let func = func.take().unwrap();
        let func = &mut *func;
        let mut outbuf = outbuf.take().unwrap();

        let result = func();
        outbuf.write(result);

        outbuf.as_ptr() as _
    }

    unsafe {
        let mut state = 0;
        let func_ref = &Some(f) as *const _;
        let outbuf_ref = &MaybeUninit::uninit() as *const MaybeUninit<T>;
        let args = &(Some(func_ref), Some(outbuf_ref)) as *const _ as VALUE;
        let outbuf_ptr = rb_sys::rb_protect(Some(ffi_closure::<T, F>), args, &mut state);
        let outbuf_ptr: *const MaybeUninit<T> = outbuf_ptr as _;

        if state == 0 {
            if let Some(result) = outbuf_ptr.as_ref() {
                Ok(result.assume_init_read())
            } else {
                panic!("rb_protect returned a null pointer")
            }
        } else {
            let err = rb_errinfo();
            rb_set_errinfo(Qnil as _);
            Err(RubyException::new(err))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protect_returns_correct_value() {
        with_ruby_vm(|| {
            let result = protect(|| "my val");

            assert_eq!(result, Ok("my val"));
        });
    }

    #[test]
    fn test_protect_capture_ruby_exception() {
        with_ruby_vm(|| unsafe {
            let result = protect(|| {
                rb_sys::rb_raise(rb_sys::rb_eRuntimeError, "hello world\0".as_ptr() as _);
            });

            assert!(result.is_err());
        });
    }
}
