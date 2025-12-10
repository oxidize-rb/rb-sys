//! Zig AR wrapper implementation using the ZigShim trait.

use super::super::tool::{ShimArgs, ZigShim};
use super::super::args::filter_ar_args;
use clap::Args;
use std::path::PathBuf;

/// Zig archiver wrapper
pub struct ZigAr;

impl ZigShim for ZigAr {
    type Args = ZigArArgs;

    fn subcommand(&self) -> &str {
        "ar"
    }

    fn filter_args(&self, args: &[String]) -> Vec<String> {
        filter_ar_args(args)
    }
}

impl ShimArgs for ZigArArgs {
    fn zig_path(&self) -> &PathBuf {
        &self.zig_path
    }

    fn user_args(&self) -> &[String] {
        &self.args
    }
}

/// Arguments for the zig-ar subcommand.
#[derive(Args, Debug, Clone)]
pub struct ZigArArgs {
    /// Path to zig executable
    #[arg(long)]
    pub zig_path: PathBuf,

    /// Arguments to pass to zig ar
    #[arg(last = true)]
    pub args: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ar_args_parsing() {
        let args = ZigArArgs {
            zig_path: PathBuf::from("/usr/bin/zig"),
            args: vec![
                "crs".to_string(),
                "libfoo.a".to_string(),
                "foo.o".to_string(),
            ],
        };

        assert_eq!(args.zig_path, PathBuf::from("/usr/bin/zig"));
        assert_eq!(args.args.len(), 3);
    }

    #[test]
    fn test_ar_subcommand() {
        let ar = ZigAr;
        assert_eq!(ar.subcommand(), "ar");
    }
}
