//! Zig libc include path discovery for cross-compilation.
//!
//! This module provides utilities to query zig for libc include paths
//! for a given target. These paths are used to configure bindgen so it
//! can find standard C headers (stdio.h, stdlib.h, etc.) when cross-compiling.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Get libc include directories from zig for a given Rust target.
///
/// Runs `zig libc -target <zig_target> -includes` and parses the output.
/// Returns a list of include paths (one per line from zig output).
///
/// # Arguments
/// * `zig_path` - Path to the zig executable
/// * `rust_target` - Rust target triple (e.g., "x86_64-unknown-linux-gnu")
///
/// # Returns
/// A vector of PathBuf include directories, in the order zig returns them.
///
/// # Errors
/// Returns an error if:
/// - The target cannot be mapped to a zig target
/// - zig command fails to execute
/// - zig libc returns an error
pub fn get_zig_libc_includes(zig_path: &Path, rust_target: &str) -> Result<Vec<PathBuf>> {
    let zig_target = rust_target_to_zig_libc_target(rust_target)?;

    tracing::debug!(
        rust_target = %rust_target,
        zig_target = %zig_target,
        "Querying zig libc includes"
    );

    let output = Command::new(zig_path)
        .args(["libc", "-target", &zig_target, "-includes"])
        .output()
        .with_context(|| {
            format!("Failed to run `zig libc -target {zig_target} -includes`. Is zig installed?")
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "zig libc -target {} -includes failed:\n{}",
            zig_target,
            stderr.trim()
        );
    }

    let paths: Vec<PathBuf> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| PathBuf::from(line.trim()))
        .collect();

    if paths.is_empty() {
        bail!("zig libc -target {zig_target} -includes returned no paths");
    }

    tracing::debug!(
        count = paths.len(),
        paths = ?paths,
        "Got zig libc include paths"
    );

    Ok(paths)
}

/// Check if a target requires zig libc includes.
///
/// Darwin targets use SDKROOT instead of zig libc.
pub fn requires_zig_libc(rust_target: &str) -> bool {
    // Darwin targets use macOS SDK, not zig libc
    !rust_target.contains("darwin")
}

/// Check if a target requires SDKROOT (macOS SDK).
pub fn requires_sdkroot(rust_target: &str) -> bool {
    rust_target.contains("darwin")
}

/// Convert a Rust target triple to zig libc target format.
///
/// Zig uses a simplified target format for libc queries:
/// - Linux: `<arch>-linux-<abi>` (e.g., x86_64-linux-gnu)
/// - Windows: `<arch>-windows-<abi>` (e.g., x86_64-windows-gnu)
/// - macOS: `<arch>-macos` (e.g., aarch64-macos)
fn rust_target_to_zig_libc_target(rust_target: &str) -> Result<String> {
    let parts: Vec<&str> = rust_target.split('-').collect();

    match parts.as_slice() {
        // i686-unknown-linux-gnu -> x86-linux-gnu (zig uses x86 not i686)
        // Must come before generic linux pattern
        ["i686", _, "linux", abi] => Ok(format!("x86-linux-{abi}")),

        // i686-pc-windows-gnu -> x86-windows-gnu
        ["i686", _, "windows", "gnu"] => Ok("x86-windows-gnu".to_string()),

        // x86_64-unknown-linux-gnu -> x86_64-linux-gnu
        // aarch64-unknown-linux-gnu -> aarch64-linux-gnu
        // aarch64-unknown-linux-musl -> aarch64-linux-musl
        // arm-unknown-linux-gnueabihf -> arm-linux-gnueabihf
        [arch, _, "linux", abi] => Ok(format!("{arch}-linux-{abi}")),

        // x86_64-pc-windows-gnu -> x86_64-windows-gnu
        // x86_64-pc-windows-msvc is not supported (zig doesn't provide msvc libc)
        [arch, _, "windows", "gnu"] => Ok(format!("{arch}-windows-gnu")),

        // aarch64-apple-darwin -> aarch64-macos
        // x86_64-apple-darwin -> x86_64-macos
        [arch, "apple", "darwin"] => Ok(format!("{arch}-macos")),

        _ => bail!(
            "Unsupported target for zig libc: {rust_target}\n\
             Supported patterns:\n  \
             - <arch>-unknown-linux-<abi> (e.g., x86_64-unknown-linux-gnu)\n  \
             - <arch>-pc-windows-gnu (e.g., x86_64-pc-windows-gnu)\n  \
             - <arch>-apple-darwin (e.g., aarch64-apple-darwin)"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_target_to_zig_libc_target_linux_gnu() {
        assert_eq!(
            rust_target_to_zig_libc_target("x86_64-unknown-linux-gnu").unwrap(),
            "x86_64-linux-gnu"
        );
        assert_eq!(
            rust_target_to_zig_libc_target("aarch64-unknown-linux-gnu").unwrap(),
            "aarch64-linux-gnu"
        );
        assert_eq!(
            rust_target_to_zig_libc_target("arm-unknown-linux-gnueabihf").unwrap(),
            "arm-linux-gnueabihf"
        );
    }

    #[test]
    fn test_rust_target_to_zig_libc_target_linux_musl() {
        assert_eq!(
            rust_target_to_zig_libc_target("x86_64-unknown-linux-musl").unwrap(),
            "x86_64-linux-musl"
        );
        assert_eq!(
            rust_target_to_zig_libc_target("aarch64-unknown-linux-musl").unwrap(),
            "aarch64-linux-musl"
        );
    }

    #[test]
    fn test_rust_target_to_zig_libc_target_windows() {
        assert_eq!(
            rust_target_to_zig_libc_target("x86_64-pc-windows-gnu").unwrap(),
            "x86_64-windows-gnu"
        );
    }

    #[test]
    fn test_rust_target_to_zig_libc_target_darwin() {
        assert_eq!(
            rust_target_to_zig_libc_target("aarch64-apple-darwin").unwrap(),
            "aarch64-macos"
        );
        assert_eq!(
            rust_target_to_zig_libc_target("x86_64-apple-darwin").unwrap(),
            "x86_64-macos"
        );
    }

    #[test]
    fn test_rust_target_to_zig_libc_target_i686() {
        assert_eq!(
            rust_target_to_zig_libc_target("i686-unknown-linux-gnu").unwrap(),
            "x86-linux-gnu"
        );
        assert_eq!(
            rust_target_to_zig_libc_target("i686-pc-windows-gnu").unwrap(),
            "x86-windows-gnu"
        );
    }

    #[test]
    fn test_rust_target_to_zig_libc_target_unsupported() {
        assert!(rust_target_to_zig_libc_target("wasm32-unknown-unknown").is_err());
        assert!(rust_target_to_zig_libc_target("x86_64-pc-windows-msvc").is_err());
    }

    #[test]
    fn test_requires_zig_libc() {
        assert!(requires_zig_libc("x86_64-unknown-linux-gnu"));
        assert!(requires_zig_libc("aarch64-unknown-linux-musl"));
        assert!(requires_zig_libc("x86_64-pc-windows-gnu"));
        assert!(!requires_zig_libc("aarch64-apple-darwin"));
        assert!(!requires_zig_libc("x86_64-apple-darwin"));
    }

    #[test]
    fn test_requires_sdkroot() {
        assert!(!requires_sdkroot("x86_64-unknown-linux-gnu"));
        assert!(requires_sdkroot("aarch64-apple-darwin"));
        assert!(requires_sdkroot("x86_64-apple-darwin"));
    }
}
