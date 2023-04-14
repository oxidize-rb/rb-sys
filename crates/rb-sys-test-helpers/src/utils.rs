/// Creates a new Ruby string from a Rust string.
#[macro_export]
macro_rules! rstring {
    ($s:expr) => {
        unsafe { rb_sys::rb_utf8_str_new($s.as_ptr() as _, $s.len() as _) }
    };
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
            let args: &mut [rb_sys::VALUE] = $args.as_mut_slice();
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
