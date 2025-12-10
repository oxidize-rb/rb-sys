//! Zig LD wrapper implementation using the ZigShim trait.

use super::super::tool::{ShimArgs, ZigShim};
use super::super::args::{ArgFilter, LinkMode};
use super::super::target::{Arch, Os, RustTarget};
use crate::platform::LinuxConfig;
use anyhow::{bail, Result};
use clap::Args;
use std::path::PathBuf;
use std::process::Command;

/// Zig linker wrapper
pub struct ZigLd {
    pub target: RustTarget,
    pub link_mode: LinkMode,
}

impl ZigShim for ZigLd {
    type Args = ZigLdArgs;

    fn subcommand(&self) -> &str {
        match self.target.os {
            Os::Darwin => "ld64.lld",
            Os::Windows => "cc",
            _ => "ld.lld",
        }
    }

    fn target(&self) -> Option<&RustTarget> {
        Some(&self.target)
    }

    fn validate(&self, args: &ZigLdArgs) -> Result<()> {
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

        Ok(())
    }

    fn add_platform_flags(&self, cmd: &mut Command, args: &ZigLdArgs) -> Result<()> {
        match self.target.os {
            Os::Windows => {
                // Windows GNU uses zig cc as linker driver
                cmd.arg("-target").arg(self.target.to_zig_target());
                cmd.arg("-fno-sanitize=all");
            }
            _ => {
                // Linux/Darwin use direct linker invocation
                if let Some(emulation) = linker_emulation(&self.target) {
                    cmd.arg("-m").arg(emulation);
                }

                match self.target.os {
                    Os::Linux => {
                        let sysroot = args.sysroot.as_ref().unwrap();
                        cmd.arg(format!("--sysroot={}", sysroot.display()));
                    }
                    Os::Darwin => {
                        if let Ok(sdkroot) = std::env::var("SDKROOT") {
                            cmd.arg("-syslibroot").arg(&sdkroot);
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn filter_args(&self, args: &[String]) -> Vec<String> {
        let filter = ArgFilter::with_link_mode(&self.target, self.link_mode);
        filter.filter_link_args(args)
    }
}

impl ShimArgs for ZigLdArgs {
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

fn linker_emulation(target: &RustTarget) -> Option<&'static str> {
    match target.os {
        Os::Darwin => None,
        Os::Windows => None,
        Os::Linux => Some(match target.arch {
            Arch::X86_64 => "elf_x86_64",
            Arch::Aarch64 => "aarch64linux",
            Arch::Arm => "armelf_linux_eabi",
        }),
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ld_args_parsing() {
        let args = ZigLdArgs {
            target: "x86_64-unknown-linux-gnu".to_string(),
            zig_path: PathBuf::from("/usr/bin/zig"),
            sysroot: None,
            args: vec!["-o".to_string(), "output".to_string()],
        };

        assert_eq!(args.target, "x86_64-unknown-linux-gnu");
        assert_eq!(args.zig_path, PathBuf::from("/usr/bin/zig"));
        assert_eq!(args.args.len(), 2);
    }

    #[test]
    fn test_ld_subcommand_selection() {
        let linux_target = RustTarget::parse("x86_64-unknown-linux-gnu").unwrap();
        let linux_ld = ZigLd {
            target: linux_target,
            link_mode: LinkMode::Direct,
        };
        assert_eq!(linux_ld.subcommand(), "ld.lld");

        let darwin_target = RustTarget::parse("aarch64-apple-darwin").unwrap();
        let darwin_ld = ZigLd {
            target: darwin_target,
            link_mode: LinkMode::Direct,
        };
        assert_eq!(darwin_ld.subcommand(), "ld64.lld");

        let windows_target = RustTarget::parse("x86_64-pc-windows-gnu").unwrap();
        let windows_ld = ZigLd {
            target: windows_target,
            link_mode: LinkMode::Direct,
        };
        assert_eq!(windows_ld.subcommand(), "cc");
    }
}
