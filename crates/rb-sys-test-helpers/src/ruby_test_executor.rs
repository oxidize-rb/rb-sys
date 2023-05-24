use std::panic;
use std::sync::mpsc::{self, SyncSender};
use std::sync::Once;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::once_cell::OnceCell;
#[cfg(ruby_gte_3_0)]
use rb_sys::rb_ext_ractor_safe;
use rb_sys::{
    rb_errinfo, rb_inspect, rb_protect, rb_set_errinfo, rb_string_value_cstr, ruby_exec_node,
    ruby_process_options, ruby_setup, Qnil, VALUE,
};

static mut GLOBAL_EXECUTOR: OnceCell<RubyTestExecutor> = OnceCell::new();

pub struct RubyTestExecutor {
    sender: Option<SyncSender<Box<dyn FnOnce() + Send>>>,
    handle: Option<JoinHandle<()>>,
    timeout: Duration,
}

impl RubyTestExecutor {
    pub fn start() -> Self {
        let (sender, receiver) = mpsc::sync_channel::<Box<dyn FnOnce() + Send>>(0);

        let handle = thread::spawn(move || {
            for closure in receiver {
                closure();
            }
        });

        let executor = Self {
            sender: Some(sender),
            handle: Some(handle),
            timeout: Duration::from_secs(5),
        };

        executor.run(|| {
            static INIT: Once = Once::new();

            INIT.call_once(|| unsafe {
                ruby_setup_ceremony();
            });
        });

        executor
    }

    pub fn set_test_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    pub fn shutdown(&mut self) {
        self.set_test_timeout(Duration::from_secs(3));

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
        let (result_sender, result_receiver) = mpsc::sync_channel(1);

        let closure = Box::new(move || {
            let result = panic::catch_unwind(panic::AssertUnwindSafe(f));
            result_sender
                .send(result)
                .expect("Failed to send Ruby test result to Rust test thread");
        });

        if let Some(sender) = self.sender.as_ref() {
            sender
                .send(closure)
                .expect("Failed to send closure to Ruby test thread");
        } else {
            panic!("RubyTestExecutor is not running");
        }

        match result_receiver.recv_timeout(self.timeout) {
            Ok(Ok(result)) => result,
            Ok(Err(err)) => std::panic::resume_unwind(err),
            Err(_err) => {
                eprintln!("Ruby test timed out after {:?}", self.timeout);
                std::process::abort();
            }
        }
    }

    pub fn run_test<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        self.run(f)
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
    trick_the_linker();

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

        let msg = std::ffi::CStr::from_ptr(msg).to_string_lossy().into_owned();
        rb_set_errinfo(Qnil as _);
        panic!("Failed to process Ruby options: {}", msg);
    }

    match ruby_exec_node(node as _) {
        0 => {}
        code => panic!("Failed to execute Ruby (error code: {})", code),
    };
}

fn trick_the_linker() {
    // Force the compiler to not optimize out rb_ext_ractor_safe...
    #[cfg(ruby_gte_3_0)]
    {
        #[allow(clippy::cmp_null)]
        let ensure_ractor_safe = rb_ext_ractor_safe as *const () != std::ptr::null();
        assert!(ensure_ractor_safe);
    }
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

            executor.run_test(|| {
                unsafe { ruby_vm_at_exit(Some(set_called))}
            });

            drop(executor);

            assert_eq!(Some("hell yeah it was"), unsafe { RUBY_VM_AT_EXIT_CALLED });
        }
    }

    rusty_fork_test! {
        #[test]
        #[should_panic]
        fn test_timeout() {
            let mut executor = RubyTestExecutor::start();
            executor.set_test_timeout(Duration::from_millis(1));

            executor.run_test(|| {
                std::thread::sleep(Duration::from_millis(100));
            });
        }
    }
}
