//! Environment variable setup for Cargo cross-compilation.
//!
//! This module generates the environment variables needed for Cargo
//! to use the Zig compiler shims for cross-compilation.

use std::collections::HashMap;
use std::path::Path;

use super::shim::ShimPaths;
use super::target::{Os, RustTarget};
use crate::platform::WindowsConfig;

/// Generate all environment variables needed for Cargo cross-compilation.
///
/// This sets up:
/// - CC/CXX/AR paths for the target
/// - Cargo linker configuration
/// - Platform-specific environment variables
/// - Cross-compilation signals
pub fn cargo_env(
    target: &RustTarget,
    shim_dir: &Path,
    sysroot: Option<&Path>,
) -> HashMap<String, String> {
    let mut env = HashMap::new();
    let shim_paths = ShimPaths::new(shim_dir);

    // Convert target triple to environment variable format
    let triple_underscore = target.raw.replace('-', "_");
    let triple_upper = triple_underscore.to_uppercase();

    // === Compiler paths (target-specific) ===
    // cc-rs looks for CC_{target} with underscores
    env.insert(
        format!("CC_{}", triple_underscore),
        shim_paths.cc.display().to_string(),
    );
    env.insert(
        format!("CXX_{}", triple_underscore),
        shim_paths.cxx.display().to_string(),
    );
    env.insert(
        format!("AR_{}", triple_underscore),
        shim_paths.ar.display().to_string(),
    );

    // === Cargo linker configuration ===
    // Use the dedicated ld shim for linking (not cc)
    env.insert(
        format!("CARGO_TARGET_{}_LINKER", triple_upper),
        shim_paths.ld.display().to_string(),
    );

    // === Cross-compilation signals ===
    env.insert("RB_SYS_CROSS_COMPILING".to_string(), "1".to_string());
    env.insert("CRATE_CC_NO_DEFAULTS".to_string(), "1".to_string());

    // === Platform-specific environment variables ===
    match target.os {
        Os::Linux => {
            // Bindgen sysroot configuration
            if let Some(sysroot) = sysroot {
                env.insert(
                    format!("BINDGEN_EXTRA_CLANG_ARGS_{}", triple_underscore),
                    format!("--sysroot={}", sysroot.display()),
                );
            }
        }
        Os::Darwin => {
            // PKG_CONFIG sysroot (SDKROOT is read directly by MacOSConfig)
            if let Ok(sdkroot) = std::env::var("SDKROOT") {
                env.insert("PKG_CONFIG_SYSROOT_DIR".to_string(), sdkroot);
            }
        }
        Os::Windows => {
            // Windows-specific environment variables
            for (key, value) in WindowsConfig::env_vars() {
                env.insert(key, value);
            }
        }
    }

    env
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_cargo_env_linux() {
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        let shim_dir = PathBuf::from("/tmp/shims");
        let sysroot = PathBuf::from("/path/to/sysroot");

        let env = cargo_env(&target, &shim_dir, Some(&sysroot));

        // Check compiler paths
        assert_eq!(
            env.get("CC_x86_64_unknown_linux_gnu"),
            Some(&"/tmp/shims/cc".to_string())
        );
        assert_eq!(
            env.get("CXX_x86_64_unknown_linux_gnu"),
            Some(&"/tmp/shims/c++".to_string())
        );
        assert_eq!(
            env.get("AR_x86_64_unknown_linux_gnu"),
            Some(&"/tmp/shims/ar".to_string())
        );

        // Check linker (uses dedicated ld shim, not cc)
        assert_eq!(
            env.get("CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER"),
            Some(&"/tmp/shims/ld".to_string())
        );

        // Check cross-compilation signals
        assert_eq!(env.get("RB_SYS_CROSS_COMPILING"), Some(&"1".to_string()));
        assert_eq!(env.get("CRATE_CC_NO_DEFAULTS"), Some(&"1".to_string()));

        // Check bindgen args
        assert_eq!(
            env.get("BINDGEN_EXTRA_CLANG_ARGS_x86_64_unknown_linux_gnu"),
            Some(&"--sysroot=/path/to/sysroot".to_string())
        );
    }

    #[test]
    fn test_cargo_env_windows() {
        let target = RustTarget::parse("x86_64-pc-windows-gnu").unwrap();
        let shim_dir = PathBuf::from("/tmp/shims");

        let env = cargo_env(&target, &shim_dir, None);

        // Check Windows-specific env vars
        assert_eq!(
            env.get("WINAPI_NO_BUNDLED_LIBRARIES"),
            Some(&"1".to_string())
        );

        // Check compiler paths use correct format
        assert_eq!(
            env.get("CC_x86_64_pc_windows_gnu"),
            Some(&"/tmp/shims/cc".to_string())
        );
    }

    #[test]
    fn test_cargo_env_macos() {
        let target = RustTarget::parse("aarch64-apple-darwin").unwrap();
        let shim_dir = PathBuf::from("/tmp/shims");

        let env = cargo_env(&target, &shim_dir, None);

        // Check compiler paths
        assert_eq!(
            env.get("CC_aarch64_apple_darwin"),
            Some(&"/tmp/shims/cc".to_string())
        );

        // macOS doesn't have sysroot in env (uses SDKROOT directly)
        assert!(!env.contains_key("BINDGEN_EXTRA_CLANG_ARGS_aarch64_apple_darwin"));
    }

    #[test]
    fn test_cargo_env_musl() {
        let target = RustTarget::parse("x86_64-unknown-linux-musl").unwrap();
        let shim_dir = PathBuf::from("/tmp/shims");
        let sysroot = PathBuf::from("/musl/sysroot");

        let env = cargo_env(&target, &shim_dir, Some(&sysroot));

        assert_eq!(
            env.get("CC_x86_64_unknown_linux_musl"),
            Some(&"/tmp/shims/cc".to_string())
        );
        assert_eq!(
            env.get("BINDGEN_EXTRA_CLANG_ARGS_x86_64_unknown_linux_musl"),
            Some(&"--sysroot=/musl/sysroot".to_string())
        );
    }
}
