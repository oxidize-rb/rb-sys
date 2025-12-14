#![allow(rustdoc::bare_urls)]
#![doc = include_str!("../readme.md")]
mod once_cell;
mod ruby_exception;
mod ruby_test_executor;
mod utils;

use rb_sys::{rb_errinfo, rb_intern, rb_set_errinfo, Qnil, VALUE};
use ruby_test_executor::global_executor;
use std::{any::Any, error::Error, mem::MaybeUninit, panic::UnwindSafe};

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
    F: FnOnce() -> R,
{
    let _guard = GcStressGuard::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));

    match result {
        Ok(result) => result,
        Err(err) => std::panic::resume_unwind(err),
    }
}

/// Type alias for panic payloads captured by `protect()`.
type PanicPayload = Box<dyn Any + Send + 'static>;

/// The result of running a closure inside `ffi_closure`. This captures both
/// successful results and Rust panics, allowing us to propagate panics safely
/// across FFI boundaries.
enum ClosureResult<T> {
    /// The closure completed successfully with a value.
    Ok(T),
    /// The closure panicked with this payload.
    Panic(PanicPayload),
}

/// Catches a Ruby exception and returns it as a `Result` (using [`rb_sys::rb_protect`]).
///
/// This function also safely handles Rust panics that occur inside the closure.
/// Panics are caught before they cross the FFI boundary (which would be undefined
/// behavior) and are re-thrown after `rb_protect` returns.
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
        let args: *mut (Option<*mut F>, *mut Option<ClosureResult<T>>) = args as _;
        let args = *args;
        let (mut func, outbuf) = args;
        let func = func.take().unwrap();
        let func = &mut *func;

        // Catch panics before they cross the FFI boundary (which is UB).
        // The panic will be re-thrown after rb_protect returns.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(func));

        let closure_result = match result {
            Ok(value) => ClosureResult::Ok(value),
            Err(panic_payload) => ClosureResult::Panic(panic_payload),
        };

        outbuf.write_volatile(Some(closure_result));
        outbuf as _
    }

    unsafe {
        let mut state = 0;
        let func_ref = &Some(f) as *const _;
        let mut outbuf: MaybeUninit<Option<ClosureResult<T>>> = MaybeUninit::new(None);
        let args = &(Some(func_ref), outbuf.as_mut_ptr() as *mut _) as *const _ as VALUE;
        rb_sys::rb_protect(Some(ffi_closure::<T, F>), args, &mut state);

        if state == 0 {
            match outbuf.assume_init() {
                Some(ClosureResult::Ok(value)) => Ok(value),
                Some(ClosureResult::Panic(payload)) => {
                    // Re-throw the panic now that we're safely on the Rust side
                    std::panic::resume_unwind(payload)
                }
                None => Err(RubyException::new(rb_errinfo())),
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
    use rusty_fork::rusty_fork_test;

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

    rusty_fork_test! {
        #[test]
        fn test_protect_propagates_rust_panic_with_readable_output() {
            use std::panic;

            let result = with_ruby_vm(|| {
                panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    protect(|| {
                        panic!("this is a test panic message that should be visible");
                    })
                }))
            });

            // The test should complete (not abort) and the panic should be caught
            let outer_result = result.expect("with_ruby_vm should not fail");

            // The panic should have been propagated, not swallowed or aborted
            assert!(outer_result.is_err(), "panic should have been caught by catch_unwind");

            // Try to extract the panic message
            let panic_payload = outer_result.unwrap_err();
            let panic_msg = panic_payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| panic_payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown panic".to_string());

            assert!(
                panic_msg.contains("test panic message"),
                "panic message should be preserved, got: {}",
                panic_msg
            );
        }
    }

    // Test that Result-returning closures work correctly with protect()
    #[test]
    fn test_protect_with_result_ok() {
        with_ruby_vm(|| {
            let result = protect(|| -> Result<i32, &'static str> { Ok(42) });

            match result {
                Ok(Ok(value)) => assert_eq!(value, 42),
                Ok(Err(e)) => panic!("inner error: {}", e),
                Err(e) => panic!("ruby exception: {:?}", e),
            }
        })
        .unwrap();
    }

    #[test]
    fn test_protect_with_result_err() {
        with_ruby_vm(|| {
            let result = protect(|| -> Result<i32, &'static str> { Err("test error") });

            match result {
                Ok(Ok(_)) => panic!("expected error"),
                Ok(Err(e)) => assert_eq!(e, "test error"),
                Err(e) => panic!("ruby exception: {:?}", e),
            }
        })
        .unwrap();
    }

    #[test]
    fn test_gc_stress_guard() {
        with_ruby_vm(|| unsafe {
            let stress_intern = rb_intern("stress\0".as_ptr() as _);
            let gc_module =
                rb_sys::rb_const_get(rb_sys::rb_cObject, rb_intern("GC\0".as_ptr() as _));

            // Verify GC.stress is initially false
            let initial_stress = rb_sys::rb_funcall(gc_module, stress_intern, 0);
            assert_eq!(initial_stress, rb_sys::Qfalse as VALUE);

            {
                let _guard = GcStressGuard::new();

                // Verify GC.stress is now true
                let stress_during = rb_sys::rb_funcall(gc_module, stress_intern, 0);
                assert_eq!(stress_during, rb_sys::Qtrue as VALUE);
            }

            // Verify GC.stress is restored to false after guard is dropped
            let final_stress = rb_sys::rb_funcall(gc_module, stress_intern, 0);
            assert_eq!(final_stress, rb_sys::Qfalse as VALUE);
        })
        .unwrap();
    }

    #[test]
    fn test_with_gc_stress_restores_on_panic() {
        with_ruby_vm(|| unsafe {
            let stress_intern = rb_intern("stress\0".as_ptr() as _);
            let gc_module =
                rb_sys::rb_const_get(rb_sys::rb_cObject, rb_intern("GC\0".as_ptr() as _));

            // Verify GC.stress is initially false
            let initial_stress = rb_sys::rb_funcall(gc_module, stress_intern, 0);
            assert_eq!(initial_stress, rb_sys::Qfalse as VALUE);

            // Panic inside with_gc_stress and catch it
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                with_gc_stress(|| {
                    panic!("test panic");
                })
            }));

            assert!(result.is_err(), "should have panicked");

            // Verify GC.stress is restored to false after panic
            let final_stress = rb_sys::rb_funcall(gc_module, stress_intern, 0);
            assert_eq!(final_stress, rb_sys::Qfalse as VALUE);
        })
        .unwrap();
    }
}
