//! Zig dlltool wrapper implementation using the ZigShim trait.

use super::super::tool::{ShimArgs, ZigShim};
use super::super::target::RustTarget;
use clap::Args;
use std::path::PathBuf;

/// Zig dlltool wrapper
pub struct ZigDlltool {
    pub target: RustTarget,
}

impl ZigShim for ZigDlltool {
    type Args = ZigDlltoolArgs;

    fn subcommand(&self) -> &str {
        "dlltool"
    }

    fn target(&self) -> Option<&RustTarget> {
        Some(&self.target)
    }

    fn filter_args(&self, args: &[String]) -> Vec<String> {
        // dlltool args don't need filtering
        args.to_vec()
    }
}

impl ShimArgs for ZigDlltoolArgs {
    fn zig_path(&self) -> &PathBuf {
        &self.zig_path
    }

    fn user_args(&self) -> &[String] {
        &self.args
    }

    fn target(&self) -> Option<&str> {
        Some(&self.target)
    }
}

/// Arguments for the zig-dlltool subcommand.
#[derive(Args, Debug, Clone)]
pub struct ZigDlltoolArgs {
    /// Rust target triple
    #[arg(long)]
    pub target: String,

    /// Path to zig executable
    #[arg(long)]
    pub zig_path: PathBuf,

    /// Arguments to pass to zig dlltool
    #[arg(last = true)]
    pub args: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dlltool_args_parsing() {
        let args = ZigDlltoolArgs {
            target: "x86_64-pc-windows-gnu".to_string(),
            zig_path: PathBuf::from("/usr/bin/zig"),
            args: vec!["-d".to_string(), "lib.def".to_string()],
        };

        assert_eq!(args.target, "x86_64-pc-windows-gnu");
        assert_eq!(args.zig_path, PathBuf::from("/usr/bin/zig"));
        assert_eq!(args.args.len(), 2);
    }

    #[test]
    fn test_dlltool_subcommand() {
        let target = RustTarget::parse("x86_64-pc-windows-gnu").unwrap();
        let dlltool = ZigDlltool { target };
        assert_eq!(dlltool.subcommand(), "dlltool");
    }
}
