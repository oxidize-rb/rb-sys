/// Splits shell words.
pub fn shellsplit(s: &str) -> Vec<String> {
    match shell_words::split(s) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("shellsplit failed: {}", e);
            s.split_whitespace().map(|s| s.to_string()).collect()
        }
    }
}

/// Memoizes the result of an expression.
#[macro_export]
macro_rules! memoize {
    ($type:ty: $val:expr) => {{
        static INIT: std::sync::Once = std::sync::Once::new();
        static mut VALUE: Option<$type> = None;
        unsafe {
            INIT.call_once(|| {
                VALUE = Some($val);
            });
            VALUE.as_ref().unwrap()
        }
    }};
}
