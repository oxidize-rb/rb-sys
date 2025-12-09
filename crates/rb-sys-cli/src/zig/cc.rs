//! Zig CC/CXX wrapper command implementation.
//!
//! This module implements the `zig-cc` and `zig-cxx` subcommands that are
//! called by the bash shims. It handles argument filtering, platform-specific
//! configuration, and invokes Zig with the correct flags.

use anyhow::{bail, Context, Result};
use clap::Args;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info};

use super::args::ArgFilter;
use super::cpu::cpu_flag;
use super::target::{Os, RustTarget};
use crate::platform::{LinuxConfig, MacOSConfig, WindowsConfig};

/// Arguments for the zig-cc subcommand.
#[derive(Args, Debug, Clone)]
pub struct ZigCcArgs {
    /// Rust target triple
    #[arg(long)]
    pub target: String,

    /// Path to zig executable
    #[arg(long)]
    pub zig_path: PathBuf,

    /// Path to sysroot (required for Linux targets)
    #[arg(long)]
    pub sysroot: Option<PathBuf>,

    /// Arguments to pass to zig cc
    #[arg(last = true)]
    pub args: Vec<String>,
}

/// Run the zig-cc wrapper.
///
/// This function:
/// 1. Parses and validates the target
/// 2. Validates platform requirements (sysroot, SDKROOT)
/// 3. Builds the zig cc command with appropriate flags
/// 4. Filters and transforms the input arguments
/// 5. Executes zig cc
pub fn run_cc(args: ZigCcArgs, is_cxx: bool) -> Result<()> {
    let target = RustTarget::parse(&args.target)?;
    let subcommand = if is_cxx { "c++" } else { "cc" };

    debug!(
        target = %target,
        zig_path = %args.zig_path.display(),
        is_cxx = is_cxx,
        "Running zig wrapper"
    );

    // Validate platform requirements
    validate_requirements(&target, &args)?;

    // Build the zig command
    let mut cmd = Command::new(&args.zig_path);
    cmd.arg(subcommand);

    // Add target
    let zig_target = target.to_zig_target();
    cmd.arg("-target").arg(&zig_target);
    debug!(zig_target = %zig_target, "Using Zig target");

    // Add CPU flag if needed
    if let Some(cpu) = cpu_flag(&target) {
        cmd.arg(format!("-mcpu={cpu}"));
        debug!(mcpu = %cpu, "Using CPU flag");
    }

    // Add base flags
    cmd.arg("-g"); // Keep debug info
    cmd.arg("-fno-sanitize=all"); // Disable sanitizers

    // Add platform-specific arguments
    add_platform_args(&mut cmd, &target, &args)?;

    // Filter and add user arguments (CC args only - linking uses zig-ld)
    let filter = ArgFilter::new(&target);
    let filtered_args = filter.filter_cc_args(&args.args);

    debug!(
        original_args = ?args.args,
        filtered_args = ?filtered_args,
        "Filtered CC arguments"
    );

    for arg in filtered_args {
        cmd.arg(arg);
    }

    // Log the full command in verbose mode
    info!(command = ?cmd, "Executing zig");

    // Execute
    let status = cmd.status().context("Failed to execute zig")?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

/// Validate that all platform requirements are met.
fn validate_requirements(target: &RustTarget, args: &ZigCcArgs) -> Result<()> {
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

    // macOS targets require SDKROOT
    if target.requires_sdkroot() {
        let config = MacOSConfig::from_env_or_embedded(None)?;
        if let Err(e) = config.validate() {
            bail!("{e}");
        }
    }

    Ok(())
}

/// Add platform-specific arguments to the command.
fn add_platform_args(cmd: &mut Command, target: &RustTarget, args: &ZigCcArgs) -> Result<()> {
    match target.os {
        Os::Linux => {
            let sysroot = args.sysroot.as_ref().unwrap();
            let config = LinuxConfig::new(target, sysroot.clone());

            for arg in config.cc_args() {
                cmd.arg(arg);
            }

            // Add musl-specific defines
            if config.is_musl {
                for arg in LinuxConfig::musl_defines() {
                    cmd.arg(arg);
                }
            }
        }
        Os::Darwin => {
            let config = MacOSConfig::from_env_or_embedded(None)?;

            for arg in config.cc_args() {
                cmd.arg(arg);
            }
        }
        Os::Windows => {
            for arg in WindowsConfig::cc_args() {
                cmd.arg(arg);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_linux_requires_sysroot() {
        let args = ZigCcArgs {
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
        assert!(err.contains("cargo gem extract"));
    }

    #[test]
    fn test_validate_windows_no_sysroot_needed() {
        let args = ZigCcArgs {
            target: "x86_64-pc-windows-gnu".to_string(),
            zig_path: PathBuf::from("/usr/bin/zig"),
            sysroot: None,
            args: vec![],
        };

        let target = RustTarget::parse(&args.target).unwrap();
        // Windows doesn't require sysroot, so this should pass
        // (SDKROOT check is separate)
        assert!(!target.requires_sysroot());
    }
}
