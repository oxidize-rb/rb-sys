//! Zig LD wrapper command implementation.
//!
//! This module implements the `zig-ld` subcommand that is called by the linker
//! shim. It invokes `zig ld.lld` directly with appropriate flags for the target
//! platform.

use anyhow::{bail, Context, Result};
use clap::Args;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info};

use super::args::ArgFilter;
use super::target::{Arch, Os, RustTarget};
use crate::platform::LinuxConfig;

/// Arguments for the zig-ld subcommand.
#[derive(Args, Debug, Clone)]
pub struct ZigLdArgs {
    /// Rust target triple
    #[arg(long)]
    pub target: String,

    /// Path to zig executable
    #[arg(long)]
    pub zig_path: PathBuf,

    /// Path to sysroot (required for Linux targets)
    #[arg(long)]
    pub sysroot: Option<PathBuf>,

    /// Arguments to pass to zig ld.lld
    #[arg(last = true)]
    pub args: Vec<String>,
}

/// Run the zig-ld wrapper.
///
/// This function:
/// 1. Parses and validates the target
/// 2. Determines the correct linker flavor (ld.lld, ld64.lld, etc.)
/// 3. Adds the appropriate emulation flag
/// 4. Filters and transforms the input arguments
/// 5. Executes the linker
pub fn run_ld(args: ZigLdArgs) -> Result<()> {
    let target = RustTarget::parse(&args.target)?;

    debug!(
        target = %target,
        zig_path = %args.zig_path.display(),
        "Running zig linker wrapper"
    );

    // Validate platform requirements
    validate_requirements(&target, &args)?;

    // Build the zig command
    let mut cmd = Command::new(&args.zig_path);

    // Select the appropriate linker flavor
    let linker_flavor = linker_flavor(&target);
    cmd.arg(linker_flavor);
    debug!(linker_flavor = %linker_flavor, "Using linker flavor");

    // Add emulation flag for ELF targets
    if let Some(emulation) = linker_emulation(&target) {
        cmd.arg("-m").arg(emulation);
        debug!(emulation = %emulation, "Using linker emulation");
    }

    // Add platform-specific linker arguments
    add_platform_args(&mut cmd, &target, &args)?;

    // Filter and add user arguments
    let filter = ArgFilter::new(&target);
    let filtered_args = filter.filter_link_args(&args.args);

    debug!(
        original_args = ?args.args,
        filtered_args = ?filtered_args,
        "Filtered linker arguments"
    );

    for arg in filtered_args {
        cmd.arg(arg);
    }

    // Log the full command
    info!(command = ?cmd, "Executing zig linker");

    // Execute
    let status = cmd.status().context("Failed to execute zig linker")?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

/// Validate that all platform requirements are met.
fn validate_requirements(target: &RustTarget, args: &ZigLdArgs) -> Result<()> {
    // Linux targets require a sysroot
    if target.requires_sysroot() {
        match &args.sysroot {
            Some(sysroot) => {
                let config = LinuxConfig::new(target, sysroot.clone());
                if let Err(e) = config.validate() {
                    bail!(
                        "{}\n\n\
                         To extract the sysroot, run:\n  \
                         cargo gem extract --target {}",
                        e,
                        target.raw
                    );
                }
            }
            None => {
                bail!(
                    "Sysroot is required for Linux target: {}\n\n\
                     To extract the sysroot, run:\n  \
                     cargo gem extract --target {}",
                    target.raw,
                    target.raw
                );
            }
        }
    }

    Ok(())
}

/// Get the linker flavor for the target.
///
/// - Linux: ld.lld (ELF linker)
/// - macOS: ld64.lld (Mach-O linker)
/// - Windows: lld-link (PE/COFF linker, MSVC-compatible)
fn linker_flavor(target: &RustTarget) -> &'static str {
    match target.os {
        Os::Darwin => "ld64.lld",
        Os::Linux => "ld.lld",
        Os::Windows => "lld-link",
    }
}

/// Get the linker emulation flag for the target.
///
/// This is required for ld.lld (Linux) to know the output format.
/// ld64.lld and lld-link don't use emulation flags.
fn linker_emulation(target: &RustTarget) -> Option<&'static str> {
    match target.os {
        Os::Darwin => None,  // ld64.lld doesn't use -m
        Os::Windows => None, // lld-link doesn't use -m
        Os::Linux => Some(match target.arch {
            Arch::X86_64 => "elf_x86_64",
            Arch::Aarch64 => "aarch64linux",
            Arch::Arm => "armelf_linux_eabi",
        }),
    }
}

/// Add platform-specific linker arguments.
fn add_platform_args(cmd: &mut Command, target: &RustTarget, args: &ZigLdArgs) -> Result<()> {
    match target.os {
        Os::Linux => {
            let sysroot = args.sysroot.as_ref().unwrap();
            cmd.arg(format!("--sysroot={}", sysroot.display()));

            // NOTE: We intentionally do NOT add -L paths here.
            // The sysroot typically only contains static libraries (.a files),
            // not dynamic libraries (.so files). If we add -L paths, the linker
            // will find libc.a and try to statically link glibc, which fails
            // because glibc's malloc uses TLS relocations (R_X86_64_TPOFF32)
            // that are incompatible with shared libraries (-shared).
            //
            // Zig provides its own glibc shims for cross-compilation, so we
            // let it handle library resolution. The --sysroot is still needed
            // for finding CRT objects (crt1.o, crti.o, crtn.o).
        }
        Os::Darwin => {
            // macOS uses SDKROOT from environment
            if let Ok(sdkroot) = std::env::var("SDKROOT") {
                cmd.arg("-syslibroot").arg(&sdkroot);
            }
        }
        Os::Windows => {
            // Windows MinGW doesn't need special sysroot handling
            // Zig provides the Windows libraries
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linker_flavor_linux() {
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        assert_eq!(linker_flavor(&target), "ld.lld");
    }

    #[test]
    fn test_linker_flavor_darwin() {
        let target = RustTarget::parse("aarch64-apple-darwin").unwrap();
        assert_eq!(linker_flavor(&target), "ld64.lld");
    }

    #[test]
    fn test_linker_flavor_windows() {
        let target = RustTarget::parse("x86_64-pc-windows-gnu").unwrap();
        assert_eq!(linker_flavor(&target), "lld-link");
    }

    #[test]
    fn test_linker_emulation_linux_x86_64() {
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        assert_eq!(linker_emulation(&target), Some("elf_x86_64"));
    }

    #[test]
    fn test_linker_emulation_linux_aarch64() {
        let target = RustTarget::parse("aarch64-unknown-linux-gnu").unwrap();
        assert_eq!(linker_emulation(&target), Some("aarch64linux"));
    }

    #[test]
    fn test_linker_emulation_linux_arm() {
        let target = RustTarget::parse("arm-unknown-linux-gnueabihf").unwrap();
        assert_eq!(linker_emulation(&target), Some("armelf_linux_eabi"));
    }

    #[test]
    fn test_linker_emulation_darwin() {
        let target = RustTarget::parse("x86_64-apple-darwin").unwrap();
        assert_eq!(linker_emulation(&target), None);
    }

    #[test]
    fn test_linker_emulation_windows_x86_64() {
        let target = RustTarget::parse("x86_64-pc-windows-gnu").unwrap();
        assert_eq!(linker_emulation(&target), None); // lld-link doesn't use -m
    }

    #[test]
    fn test_linker_emulation_windows_aarch64() {
        let target = RustTarget::parse("aarch64-pc-windows-gnullvm").unwrap();
        assert_eq!(linker_emulation(&target), None); // lld-link doesn't use -m
    }

    #[test]
    fn test_validate_linux_requires_sysroot() {
        let args = ZigLdArgs {
            target: "x86_64-unknown-linux-gnu".to_string(),
            zig_path: PathBuf::from("/usr/bin/zig"),
            sysroot: None,
            args: vec![],
        };

        let target = RustTarget::parse(&args.target).unwrap();
        let result = validate_requirements(&target, &args);

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Sysroot is required"));
    }

    #[test]
    fn test_validate_windows_no_sysroot_needed() {
        let args = ZigLdArgs {
            target: "x86_64-pc-windows-gnu".to_string(),
            zig_path: PathBuf::from("/usr/bin/zig"),
            sysroot: None,
            args: vec![],
        };

        let target = RustTarget::parse(&args.target).unwrap();
        let result = validate_requirements(&target, &args);

        assert!(result.is_ok());
    }
}
