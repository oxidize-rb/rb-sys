#![allow(rustdoc::bare_urls)]
#![doc = include_str!("../readme.md")]
mod once_cell;
mod ruby_exception;
mod ruby_test_executor;
mod utils;

use rb_sys::{rb_errinfo, rb_intern, rb_set_errinfo, Qnil, VALUE};
use ruby_test_executor::global_executor;
use std::{error::Error, mem::MaybeUninit, panic::UnwindSafe};

pub use rb_sys_test_helpers_macros::*;
pub use ruby_exception::RubyException;
pub use ruby_test_executor::{cleanup_ruby, setup_ruby, setup_ruby_unguarded};

/// Run a given function with inside of a Ruby VM.
///
/// Doing this properly it is not trivial, so this function abstracts away the
/// details. Under the hood, it ensures:
///
/// 1. The Ruby VM is setup and initialized once and only once.
/// 2. All code runs on the same OS thread.
/// 3. Exceptions are properly handled and propagated as Rust `Result<T,
///    RubyException>` values.
///
/// ### Example
///
/// ```
/// use rb_sys_test_helpers::with_ruby_vm;
/// use std::ffi::CStr;
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
pub fn with_ruby_vm<R, F>(f: F) -> Result<R, Box<dyn Error>>
where
    R: Send + 'static,
    F: FnOnce() -> R + UnwindSafe + Send + 'static,
{
    global_executor().run_test(f)
}

/// Runs a test with GC stress enabled to help find GC bugs.
///
/// ### Example
///
/// ```
/// use rb_sys_test_helpers::{with_gc_stress, with_ruby_vm};
/// use std::ffi::CStr;
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
pub fn with_gc_stress<R, F>(f: F) -> R
where
    R: Send + 'static,
    F: FnOnce() -> R + UnwindSafe + Send + 'static,
{
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
        let args: *mut (Option<*mut F>, *mut Option<T>) = args as _;
        let args = *args;
        let (mut func, outbuf) = args;
        let func = func.take().unwrap();
        let func = &mut *func;
        let result = func();
        outbuf.write_volatile(Some(result));
        outbuf as _
    }

    unsafe {
        let mut state = 0;
        let func_ref = &Some(f) as *const _;
        let mut outbuf: MaybeUninit<Option<T>> = MaybeUninit::new(None);
        let args = &(Some(func_ref), outbuf.as_mut_ptr() as *mut _) as *const _ as VALUE;
        rb_sys::rb_protect(Some(ffi_closure::<T, F>), args, &mut state);

        if state == 0 {
            if outbuf.as_mut_ptr().read_volatile().is_some() {
                Ok(outbuf.assume_init().expect("unreachable"))
            } else {
                Err(RubyException::new(rb_errinfo()))
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
    fn test_protect_returns_correct_value() -> Result<(), Box<dyn Error>> {
        let ret = with_ruby_vm(|| protect(|| "my val"))?;

        assert_eq!(ret, Ok("my val"));

        Ok(())
    }

    #[test]
    fn test_protect_capture_ruby_exception() {
        with_ruby_vm(|| unsafe {
            let result = protect(|| {
                rb_sys::rb_raise(rb_sys::rb_eRuntimeError, "hello world\0".as_ptr() as _);
            });

            assert!(result.is_err());
        })
        .unwrap();
    }
}
