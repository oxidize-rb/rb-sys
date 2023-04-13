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

use std::{mem::MaybeUninit, panic::UnwindSafe, sync::Mutex};

use async_executor::LocalExecutor;
use futures_lite::future;

/// Initializes a Ruby VM, and ensures all tests are run by the same thread and
/// that the Ruby VM is initialized from.
///
/// ### Example
///
/// ```
/// use rb_sys_test_helpers::with_ruby_vm;
///
/// with_ruby_vm(|| unsafe {
///     let mut hello = rb_sys::rb_utf8_str_new_cstr("hello \0".as_ptr() as _);
///     rb_sys::rb_str_cat(hello, "world\0".as_ptr() as _, 5);
///     let result = rb_sys::rb_string_value_cstr(&mut hello);
///     let result = std::ffi::CStr::from_ptr(result).to_string_lossy().into_owned();
///
///     assert_eq!(result, "hello world");
/// });
/// ```
pub fn with_ruby_vm<F, T>(f: F) -> T
where
    F: FnOnce() -> T + UnwindSafe,
{
    static INIT: std::sync::Once = std::sync::Once::new();
    static mut EXECUTOR: MaybeUninit<RubyTestExecutor> = MaybeUninit::uninit();

    unsafe {
        INIT.call_once(|| {
            ruby_setup_ceremony();
            EXECUTOR.write(RubyTestExecutor::default());
        });
    }

    let executor = unsafe { EXECUTOR.assume_init_ref() };
    executor.run_test(f)
}

fn ruby_setup_ceremony() {
    unsafe {
        #[cfg(windows)]
        {
            let mut argc = 0;
            let mut argv: [*mut std::os::raw::c_char; 0] = [];
            let mut argv = argv.as_mut_ptr();
            rb_sys::rb_w32_sysinit(&mut argc, &mut argv);
        }

        match rb_sys::ruby_setup() {
            0 => {}
            code => panic!("Failed to setup Ruby (error code: {})", code),
        };

        let mut argv: [*mut i8; 3] = [
            "ruby\0".as_ptr() as _,
            "-e\0".as_ptr() as _,
            "\0".as_ptr() as _,
        ];

        let node = rb_sys::ruby_process_options(argv.len() as i32, argv.as_mut_ptr());

        match rb_sys::ruby_exec_node(node) {
            0 => {}
            code => panic!("Failed to execute Ruby (error code: {})", code),
        };
    }
}

/// Runs a test with GC stress enabled to help find GC bugs.
///
/// ### Example
///
/// ```
/// use rb_sys_test_helpers::{with_gc_stress, with_ruby_vm};
///
/// with_ruby_vm(|| unsafe {
///     let hello_world = with_gc_stress(|| unsafe {
///         let mut rstring = rb_sys::rb_utf8_str_new_cstr("hello world\0".as_ptr() as _);
///         let result = rb_sys::rb_string_value_cstr(&mut rstring);
///         std::ffi::CStr::from_ptr(result).to_string_lossy().into_owned()
///     });
///
///    assert_eq!(hello_world, "hello world");
/// });
/// ```
pub fn with_gc_stress<T>(f: impl FnOnce() -> T + std::panic::UnwindSafe) -> T {
    unsafe {
        let stress_intern = rb_sys::rb_intern("stress\0".as_ptr() as _);
        let stress_eq_intern = rb_sys::rb_intern("stress=\0".as_ptr() as _);
        let gc_module =
            rb_sys::rb_const_get(rb_sys::rb_cObject, rb_sys::rb_intern("GC\0".as_ptr() as _));

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

#[derive(Debug)]
struct RubyTestExecutor {
    executor: Mutex<LocalExecutor<'static>>,
}

impl RubyTestExecutor {
    pub fn run_test<F, T>(&self, f: F) -> T
    where
        F: FnOnce() -> T + UnwindSafe,
    {
        // Make sure we don't panic while holding the lock.
        let result = {
            let local_ex = &self
                .executor
                .lock()
                .expect("failed to lock Ruby test executor");

            std::panic::catch_unwind(|| future::block_on(local_ex.run(async { f() })))
        };

        match result {
            Ok(result) => result,
            Err(err) => std::panic::resume_unwind(err),
        }
    }
}

impl Default for RubyTestExecutor {
    fn default() -> Self {
        Self {
            executor: Mutex::new(LocalExecutor::new()),
        }
    }
}

unsafe impl Sync for RubyTestExecutor {}
unsafe impl Send for RubyTestExecutor {}

pub use rb_sys_test_helpers_macros::*;
