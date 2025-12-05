//! Windows-specific configuration for cross-compilation.
//!
//! Windows targets (MinGW) require specific environment variables
//! and have different library requirements than Unix targets.

/// Configuration for Windows cross-compilation.
#[derive(Debug, Clone, Default)]
pub struct WindowsConfig;

impl WindowsConfig {
    /// Create a new Windows configuration.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    /// Get environment variables for Windows cross-compilation.
    ///
    /// These environment variables configure the build to avoid
    /// bundling Windows-specific libraries that Zig provides.
    pub fn env_vars() -> Vec<(String, String)> {
        vec![
            // Prevent winapi crate from bundling import libraries
            // Zig provides these through its Windows support
            ("WINAPI_NO_BUNDLED_LIBRARIES".to_string(), "1".to_string()),
        ]
    }

    /// Get additional CC/CXX arguments for Windows targets.
    ///
    /// Currently no additional arguments are needed as Zig handles
    /// Windows cross-compilation well out of the box.
    pub fn cc_args() -> Vec<String> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_config_env_vars() {
        let env_vars = WindowsConfig::env_vars();

        assert!(env_vars.contains(&("WINAPI_NO_BUNDLED_LIBRARIES".to_string(), "1".to_string())));
    }

    #[test]
    fn test_windows_config_cc_args() {
        let args = WindowsConfig::cc_args();
        assert!(args.is_empty());
    }
}
