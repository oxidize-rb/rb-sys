/// Check if current platform is mswin.
pub fn is_msvc() -> bool {
    std::env::var("TARGET").unwrap().contains("msvc")
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
