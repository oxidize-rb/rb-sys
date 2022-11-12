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
