//! macOS-specific configuration for cross-compilation.
//!
//! macOS targets require the macOS SDK (SDKROOT) for cross-compilation.
//! This module handles SDK path configuration and macOS-specific compiler flags.

use anyhow::{bail, Context, Result};
use std::path::PathBuf;

/// Configuration for macOS cross-compilation.
#[derive(Debug, Clone)]
pub struct MacOSConfig {
    /// Path to the macOS SDK root
    pub sdkroot: PathBuf,
}

impl MacOSConfig {
    /// Create a new macOS configuration from the SDKROOT environment variable.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - SDKROOT environment variable is not set
    /// - The SDKROOT path does not exist
    pub fn from_env() -> Result<Self> {
        let sdkroot = std::env::var("SDKROOT").context(
            "SDKROOT environment variable is required for macOS cross-compilation.\n\n\
             Set it to the path of your macOS SDK, for example:\n  \
             export SDKROOT=/path/to/MacOSX14.0.sdk\n\n\
             You can obtain the macOS SDK from Xcode or from:\n  \
             https://github.com/joseluisq/macosx-sdks",
        )?;

        let sdkroot = PathBuf::from(&sdkroot);
        if !sdkroot.exists() {
            bail!(
                "SDKROOT path does not exist: {}\n\n\
                 Please ensure the path points to a valid macOS SDK directory.",
                sdkroot.display()
            );
        }

        Ok(Self { sdkroot })
    }

    /// Get additional CC/CXX arguments for macOS targets.
    ///
    /// These arguments configure the compiler to use the SDK
    /// for headers, libraries, and frameworks.
    pub fn cc_args(&self) -> Vec<String> {
        vec![
            // Set the sysroot to the SDK
            format!("--sysroot={}", self.sdkroot.display()),
            // Add SDK library paths
            format!("-L{}/usr/lib", self.sdkroot.display()),
            // Add framework search path
            format!(
                "-iframework{}/System/Library/Frameworks",
                self.sdkroot.display()
            ),
            // Ensure we're not building for iOS
            "-DTARGET_OS_IPHONE=0".to_string(),
        ]
    }

    /// Get environment variables for macOS cross-compilation.
    #[allow(dead_code)]
    pub fn env_vars(&self) -> Vec<(String, String)> {
        vec![(
            "PKG_CONFIG_SYSROOT_DIR".to_string(),
            self.sdkroot.display().to_string(),
        )]
    }

    /// Check if the SDK has the expected structure.
    ///
    /// Returns an error message if the SDK is missing expected directories.
    pub fn validate(&self) -> Result<(), String> {
        let usr_include = self.sdkroot.join("usr/include");
        if !usr_include.exists() {
            return Err(format!(
                "macOS SDK is missing usr/include directory: {}\n\
                 This may not be a valid macOS SDK.",
                usr_include.display()
            ));
        }

        let frameworks = self.sdkroot.join("System/Library/Frameworks");
        if !frameworks.exists() {
            return Err(format!(
                "macOS SDK is missing System/Library/Frameworks directory: {}\n\
                 This may not be a valid macOS SDK.",
                frameworks.display()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_config_cc_args() {
        let config = MacOSConfig {
            sdkroot: PathBuf::from("/fake/MacOSX14.0.sdk"),
        };

        let args = config.cc_args();

        assert!(args.contains(&"--sysroot=/fake/MacOSX14.0.sdk".to_string()));
        assert!(args.contains(&"-L/fake/MacOSX14.0.sdk/usr/lib".to_string()));
        assert!(
            args.contains(&"-iframework/fake/MacOSX14.0.sdk/System/Library/Frameworks".to_string())
        );
        assert!(args.contains(&"-DTARGET_OS_IPHONE=0".to_string()));
    }

    #[test]
    fn test_macos_config_env_vars() {
        let config = MacOSConfig {
            sdkroot: PathBuf::from("/fake/MacOSX14.0.sdk"),
        };

        let env_vars = config.env_vars();

        assert!(env_vars.contains(&(
            "PKG_CONFIG_SYSROOT_DIR".to_string(),
            "/fake/MacOSX14.0.sdk".to_string()
        )));
    }

    #[test]
    fn test_from_env_missing() {
        // Temporarily unset SDKROOT if it exists
        let original = std::env::var("SDKROOT").ok();
        std::env::remove_var("SDKROOT");

        let result = MacOSConfig::from_env();
        assert!(result.is_err());

        // Restore original value
        if let Some(val) = original {
            std::env::set_var("SDKROOT", val);
        }
    }
}
