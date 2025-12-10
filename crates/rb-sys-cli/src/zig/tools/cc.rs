//! Zig CC/CXX wrapper implementation using the ZigShim trait.

use super::super::tool::{ShimArgs, ZigShim};
use super::super::args::ArgFilter;
use super::super::cpu::cpu_flag;
use super::super::target::{Os, RustTarget};
use crate::platform::{LinuxConfig, MacOSConfig, WindowsConfig};
use anyhow::{bail, Result};
use clap::Args;
use std::path::PathBuf;
use std::process::Command;

/// Zig C/C++ compiler wrapper
pub struct ZigCc {
    pub target: RustTarget,
    pub is_cxx: bool,
}

impl ZigShim for ZigCc {
    type Args = ZigCcArgs;

    fn subcommand(&self) -> &str {
        if self.is_cxx { "c++" } else { "cc" }
    }

    fn target(&self) -> Option<&RustTarget> {
        Some(&self.target)
    }

    fn validate(&self, args: &ZigCcArgs) -> Result<()> {
        // Validate sysroot for Linux targets
        if self.target.requires_sysroot() {
            match &args.sysroot {
                Some(sysroot) => {
                    let config = LinuxConfig::new(&self.target, sysroot.clone());
                    if let Err(e) = config.validate() {
                        bail!(
                            "{}\n\nTo extract the sysroot, run:\n  cargo gem extract --target {}",
                            e,
                            self.target.raw
                        );
                    }
                }
                None => {
                    bail!(
                        "Sysroot is required for Linux target: {}\n\nTo extract the sysroot, run:\n  cargo gem extract --target {}",
                        self.target.raw,
                        self.target.raw
                    );
                }
            }
        }

        // Validate SDKROOT for macOS targets
        if self.target.requires_sdkroot() {
            let config = MacOSConfig::from_env_or_embedded(None)?;
            if let Err(e) = config.validate() {
                bail!("{e}");
            }
        }

        Ok(())
    }

    fn add_platform_flags(&self, cmd: &mut Command, args: &ZigCcArgs) -> Result<()> {
        // Add target
        let zig_target = self.target.to_zig_target();
        cmd.arg("-target").arg(&zig_target);

        // Add CPU flag if needed
        if let Some(cpu) = cpu_flag(&self.target) {
            cmd.arg(format!("-mcpu={cpu}"));
        }

        // Add base flags
        cmd.arg("-g"); // Keep debug info
        cmd.arg("-fno-sanitize=all"); // Disable sanitizers

        // Add platform-specific arguments
        match self.target.os {
            Os::Linux => {
                let sysroot = args.sysroot.as_ref().unwrap();
                let config = LinuxConfig::new(&self.target, sysroot.clone());
                for arg in config.cc_args() {
                    cmd.arg(arg);
                }
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

    fn filter_args(&self, args: &[String]) -> Vec<String> {
        let filter = ArgFilter::new(&self.target);
        filter.filter_cc_args(args)
    }
}

impl ShimArgs for ZigCcArgs {
    fn zig_path(&self) -> &PathBuf {
        &self.zig_path
    }

    fn user_args(&self) -> &[String] {
        &self.args
    }

    fn target(&self) -> Option<&str> {
        Some(&self.target)
    }

    fn sysroot(&self) -> Option<&PathBuf> {
        self.sysroot.as_ref()
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cc_args_parsing() {
        let args = ZigCcArgs {
            target: "x86_64-unknown-linux-gnu".to_string(),
            zig_path: PathBuf::from("/usr/bin/zig"),
            sysroot: None,
            args: vec!["-c".to_string(), "foo.c".to_string()],
        };

        assert_eq!(args.target, "x86_64-unknown-linux-gnu");
        assert_eq!(args.zig_path, PathBuf::from("/usr/bin/zig"));
        assert_eq!(args.args.len(), 2);
    }

    #[test]
    fn test_cc_subcommand() {
        let target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        let cc = ZigCc {
            target,
            is_cxx: false,
        };
        assert_eq!(cc.subcommand(), "cc");

        let cxx = ZigCc {
            target: RustTarget::parse("x86_64-unknown-linux-gnu").unwrap(),
            is_cxx: true,
        };
        assert_eq!(cxx.subcommand(), "c++");
    }
}
