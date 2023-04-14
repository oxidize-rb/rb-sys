use std::panic;
use std::sync::mpsc::{self, Sender};
use std::sync::Once;
use std::thread::{self, JoinHandle};

#[cfg(ruby_gte_3_0)]
use rb_sys::rb_ext_ractor_safe;

use rb_sys::{
    rb_errinfo, rb_inspect, rb_protect, rb_set_errinfo, rb_string_value_cstr, ruby_exec_node,
    ruby_process_options, ruby_setup, Qnil, VALUE,
};

use crate::once_cell::OnceCell;

static mut GLOBAL_EXECUTOR: OnceCell<RubyTestExecutor> = OnceCell::new();

pub struct RubyTestExecutor {
    sender: Option<Sender<Box<dyn FnOnce() + Send>>>,
    handle: Option<JoinHandle<()>>,
}

impl RubyTestExecutor {
    pub fn start() -> Self {
        let (sender, receiver) = mpsc::channel::<Box<dyn FnOnce() + Send>>();

        // Spawn a new scoped thread
        let handle = thread::spawn(move || {
            static INIT: Once = Once::new();

            INIT.call_once(|| unsafe {
                ruby_setup_ceremony();
            });

            for closure in receiver {
                closure();
            }
        });

        Self {
            sender: Some(sender),
            handle: Some(handle),
        }
    }

    pub fn shutdown(&mut self) {
        self.run(|| unsafe {
            let ret = rb_sys::ruby_cleanup(0);

            if ret != 0 {
                panic!("Failed to cleanup Ruby (error code: {})", ret);
            }
        });

        if let Some(sender) = self.sender.take() {
            drop(sender);
        }

        if let Some(handle) = self.handle.take() {
            handle.join().expect("Failed to join executor thread");
        }
    }

    pub fn run<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        let (result_sender, result_receiver) = mpsc::channel();

        let closure = Box::new(move || {
            let result = panic::catch_unwind(panic::AssertUnwindSafe(f));
            result_sender.send(result).unwrap();
        });

        if let Some(sender) = &self.sender {
            sender.send(closure).unwrap();
        } else {
            panic!("RubyTestExecutor is not running");
        }

        // This code is pretty sketchy for Apple silicon for Cargo < 1.57
        // (without crossbeam). If you are running into issues, try upgrading
        // Rust.
        match result_receiver
            .recv()
            .expect("Failed to receive test result")
        {
            Ok(result) => result,
            Err(err) => std::panic::resume_unwind(err),
        }
    }
}

impl Drop for RubyTestExecutor {
    fn drop(&mut self) {
        self.shutdown();
    }
}

pub fn global_executor() -> &'static RubyTestExecutor {
    unsafe { &GLOBAL_EXECUTOR }.get_or_init(RubyTestExecutor::start)
}

unsafe fn ruby_setup_ceremony() {
    #[cfg(windows)]
    {
        let mut argc = 0;
        let mut argv: [*mut std::os::raw::c_char; 0] = [];
        let mut argv = argv.as_mut_ptr();
        rb_sys::rb_w32_sysinit(&mut argc, &mut argv);
    }

    match ruby_setup() {
        0 => {}
        code => panic!("Failed to setup Ruby (error code: {})", code),
    };

    unsafe extern "C" fn do_ruby_process_options(_: VALUE) -> VALUE {
        let mut argv: [*mut i8; 3] = [
            "ruby\0".as_ptr() as _,
            "-e\0".as_ptr() as _,
            "\0".as_ptr() as _,
        ];

        ruby_process_options(argv.len() as _, argv.as_mut_ptr() as _) as _
    }

    let mut protect_status = 0;

    let node = rb_protect(
        Some(do_ruby_process_options),
        Qnil as _,
        &mut protect_status as _,
    );

    if protect_status != 0 {
        let err = rb_errinfo();
        let mut msg = rb_inspect(err);
        let msg = rb_string_value_cstr(&mut msg);

        // Force the compiler to not optimize out rb_ext_ractor_safe...
        #[cfg(ruby_gte_3_0)]
        {
            #[allow(clippy::cmp_null)]
            let ensure_ractor_safe =
                rb_ext_ractor_safe as *const std::ffi::c_void != std::ptr::null();
            assert!(ensure_ractor_safe);
        }

        let msg = std::ffi::CStr::from_ptr(msg).to_string_lossy().into_owned();
        rb_set_errinfo(Qnil as _);
        panic!("Failed to process Ruby options: {}", msg);
    }

    match ruby_exec_node(node as _) {
        0 => {}
        code => panic!("Failed to execute Ruby (error code: {})", code),
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use rb_sys::ruby_vm_at_exit;
    use rusty_fork::rusty_fork_test;

    rusty_fork_test! {
        #[test]
        fn test_shutdown() {
            static mut RUBY_VM_AT_EXIT_CALLED: Option<&str> = None;

            let executor = RubyTestExecutor::start();

            unsafe extern "C" fn set_called(_: *mut rb_sys::ruby_vm_t) {
                RUBY_VM_AT_EXIT_CALLED = Some("hell yeah it was");
            }

            executor.run(|| {
                unsafe { ruby_vm_at_exit(Some(set_called))}
            });

            drop(executor);

            assert_eq!(Some("hell yeah it was"), unsafe { RUBY_VM_AT_EXIT_CALLED });
        }
    }
}
