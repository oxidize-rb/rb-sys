//! CPU feature flag selection for Zig cross-compilation.
//!
//! Different architectures require specific CPU feature flags to generate
//! correct code. This module provides the `-mcpu` flag values for each
//! supported target.

use super::target::{Arch, Env, Os, RustTarget};

/// Get the `-mcpu` flag value for a target, if one is needed.
///
/// Most targets don't need an explicit CPU flag, but ARM targets require
/// specific feature sets to be enabled for correct code generation.
///
/// # Returns
///
/// - `Some(cpu_string)` if the target needs a `-mcpu` flag
/// - `None` if no CPU flag is needed
pub fn cpu_flag(target: &RustTarget) -> Option<&'static str> {
    match (target.arch, target.os, target.env) {
        // ARM hard-float Linux needs specific CPU features:
        // - generic: base ARM architecture
        // - +v6: ARMv6 instructions
        // - +strict_align: require aligned memory access
        // - +vfp2-d32: VFPv2 with 32 double-precision registers (for hard-float)
        (Arch::Arm, Os::Linux, Env::Gnueabihf) => Some("generic+v6+strict_align+vfp2-d32"),

        // All other supported targets don't need explicit CPU flags:
        // - x86_64: defaults are fine
        // - aarch64: defaults are fine
        // - Windows: uses default CPU features
        // - macOS: uses default CPU features
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arm_gnueabihf_has_cpu_flag() {
        let target = RustTarget::parse("arm-unknown-linux-gnueabihf").unwrap();
        assert_eq!(cpu_flag(&target), Some("generic+v6+strict_align+vfp2-d32"));
    }

    #[test]
    fn test_x86_64_linux_no_cpu_flag() {
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        assert_eq!(cpu_flag(&target), None);
    }

    #[test]
    fn test_x86_64_linux_musl_no_cpu_flag() {
        let target = RustTarget::parse("x86_64-unknown-linux-musl").unwrap();
        assert_eq!(cpu_flag(&target), None);
    }

    #[test]
    fn test_aarch64_linux_no_cpu_flag() {
        let target = RustTarget::parse("aarch64-unknown-linux-gnu").unwrap();
        assert_eq!(cpu_flag(&target), None);
    }

    #[test]
    fn test_aarch64_linux_musl_no_cpu_flag() {
        let target = RustTarget::parse("aarch64-unknown-linux-musl").unwrap();
        assert_eq!(cpu_flag(&target), None);
    }

    #[test]
    fn test_x86_64_darwin_no_cpu_flag() {
        let target = RustTarget::parse("x86_64-apple-darwin").unwrap();
        assert_eq!(cpu_flag(&target), None);
    }

    #[test]
    fn test_aarch64_darwin_no_cpu_flag() {
        let target = RustTarget::parse("aarch64-apple-darwin").unwrap();
        assert_eq!(cpu_flag(&target), None);
    }

    #[test]
    fn test_x86_64_windows_no_cpu_flag() {
        let target = RustTarget::parse("x86_64-pc-windows-gnu").unwrap();
        assert_eq!(cpu_flag(&target), None);
    }

    #[test]
    fn test_aarch64_windows_no_cpu_flag() {
        let target = RustTarget::parse("aarch64-pc-windows-gnullvm").unwrap();
        assert_eq!(cpu_flag(&target), None);
    }

    /// Verify all 10 supported targets have correct CPU flag behavior
    #[test]
    fn test_all_supported_targets_cpu_flags() {
        let test_cases = [
            // (target, expected_cpu_flag)
            (
                "arm-unknown-linux-gnueabihf",
                Some("generic+v6+strict_align+vfp2-d32"),
            ),
            ("aarch64-unknown-linux-gnu", None),
            ("aarch64-unknown-linux-musl", None),
            ("aarch64-apple-darwin", None),
            ("x86_64-pc-windows-gnu", None),
            ("aarch64-pc-windows-gnullvm", None),
            ("x86_64-apple-darwin", None),
            ("x86_64-unknown-linux-gnu", None),
            ("x86_64-unknown-linux-musl", None),
        ];

        for (rust_target, expected) in test_cases {
            let target = RustTarget::parse(rust_target).unwrap();
            assert_eq!(
                cpu_flag(&target),
                expected,
                "CPU flag mismatch for {}",
                rust_target
            );
        }
    }
}
