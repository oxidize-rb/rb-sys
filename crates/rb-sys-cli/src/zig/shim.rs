//! Bash shim generation for Zig cross-compilation.
//!
//! This module generates bash scripts that act as CC/CXX/AR/LD wrappers.
//! The shims call back into the `cargo-gem` binary with the appropriate
//! subcommand (zig-cc, zig-cxx, zig-ar, zig-ld) to perform argument filtering
//! and invoke Zig.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use super::target::RustTarget;

/// Generate all shims for a target in the given directory.
///
/// Creates:
/// - `cc`  - C compiler shim
/// - `c++` - C++ compiler shim
/// - `ar`  - Archiver shim
/// - `ld`  - Linker shim
/// - `dlltool` - dlltool emulator shim (Windows only)
///
/// Returns the ShimPaths for the generated shims.
pub fn generate_shims(
    shim_dir: &Path,
    cli_path: &Path,
    zig_path: &Path,
    target: &RustTarget,
    sysroot: Option<&Path>,
) -> Result<ShimPaths> {
    // Ensure shim directory exists
    fs::create_dir_all(shim_dir).context("Failed to create shim directory")?;

    // Generate CC shim
    let cc_content = generate_cc_shim(cli_path, zig_path, target, sysroot);
    let cc_path = shim_dir.join("cc");
    write_executable(&cc_path, &cc_content)?;

    // Generate C++ shim (conventional name)
    let cxx_content = generate_cxx_shim(cli_path, zig_path, target, sysroot);
    let cxx_path = shim_dir.join("c++");
    write_executable(&cxx_path, &cxx_content)?;

    // Generate AR shim
    let ar_content = generate_ar_shim(cli_path, zig_path);
    let ar_path = shim_dir.join("ar");
    write_executable(&ar_path, &ar_content)?;

    // Generate LD shim
    let ld_content = generate_ld_shim(cli_path, zig_path, target, sysroot);
    let ld_path = shim_dir.join("ld");
    write_executable(&ld_path, &ld_content)?;

    // Generate dlltool shim for Windows targets
    let has_dlltool = if target.os == super::target::Os::Windows {
        let dlltool_content = generate_dlltool_shim(cli_path, zig_path, target);
        let dlltool_path = shim_dir.join("dlltool");
        write_executable(&dlltool_path, &dlltool_content)?;

        // Create arch-prefixed symlink (e.g., x86_64-w64-mingw32-dlltool -> dlltool)
        let prefix = mingw_prefix(target);
        let prefixed_path = shim_dir.join(format!("{prefix}-dlltool"));

        #[cfg(unix)]
        {
            let _ = std::os::unix::fs::symlink("dlltool", &prefixed_path);
        }

        true
    } else {
        false
    };

    Ok(ShimPaths::new(shim_dir, has_dlltool))
}

/// Generate the CC (C compiler) shim script.
fn generate_cc_shim(
    cli_path: &Path,
    zig_path: &Path,
    target: &RustTarget,
    sysroot: Option<&Path>,
) -> String {
    let sysroot_arg = sysroot
        .map(|p| format!("--sysroot '{}'", p.display()))
        .unwrap_or_default();

    format!(
        r#"#!/usr/bin/env bash
exec '{cli_path}' zig-cc \
    --target '{target}' \
    --zig-path '{zig_path}' \
    {sysroot_arg} \
    -- "$@"
"#,
        cli_path = cli_path.display(),
        target = target.raw,
        zig_path = zig_path.display(),
        sysroot_arg = sysroot_arg,
    )
}

/// Generate the CXX (C++ compiler) shim script.
fn generate_cxx_shim(
    cli_path: &Path,
    zig_path: &Path,
    target: &RustTarget,
    sysroot: Option<&Path>,
) -> String {
    let sysroot_arg = sysroot
        .map(|p| format!("--sysroot '{}'", p.display()))
        .unwrap_or_default();

    format!(
        r#"#!/usr/bin/env bash
exec '{cli_path}' zig-cxx \
    --target '{target}' \
    --zig-path '{zig_path}' \
    {sysroot_arg} \
    -- "$@"
"#,
        cli_path = cli_path.display(),
        target = target.raw,
        zig_path = zig_path.display(),
        sysroot_arg = sysroot_arg,
    )
}

/// Generate the AR (archiver) shim script.
fn generate_ar_shim(cli_path: &Path, zig_path: &Path) -> String {
    format!(
        r#"#!/usr/bin/env bash
exec '{cli_path}' zig-ar \
    --zig-path '{zig_path}' \
    -- "$@"
"#,
        cli_path = cli_path.display(),
        zig_path = zig_path.display(),
    )
}

/// Generate the LD (linker) shim script.
fn generate_ld_shim(
    cli_path: &Path,
    zig_path: &Path,
    target: &RustTarget,
    sysroot: Option<&Path>,
) -> String {
    let sysroot_arg = sysroot
        .map(|p| format!("--sysroot '{}'", p.display()))
        .unwrap_or_default();

    format!(
        r#"#!/usr/bin/env bash
exec '{cli_path}' zig-ld \
    --target '{target}' \
    --zig-path '{zig_path}' \
    {sysroot_arg} \
    -- "$@"
"#,
        cli_path = cli_path.display(),
        target = target.raw,
        zig_path = zig_path.display(),
        sysroot_arg = sysroot_arg,
    )
}

/// Generate the dlltool (import library builder) shim script.
fn generate_dlltool_shim(cli_path: &Path, zig_path: &Path, target: &RustTarget) -> String {
    format!(
        r#"#!/usr/bin/env bash
exec '{cli_path}' zig-dlltool \
    --target '{target}' \
    --zig-path '{zig_path}' \
    -- "$@"
"#,
        cli_path = cli_path.display(),
        target = target.raw,
        zig_path = zig_path.display(),
    )
}

/// Get the MinGW triple prefix for a target (used for arch-prefixed tool names).
fn mingw_prefix(target: &RustTarget) -> &'static str {
    use super::target::Arch;
    match target.arch {
        Arch::X86_64 => "x86_64-w64-mingw32",
        Arch::Aarch64 => "aarch64-w64-mingw32",
        Arch::Arm => "arm-w64-mingw32",
    }
}

/// Write content to a file and make it executable.
fn write_executable(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content)
        .with_context(|| format!("Failed to write shim: {}", path.display()))?;

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms)?;
    }

    Ok(())
}

/// Get the paths to the generated shims.
pub struct ShimPaths {
    pub cc: std::path::PathBuf,
    pub cxx: std::path::PathBuf,
    pub ar: std::path::PathBuf,
    pub ld: std::path::PathBuf,
    pub dlltool: Option<std::path::PathBuf>,
}

impl ShimPaths {
    /// Create shim paths for a given directory.
    /// dlltool is only created for Windows targets.
    pub fn new(shim_dir: &Path, has_dlltool: bool) -> Self {
        Self {
            cc: shim_dir.join("cc"),
            cxx: shim_dir.join("c++"),
            ar: shim_dir.join("ar"),
            ld: shim_dir.join("ld"),
            dlltool: if has_dlltool {
                Some(shim_dir.join("dlltool"))
            } else {
                None
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_generate_cc_shim_linux() {
        let cli_path = PathBuf::from("/usr/local/bin/cargo-gem");
        let zig_path = PathBuf::from("/usr/bin/zig");
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        let sysroot = PathBuf::from("/path/to/sysroot");

        let shim = generate_cc_shim(&cli_path, &zig_path, &target, Some(&sysroot));

        assert!(shim.contains("#!/usr/bin/env bash"));
        assert!(shim.contains("cargo-gem"));
        assert!(shim.contains("zig-cc"));
        assert!(shim.contains("x86_64-unknown-linux-gnu"));
        assert!(shim.contains("/path/to/sysroot"));
        assert!(shim.contains("\"$@\""));
    }

    #[test]
    fn test_generate_cc_shim_macos_no_sysroot() {
        let cli_path = PathBuf::from("/usr/local/bin/cargo-gem");
        let zig_path = PathBuf::from("/usr/bin/zig");
        let target = RustTarget::parse("aarch64-apple-darwin").unwrap();

        let shim = generate_cc_shim(&cli_path, &zig_path, &target, None);

        assert!(shim.contains("aarch64-apple-darwin"));
        assert!(!shim.contains("--sysroot"));
    }

    #[test]
    fn test_generate_cxx_shim() {
        let cli_path = PathBuf::from("/usr/local/bin/cargo-gem");
        let zig_path = PathBuf::from("/usr/bin/zig");
        let target = RustTarget::parse("x86_64-pc-windows-gnu").unwrap();

        let shim = generate_cxx_shim(&cli_path, &zig_path, &target, None);

        assert!(shim.contains("zig-cxx"));
        assert!(shim.contains("x86_64-pc-windows-gnu"));
    }

    #[test]
    fn test_generate_ar_shim() {
        let cli_path = PathBuf::from("/usr/local/bin/cargo-gem");
        let zig_path = PathBuf::from("/usr/bin/zig");

        let shim = generate_ar_shim(&cli_path, &zig_path);

        assert!(shim.contains("zig-ar"));
        assert!(shim.contains("/usr/bin/zig"));
        assert!(!shim.contains("--target")); // AR doesn't need target
    }

    #[test]
    fn test_generate_ld_shim() {
        let cli_path = PathBuf::from("/usr/local/bin/cargo-gem");
        let zig_path = PathBuf::from("/usr/bin/zig");
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        let sysroot = PathBuf::from("/path/to/sysroot");

        let shim = generate_ld_shim(&cli_path, &zig_path, &target, Some(&sysroot));

        assert!(shim.contains("#!/usr/bin/env bash"));
        assert!(shim.contains("zig-ld"));
        assert!(shim.contains("x86_64-unknown-linux-gnu"));
        assert!(shim.contains("/path/to/sysroot"));
        assert!(shim.contains("\"$@\""));
    }

    #[test]
    fn test_generate_ld_shim_no_sysroot() {
        let cli_path = PathBuf::from("/usr/local/bin/cargo-gem");
        let zig_path = PathBuf::from("/usr/bin/zig");
        let target = RustTarget::parse("x86_64-pc-windows-gnu").unwrap();

        let shim = generate_ld_shim(&cli_path, &zig_path, &target, None);

        assert!(shim.contains("zig-ld"));
        assert!(!shim.contains("--sysroot"));
    }

    #[test]
    fn test_shim_paths_without_dlltool() {
        let shim_dir = PathBuf::from("/tmp/shims");
        let paths = ShimPaths::new(&shim_dir, false);

        assert_eq!(paths.cc, PathBuf::from("/tmp/shims/cc"));
        assert_eq!(paths.cxx, PathBuf::from("/tmp/shims/c++"));
        assert_eq!(paths.ar, PathBuf::from("/tmp/shims/ar"));
        assert_eq!(paths.ld, PathBuf::from("/tmp/shims/ld"));
        assert_eq!(paths.dlltool, None);
    }

    #[test]
    fn test_shim_paths_with_dlltool() {
        let shim_dir = PathBuf::from("/tmp/shims");
        let paths = ShimPaths::new(&shim_dir, true);

        assert_eq!(paths.cc, PathBuf::from("/tmp/shims/cc"));
        assert_eq!(paths.cxx, PathBuf::from("/tmp/shims/c++"));
        assert_eq!(paths.ar, PathBuf::from("/tmp/shims/ar"));
        assert_eq!(paths.ld, PathBuf::from("/tmp/shims/ld"));
        assert_eq!(paths.dlltool, Some(PathBuf::from("/tmp/shims/dlltool")));
    }
}
