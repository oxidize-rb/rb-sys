#[macro_export]
macro_rules! rstring {
    ($s:expr) => {
        unsafe { rb_sys::rb_str_new($s.as_ptr() as _, $s.len() as _) }
    };
}
