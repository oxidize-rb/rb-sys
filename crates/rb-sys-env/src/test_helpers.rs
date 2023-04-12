//! Helpers for testing Ruby from Rust.
//!
//! ## Usage
//!
//! In your `Cargo.toml`:
//!
//! ```toml
//! [dev-dependencies]
//! rb-sys = { version = "0.9", features = ["link-ruby"] }
//! rb-sys-env = { version = "0.1", features = ["test-helpers"] }
//! ```
//!
//! In your `tests/*.rs`:
//!
//! ```rust
//! use rb_sys_env::test_helpers::with_ruby_vm;
//!
//! #[test]
//! fn test_something() {
//!     with_ruby_vm(|| {
//!         // Your test code that needs Ruby goes here...
//!     });
//! }
//! ```

use async_executor::LocalExecutor;
use futures_lite::future;

/// Initializes a Ruby VM, and ensures all tests are run by the same thread and
/// that the Ruby VM is initialized from.
///
/// ### Example
///
/// ```
/// use rb_sys_env::test_helpers::with_ruby_vm;
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
pub fn with_ruby_vm<T>(f: impl FnOnce() -> T) -> T {
    static INIT: std::sync::Once = std::sync::Once::new();
    static mut EXECUTOR: Option<RubyTestExecutor> = None;

    unsafe {
        INIT.call_once(|| {
            vm_init();
            EXECUTOR = Some(RubyTestExecutor::default());
        });
    }

    unsafe { EXECUTOR.as_ref() }.unwrap().run_test(f)
}

fn vm_init() {
    unsafe {
        let var_in_stack_frame = std::mem::zeroed();
        let argv: [*mut std::os::raw::c_char; 0] = [];
        let argv = argv.as_ptr();
        let mut argc = 0;

        rb_sys::ruby_init_stack(var_in_stack_frame);
        rb_sys::ruby_sysinit(&mut argc, argv as _);
        rb_sys::ruby_init();
    }
}

#[derive(Debug)]
struct RubyTestExecutor {
    executor: LocalExecutor<'static>,
}

impl RubyTestExecutor {
    pub fn run_test<T>(&self, f: impl FnOnce() -> T) -> T {
        let local_ex = &self.executor;

        future::block_on(local_ex.run(async { f() }))
    }
}

impl Default for RubyTestExecutor {
    fn default() -> Self {
        Self {
            executor: LocalExecutor::new(),
        }
    }
}

unsafe impl Sync for RubyTestExecutor {}
unsafe impl Send for RubyTestExecutor {}
