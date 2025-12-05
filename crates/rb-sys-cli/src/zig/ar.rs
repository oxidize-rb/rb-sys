//! Zig AR wrapper command implementation.
//!
//! This module implements the `zig-ar` subcommand that is called by the
//! bash shim. It filters AR arguments for Zig/LLVM compatibility.

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info};

use super::args::filter_ar_args;

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

/// Run the zig-ar wrapper.
///
/// This function:
/// 1. Filters AR arguments for LLVM compatibility
/// 2. Executes zig ar with the filtered arguments
pub fn run_ar(args: ZigArArgs) -> Result<()> {
    debug!(
        zig_path = %args.zig_path.display(),
        args = ?args.args,
        "Running zig ar wrapper"
    );

    // Build the zig ar command
    let mut cmd = Command::new(&args.zig_path);
    cmd.arg("ar");

    // Filter arguments for LLVM ar compatibility
    let filtered_args = filter_ar_args(&args.args);

    debug!(
        original_args = ?args.args,
        filtered_args = ?filtered_args,
        "Filtered AR arguments"
    );

    for arg in filtered_args {
        cmd.arg(arg);
    }

    // Log the full command in verbose mode
    info!(command = ?cmd, "Executing zig ar");

    // Execute
    let status = cmd.status().context("Failed to execute zig ar")?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
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
}
