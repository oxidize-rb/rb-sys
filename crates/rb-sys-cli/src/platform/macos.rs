//! macOS-specific configuration for cross-compilation.
//!
//! macOS targets require the macOS SDK (SDKROOT) for cross-compilation.
//! This module handles SDK path configuration and macOS-specific compiler flags.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

/// Configuration for macOS cross-compilation.
#[derive(Debug, Clone)]
pub struct MacOSConfig {
    /// Path to the macOS SDK root
    pub sdkroot: PathBuf,
}

impl MacOSConfig {
    /// Create a new macOS configuration from the SDKROOT environment variable
    /// or fall back to an embedded SDK.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - SDKROOT environment variable is not set and no embedded SDK is found
    /// - The SDKROOT path does not exist
    pub fn from_env_or_embedded(embedded_sdk: Option<&Path>) -> Result<Self> {
        // First try SDKROOT environment variable
        if let Ok(sdkroot_str) = std::env::var("SDKROOT") {
            let sdkroot = PathBuf::from(&sdkroot_str);
            if sdkroot.exists() {
                return Ok(Self { sdkroot });
            } else {
                tracing::warn!(
                    "SDKROOT path does not exist: {}, falling back to embedded SDK",
                    sdkroot.display()
                );
            }
        }

        // Fall back to embedded SDK
        if let Some(embedded_sdk_path) = embedded_sdk {
            if embedded_sdk_path.exists() {
                tracing::debug!(
                    sdk = %embedded_sdk_path.display(),
                    "Using embedded macOS SDK"
                );
                return Ok(Self {
                    sdkroot: embedded_sdk_path.to_path_buf(),
                });
            } else {
                tracing::warn!(
                    "Embedded macOS SDK path does not exist: {}",
                    embedded_sdk_path.display()
                );
            }
        }

        // Try to find embedded SDK in cache directory
        if let Some(cache_sdk) = Self::find_embedded_sdk_in_cache()? {
            tracing::debug!(
                sdk = %cache_sdk.display(),
                "Using cached embedded macOS SDK"
            );
            return Ok(Self { sdkroot: cache_sdk });
        }

        // No SDK available
        bail!(
            "macOS SDK is required for macOS cross-compilation.\n\n\
             Either:\n\
             1. Set the SDKROOT environment variable to a valid macOS SDK path, or\n\
             2. Ensure rb-sys-cli has embedded macOS SDK assets\n\n\
             You can obtain a macOS SDK from Xcode or from:\n\
             https://github.com/joseluisq/macosx-sdks"
        );
    }

    /// Try to find an embedded macOS SDK in the cache directory
    fn find_embedded_sdk_in_cache() -> Result<Option<PathBuf>> {
        // Try common macOS platforms
        let platforms = ["arm64-darwin", "x86_64-darwin"];

        for platform in &platforms {
            let cache_dir = dirs::cache_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not determine cache directory"))?
                .join("rb-sys/cli")
                .join(platform)
                .join("sdk");

            if cache_dir.exists() {
                // Find the latest SDK in this directory
                let entries = std::fs::read_dir(&cache_dir).with_context(|| {
                    format!(
                        "Failed to read SDK cache directory: {}",
                        cache_dir.display()
                    )
                })?;

                let mut sdk_versions = Vec::new();
                for entry in entries {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if name.starts_with("MacOSX") && name.ends_with(".sdk") {
                            sdk_versions.push(path);
                        }
                    }
                }

                if !sdk_versions.is_empty() {
                    // Sort by version (latest first)
                    sdk_versions.sort_by(|a, b| {
                        let a_name = a.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        let b_name = b.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        let a_version = a_name
                            .strip_prefix("MacOSX")
                            .and_then(|s| s.strip_suffix(".sdk"))
                            .unwrap_or("");
                        let b_version = b_name
                            .strip_prefix("MacOSX")
                            .and_then(|s| s.strip_suffix(".sdk"))
                            .unwrap_or("");
                        b_version.cmp(a_version)
                    });

                    return Ok(Some(sdk_versions[0].clone()));
                }
            }
        }

        Ok(None)
    }

    /// Create a new macOS configuration from the SDKROOT environment variable.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - SDKROOT environment variable is not set
    /// - The SDKROOT path does not exist
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
