/// Memoizes the result [`rb_sys::VALUE`] of the given expression.
#[macro_export]
macro_rules! memoized {
    ($e:expr) => {{
        pub static INIT: std::sync::Once = std::sync::Once::new();
        pub static mut MEMOIZED_VAL: Option<rb_sys::VALUE> = None;

        INIT.call_once(|| unsafe {
            MEMOIZED_VAL.replace($e);
        });

        unsafe { *MEMOIZED_VAL.as_ref().unwrap() }
    }};
}

/// Creates a new Ruby string from a Rust string.
#[macro_export]
macro_rules! rstring {
    ($s:expr) => {
        unsafe { rb_sys::rb_utf8_str_new($s.as_ptr() as _, $s.len() as _) }
    };
}

/// Creates a new Ruby symbol from a Rust literal str.
#[macro_export]
macro_rules! rsymbol {
    ($s:literal) => {
        unsafe { rb_sys::rb_id2sym(rb_sys::rb_intern(concat!($s, "\0").as_ptr() as _)) }
    };
}

/// Captures the GC stat before and after the expression.
#[macro_export]
macro_rules! capture_gc_stat_for {
    ($id:literal, $e:expr) => {{
        let id = $crate::memoized! { $crate::rsymbol!($id) };

        unsafe {
            $crate::trigger_full_gc!();

            let before = unsafe { rb_sys::rb_gc_stat(id) };
            let result = $e;
            let after = unsafe { rb_sys::rb_gc_stat(id) };

            (result, after as isize - before as isize)
        }
    }};
}

/// Allows you to convert a Ruby string to a Rust string.
#[macro_export]
macro_rules! rstring_to_string {
    ($v:expr) => {{
        let cstr = rb_sys::rb_string_value_cstr(&mut $v);

        std::ffi::CStr::from_ptr(cstr)
            .to_string_lossy()
            .into_owned()
    }};
}

/// This is a macro that allows you to call a method on a Ruby object, and get
/// an `Option` back. If the type matches, it will return `Some`, otherwise it
/// will return `None`.
#[macro_export]
macro_rules! rb_funcall_typed {
    ($v:expr, $m:expr, $args:expr, $t:expr) => {{
        {
            let args: &mut [rb_sys::VALUE] = &mut $args[..];
            let id = rb_sys::rb_intern(concat!($m, "\0").as_ptr() as _);
            let argv = $args.as_ptr();
            let result = rb_sys::rb_check_funcall($v, id, args.len() as _, argv);

            if RB_TYPE_P(result) != $t as _ {
                None
            } else {
                Some(result)
            }
        }
    }};
}

/// Runs the garbage collector 10 times to ensure that we have a clean slate.
#[macro_export]
macro_rules! trigger_full_gc {
    () => {
        let cmd = "GC.start(full_mark: false, immediate_sweep: false)\0".as_ptr() as *const _;

        for _ in 0..20 {
            unsafe { rb_sys::rb_eval_string(cmd) };
            std::thread::yield_now();
        }
    };
}
