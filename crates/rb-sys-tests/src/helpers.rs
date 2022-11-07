#[macro_export]
macro_rules! rstring {
    ($s:expr) => {
        unsafe { rb_sys::rb_str_new($s.as_ptr() as _, $s.len() as _) }
    };
}

#[macro_export]
macro_rules! rstring_to_string {
    ($v:expr) => {
        unsafe {
            let cstr = rb_sys::rb_string_value_cstr(&mut $v);

            std::ffi::CStr::from_ptr(cstr)
                .to_string_lossy()
                .into_owned()
        }
    };
}
