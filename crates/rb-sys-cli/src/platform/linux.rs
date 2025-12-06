//! Linux-specific configuration for cross-compilation.
//!
//! Linux targets require a sysroot containing headers and libraries
//! from the target system. This module handles sysroot configuration
//! and platform-specific compiler flags.

use crate::zig::target::{Env, RustTarget};
use std::path::PathBuf;

/// Configuration for Linux cross-compilation.
#[derive(Debug, Clone)]
pub struct LinuxConfig {
    /// Path to the sysroot directory
    pub sysroot: PathBuf,
    /// GNU triple for architecture-specific paths (e.g., "x86_64-linux-gnu")
    pub gnu_triple: String,
    /// Whether this is a musl target
    pub is_musl: bool,
}

impl LinuxConfig {
    /// Create a new Linux configuration from a target and sysroot path.
    pub fn new(target: &RustTarget, sysroot: PathBuf) -> Self {
        Self {
            sysroot,
            gnu_triple: target.gnu_triple(),
            is_musl: target.env == Env::Musl,
        }
    }

    /// Get additional CC/CXX arguments for Linux targets.
    ///
    /// These arguments configure the compiler to use the sysroot
    /// for headers and libraries instead of the host system.
    pub fn cc_args(&self) -> Vec<String> {
        let mut args = vec![
            // Prevent host headers from being used
            "-nostdinc".to_string(),
            // Set the sysroot for library and header lookup
            format!("--sysroot={}", self.sysroot.display()),
        ];

        // Add architecture-specific include path first (higher priority)
        let arch_include = self.sysroot.join("usr/include").join(&self.gnu_triple);
        if arch_include.exists() {
            args.push("-isystem".to_string());
            args.push(arch_include.display().to_string());
        }

        // Add generic include path
        let generic_include = self.sysroot.join("usr/include");
        if generic_include.exists() {
            args.push("-isystem".to_string());
            args.push(generic_include.display().to_string());
        }

        args
    }

    /// Get additional defines for musl targets.
    ///
    /// musl libc requires specific preprocessor definitions for
    /// compatibility with C++ standard library headers.
    pub fn musl_defines() -> Vec<String> {
        vec![
            "-D_LIBCPP_HAS_MUSL_LIBC".to_string(),
            "-D_LARGEFILE64_SOURCE".to_string(),
        ]
    }

    /// Check if the sysroot has the expected structure.
    ///
    /// Returns an error message if the sysroot is missing expected directories.
    /// Note: usr/include is no longer required since we use zig libc for headers.
    pub fn validate(&self) -> Result<(), String> {
        if !self.sysroot.exists() {
            return Err(format!(
                "Sysroot directory does not exist: {}",
                self.sysroot.display()
            ));
        }

        // Note: We no longer require usr/include in the sysroot because:
        // - Basic C headers (stdio.h, etc.) come from zig libc
        // - The sysroot now only needs to contain additional libs like OpenSSL, zlib
        // - Ruby headers come from the extracted rubies directory (not the sysroot)

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_config_cc_args() {
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        let config = LinuxConfig::new(&target, PathBuf::from("/fake/sysroot"));

        let args = config.cc_args();

        assert!(args.contains(&"-nostdinc".to_string()));
        assert!(args.contains(&"--sysroot=/fake/sysroot".to_string()));
    }

    #[test]
    fn test_linux_config_musl() {
        let target = RustTarget::parse("x86_64-unknown-linux-musl").unwrap();
        let config = LinuxConfig::new(&target, PathBuf::from("/fake/sysroot"));

        assert!(config.is_musl);
        assert_eq!(config.gnu_triple, "x86_64-linux-musl");
    }

    #[test]
    fn test_linux_config_glibc() {
        let target = RustTarget::parse("aarch64-unknown-linux-gnu").unwrap();
        let config = LinuxConfig::new(&target, PathBuf::from("/fake/sysroot"));

        assert!(!config.is_musl);
        assert_eq!(config.gnu_triple, "aarch64-linux-gnu");
    }

    #[test]
    fn test_musl_defines() {
        let defines = LinuxConfig::musl_defines();

        assert!(defines.contains(&"-D_LIBCPP_HAS_MUSL_LIBC".to_string()));
        assert!(defines.contains(&"-D_LARGEFILE64_SOURCE".to_string()));
    }

    #[test]
    fn test_validate_missing_sysroot() {
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        let config = LinuxConfig::new(&target, PathBuf::from("/nonexistent/path"));

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }
}
