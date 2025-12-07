//! Zig dlltool wrapper command implementation.
//!
//! Wraps `zig dlltool` (llvm-dlltool) to provide MinGW-compatible
//! import library generation for Windows cross-compilation.

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;
use std::process::Command;
use tracing::debug;

/// Arguments for the `zig-dlltool` subcommand.
#[derive(Args, Debug, Clone)]
pub struct ZigDlltoolArgs {
    /// Rust target triple (e.g., x86_64-pc-windows-gnu)
    #[arg(long)]
    pub target: String,

    /// Path to the zig executable
    #[arg(long)]
    pub zig_path: PathBuf,

    /// Arguments forwarded from the original `dlltool` invocation
    #[arg(last = true)]
    pub args: Vec<String>,
}

/// Run the dlltool emulator.
pub fn run_dlltool(args: ZigDlltoolArgs) -> Result<()> {
    debug!(args = ?args.args, "Running zig dlltool wrapper");

    // Zig has llvm-dlltool built-in, so we can just forward all arguments directly
    let mut cmd = Command::new(&args.zig_path);
    cmd.arg("dlltool");
    
    // Forward all arguments as-is
    for arg in &args.args {
        cmd.arg(arg);
    }

    debug!(command = ?cmd, "Invoking zig dlltool");

    let status = cmd
        .status()
        .context("Failed to execute zig dlltool")?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}


