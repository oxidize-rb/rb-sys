//! Rust target triple parsing and Zig target translation.
//!
//! Handles the 10 supported targets from toolchains.json:
//! - arm-unknown-linux-gnueabihf
//! - aarch64-unknown-linux-gnu
//! - aarch64-unknown-linux-musl
//! - aarch64-apple-darwin
//! - x86_64-pc-windows-gnu
//! - aarch64-pc-windows-gnullvm
//! - x86_64-apple-darwin
//! - x86_64-unknown-linux-gnu
//! - x86_64-unknown-linux-musl

use anyhow::{bail, Result};

/// Supported CPU architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Arch {
    X86_64,
    Aarch64,
    Arm,
}

impl Arch {
    /// Convert to Zig architecture string
    pub fn as_zig_str(&self) -> &'static str {
        match self {
            Arch::X86_64 => "x86_64",
            Arch::Aarch64 => "aarch64",
            Arch::Arm => "arm",
        }
    }
}

/// Supported operating systems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Os {
    Linux,
    Darwin,
    Windows,
}

impl Os {
    /// Convert to Zig OS string
    pub fn as_zig_str(&self) -> &'static str {
        match self {
            Os::Linux => "linux",
            Os::Darwin => "macos",
            Os::Windows => "windows",
        }
    }
}

/// Supported environments/ABIs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Env {
    /// GNU libc
    Gnu,
    /// musl libc
    Musl,
    /// GNU with hard-float ABI (ARM)
    Gnueabihf,
    /// GNU LLVM (Windows ARM64)
    Gnullvm,
}

impl Env {
    /// Convert to Zig environment string
    pub fn as_zig_str(&self, os: Os) -> &'static str {
        match (self, os) {
            // macOS uses "none" as the ABI
            (_, Os::Darwin) => "none",
            // Windows GNU variants all map to "gnu"
            (Env::Gnu | Env::Gnullvm, Os::Windows) => "gnu",
            // Linux environments
            (Env::Gnu, Os::Linux) => "gnu",
            (Env::Musl, Os::Linux) => "musl",
            (Env::Gnueabihf, Os::Linux) => "gnueabihf",
            // Fallback (shouldn't happen with supported targets)
            _ => "gnu",
        }
    }

    /// Check if this is a musl environment
    #[allow(dead_code)]
    pub fn is_musl(&self) -> bool {
        matches!(self, Env::Musl)
    }

    /// Check if this is a glibc environment
    pub fn is_glibc(&self) -> bool {
        matches!(self, Env::Gnu | Env::Gnueabihf)
    }
}

/// Parsed Rust target triple
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustTarget {
    /// CPU architecture
    pub arch: Arch,
    /// Operating system
    pub os: Os,
    /// Environment/ABI
    pub env: Env,
    /// Original triple string
    pub raw: String,
}

impl RustTarget {
    /// Parse a Rust target triple string.
    ///
    /// Only supports the 10 targets defined in toolchains.json.
    pub fn parse(triple: &str) -> Result<Self> {
        let (arch, os, env) = match triple {
            // Linux glibc targets
            "x86_64-unknown-linux-gnu" => (Arch::X86_64, Os::Linux, Env::Gnu),
            "aarch64-unknown-linux-gnu" => (Arch::Aarch64, Os::Linux, Env::Gnu),
            "arm-unknown-linux-gnueabihf" => (Arch::Arm, Os::Linux, Env::Gnueabihf),

            // Linux musl targets
            "x86_64-unknown-linux-musl" => (Arch::X86_64, Os::Linux, Env::Musl),
            "aarch64-unknown-linux-musl" => (Arch::Aarch64, Os::Linux, Env::Musl),

            // macOS targets
            "x86_64-apple-darwin" => (Arch::X86_64, Os::Darwin, Env::Gnu),
            "aarch64-apple-darwin" => (Arch::Aarch64, Os::Darwin, Env::Gnu),

            // Windows targets
            "x86_64-pc-windows-gnu" => (Arch::X86_64, Os::Windows, Env::Gnu),
            "aarch64-pc-windows-gnullvm" => (Arch::Aarch64, Os::Windows, Env::Gnullvm),

            _ => bail!(
                "Unsupported target: {triple}\n\n\
                 Supported targets:\n  \
                 - arm-unknown-linux-gnueabihf\n  \
                 - aarch64-unknown-linux-gnu\n  \
                 - aarch64-unknown-linux-musl\n  \
                 - aarch64-apple-darwin\n  \
                 - x86_64-pc-windows-gnu\n  \
                 - aarch64-pc-windows-gnullvm\n  \
                 - x86_64-apple-darwin\n  \
                 - x86_64-unknown-linux-gnu\n  \
                 - x86_64-unknown-linux-musl"
            ),
        };

        Ok(Self {
            arch,
            os,
            env,
            raw: triple.to_string(),
        })
    }

    /// Convert to Zig target format.
    ///
    /// For glibc Linux targets, appends `.2.17` (our default glibc version).
    /// For musl targets, no version suffix.
    /// For macOS, uses `none` as the ABI.
    /// For Windows, maps both `gnu` and `gnullvm` to `gnu`.
    pub fn to_zig_target(&self) -> String {
        let arch = self.arch.as_zig_str();
        let os = self.os.as_zig_str();
        let env = self.env.as_zig_str(self.os);

        let base = format!("{arch}-{os}-{env}");

        // Append glibc version for Linux glibc targets
        if self.os == Os::Linux && self.env.is_glibc() {
            format!("{base}.2.17")
        } else {
            base
        }
    }

    /// Get the GNU triple for sysroot paths.
    ///
    /// This is used to find architecture-specific include directories
    /// like `/usr/include/x86_64-linux-gnu`.
    pub fn gnu_triple(&self) -> String {
        match (self.arch, self.os, self.env) {
            (Arch::X86_64, Os::Linux, Env::Gnu) => "x86_64-linux-gnu".to_string(),
            (Arch::X86_64, Os::Linux, Env::Musl) => "x86_64-linux-musl".to_string(),
            (Arch::Aarch64, Os::Linux, Env::Gnu) => "aarch64-linux-gnu".to_string(),
            (Arch::Aarch64, Os::Linux, Env::Musl) => "aarch64-linux-musl".to_string(),
            (Arch::Arm, Os::Linux, Env::Gnueabihf) => "arm-linux-gnueabihf".to_string(),
            // macOS and Windows don't use GNU triples for sysroot
            _ => self.raw.replace("-unknown-", "-").replace("-pc-", "-"),
        }
    }

    /// Check if this target requires a sysroot for cross-compilation.
    ///
    /// Linux targets need a sysroot with headers and libraries.
    pub fn requires_sysroot(&self) -> bool {
        self.os == Os::Linux
    }

    /// Check if this target requires SDKROOT environment variable.
    ///
    /// macOS targets need the macOS SDK path.
    pub fn requires_sdkroot(&self) -> bool {
        self.os == Os::Darwin
    }

    /// Check if this is a Windows target.
    #[allow(dead_code)]
    pub fn is_windows(&self) -> bool {
        self.os == Os::Windows
    }

    /// Check if this is a macOS target.
    #[allow(dead_code)]
    pub fn is_macos(&self) -> bool {
        self.os == Os::Darwin
    }

    /// Check if this is a Linux target.
    #[allow(dead_code)]
    pub fn is_linux(&self) -> bool {
        self.os == Os::Linux
    }
}

impl std::fmt::Display for RustTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_x86_64_linux_gnu() {
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        assert_eq!(target.arch, Arch::X86_64);
        assert_eq!(target.os, Os::Linux);
        assert_eq!(target.env, Env::Gnu);
    }

    #[test]
    fn test_parse_aarch64_linux_gnu() {
        let target = RustTarget::parse("aarch64-unknown-linux-gnu").unwrap();
        assert_eq!(target.arch, Arch::Aarch64);
        assert_eq!(target.os, Os::Linux);
        assert_eq!(target.env, Env::Gnu);
    }

    #[test]
    fn test_parse_arm_linux_gnueabihf() {
        let target = RustTarget::parse("arm-unknown-linux-gnueabihf").unwrap();
        assert_eq!(target.arch, Arch::Arm);
        assert_eq!(target.os, Os::Linux);
        assert_eq!(target.env, Env::Gnueabihf);
    }

    #[test]
    fn test_parse_x86_64_linux_musl() {
        let target = RustTarget::parse("x86_64-unknown-linux-musl").unwrap();
        assert_eq!(target.arch, Arch::X86_64);
        assert_eq!(target.os, Os::Linux);
        assert_eq!(target.env, Env::Musl);
    }

    #[test]
    fn test_parse_aarch64_linux_musl() {
        let target = RustTarget::parse("aarch64-unknown-linux-musl").unwrap();
        assert_eq!(target.arch, Arch::Aarch64);
        assert_eq!(target.os, Os::Linux);
        assert_eq!(target.env, Env::Musl);
    }

    #[test]
    fn test_parse_x86_64_darwin() {
        let target = RustTarget::parse("x86_64-apple-darwin").unwrap();
        assert_eq!(target.arch, Arch::X86_64);
        assert_eq!(target.os, Os::Darwin);
    }

    #[test]
    fn test_parse_aarch64_darwin() {
        let target = RustTarget::parse("aarch64-apple-darwin").unwrap();
        assert_eq!(target.arch, Arch::Aarch64);
        assert_eq!(target.os, Os::Darwin);
    }

    #[test]
    fn test_parse_x86_64_windows_gnu() {
        let target = RustTarget::parse("x86_64-pc-windows-gnu").unwrap();
        assert_eq!(target.arch, Arch::X86_64);
        assert_eq!(target.os, Os::Windows);
        assert_eq!(target.env, Env::Gnu);
    }

    #[test]
    fn test_parse_aarch64_windows_gnullvm() {
        let target = RustTarget::parse("aarch64-pc-windows-gnullvm").unwrap();
        assert_eq!(target.arch, Arch::Aarch64);
        assert_eq!(target.os, Os::Windows);
        assert_eq!(target.env, Env::Gnullvm);
    }

    #[test]
    fn test_parse_unsupported_target() {
        let result = RustTarget::parse("riscv64-unknown-linux-gnu");
        assert!(result.is_err());
    }

    #[test]
    fn test_to_zig_target_linux_glibc() {
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        assert_eq!(target.to_zig_target(), "x86_64-linux-gnu.2.17");
    }

    #[test]
    fn test_to_zig_target_linux_glibc_aarch64() {
        let target = RustTarget::parse("aarch64-unknown-linux-gnu").unwrap();
        assert_eq!(target.to_zig_target(), "aarch64-linux-gnu.2.17");
    }

    #[test]
    fn test_to_zig_target_linux_glibc_arm() {
        let target = RustTarget::parse("arm-unknown-linux-gnueabihf").unwrap();
        assert_eq!(target.to_zig_target(), "arm-linux-gnueabihf.2.17");
    }

    #[test]
    fn test_to_zig_target_linux_musl() {
        let target = RustTarget::parse("x86_64-unknown-linux-musl").unwrap();
        assert_eq!(target.to_zig_target(), "x86_64-linux-musl");
    }

    #[test]
    fn test_to_zig_target_linux_musl_aarch64() {
        let target = RustTarget::parse("aarch64-unknown-linux-musl").unwrap();
        assert_eq!(target.to_zig_target(), "aarch64-linux-musl");
    }

    #[test]
    fn test_to_zig_target_macos_x86_64() {
        let target = RustTarget::parse("x86_64-apple-darwin").unwrap();
        assert_eq!(target.to_zig_target(), "x86_64-macos-none");
    }

    #[test]
    fn test_to_zig_target_macos_aarch64() {
        let target = RustTarget::parse("aarch64-apple-darwin").unwrap();
        assert_eq!(target.to_zig_target(), "aarch64-macos-none");
    }

    #[test]
    fn test_to_zig_target_windows_x86_64() {
        let target = RustTarget::parse("x86_64-pc-windows-gnu").unwrap();
        assert_eq!(target.to_zig_target(), "x86_64-windows-gnu");
    }

    #[test]
    fn test_to_zig_target_windows_aarch64_gnullvm() {
        let target = RustTarget::parse("aarch64-pc-windows-gnullvm").unwrap();
        assert_eq!(target.to_zig_target(), "aarch64-windows-gnu");
    }

    #[test]
    fn test_gnu_triple_x86_64_linux() {
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        assert_eq!(target.gnu_triple(), "x86_64-linux-gnu");
    }

    #[test]
    fn test_gnu_triple_aarch64_linux() {
        let target = RustTarget::parse("aarch64-unknown-linux-gnu").unwrap();
        assert_eq!(target.gnu_triple(), "aarch64-linux-gnu");
    }

    #[test]
    fn test_gnu_triple_arm_linux() {
        let target = RustTarget::parse("arm-unknown-linux-gnueabihf").unwrap();
        assert_eq!(target.gnu_triple(), "arm-linux-gnueabihf");
    }

    #[test]
    fn test_requires_sysroot() {
        assert!(RustTarget::parse("x86_64-unknown-linux-gnu")
            .unwrap()
            .requires_sysroot());
        assert!(RustTarget::parse("aarch64-unknown-linux-musl")
            .unwrap()
            .requires_sysroot());
        assert!(!RustTarget::parse("x86_64-apple-darwin")
            .unwrap()
            .requires_sysroot());
        assert!(!RustTarget::parse("x86_64-pc-windows-gnu")
            .unwrap()
            .requires_sysroot());
    }

    #[test]
    fn test_requires_sdkroot() {
        assert!(RustTarget::parse("x86_64-apple-darwin")
            .unwrap()
            .requires_sdkroot());
        assert!(RustTarget::parse("aarch64-apple-darwin")
            .unwrap()
            .requires_sdkroot());
        assert!(!RustTarget::parse("x86_64-unknown-linux-gnu")
            .unwrap()
            .requires_sdkroot());
        assert!(!RustTarget::parse("x86_64-pc-windows-gnu")
            .unwrap()
            .requires_sdkroot());
    }

    /// Test all 10 supported targets produce correct Zig targets
    #[test]
    fn test_all_supported_targets() {
        let test_cases = [
            ("arm-unknown-linux-gnueabihf", "arm-linux-gnueabihf.2.17"),
            ("aarch64-unknown-linux-gnu", "aarch64-linux-gnu.2.17"),
            ("aarch64-unknown-linux-musl", "aarch64-linux-musl"),
            ("aarch64-apple-darwin", "aarch64-macos-none"),
            ("x86_64-pc-windows-gnu", "x86_64-windows-gnu"),
            ("aarch64-pc-windows-gnullvm", "aarch64-windows-gnu"),
            ("x86_64-apple-darwin", "x86_64-macos-none"),
            ("x86_64-unknown-linux-gnu", "x86_64-linux-gnu.2.17"),
            ("x86_64-unknown-linux-musl", "x86_64-linux-musl"),
        ];

        for (rust_target, expected_zig) in test_cases {
            let target = RustTarget::parse(rust_target)
                .unwrap_or_else(|e| panic!("Failed to parse {rust_target}: {e}"));
            assert_eq!(
                target.to_zig_target(),
                expected_zig,
                "Zig target mismatch for {rust_target}"
            );
        }
    }
}
