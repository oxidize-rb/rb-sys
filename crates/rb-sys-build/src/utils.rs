use crate::debug_log;

/// Check if current platform is mswin.
pub fn is_msvc() -> bool {
    if let Ok(target) = std::env::var("TARGET") {
        target.contains("msvc")
    } else {
        false
    }
}

/// Check if current platform is mswin or mingw.
pub fn is_mswin_or_mingw() -> bool {
    if let Ok(target) = std::env::var("TARGET") {
        target.contains("msvc") || target.contains("pc-windows-gnu")
    } else {
        false
    }
}

/// Check if we are in debug mode.
pub fn is_debug_env() -> bool {
    if std::env::var("CARGO_CFG_FEATURE")
        .map(|ft| ft.contains("clippy"))
        .unwrap_or(false)
    {
        return false;
    }

    let vars_to_check = ["DEBUG", "RB_SYS_DEBUG", "RB_SYS_DEBUG_BUILD"];

    vars_to_check.iter().any(|var| {
        std::env::var(var)
            .map(|v| v == "1" || v == "true")
            .unwrap_or(false)
    })
}

/// Splits shell words.
pub fn shellsplit<S: AsRef<str>>(s: S) -> Vec<String> {
    let s = s.as_ref();
    match shell_words::split(s) {
        Ok(v) => v,
        Err(e) => {
            debug_log!("shellsplit failed: {}", e);
            s.split_whitespace().map(Into::into).collect()
        }
    }
}

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

#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        eprintln!($($arg)*);

        if $crate::utils::is_debug_env() {
            use std::io::Write;

            let dir = if let Ok(dir) = std::env::var("DEBUG_OUTPUT_DIR") {
                std::path::PathBuf::from(dir)
            } else {
                std::env::var("CARGO_MANIFEST_DIR")
                    .map(|dir| std::path::PathBuf::from(dir))
                    .unwrap_or_else(|_| std::env::current_dir().unwrap())
            };

            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(dir.join("rb-sys-build.log"))
                .unwrap();

            let _ = writeln!(file, $($arg)*);
        }
    };
}
