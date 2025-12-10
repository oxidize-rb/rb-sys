//! Zig dlltool wrapper command implementation.
//!
//! This module re-exports the argument types for the zig-dlltool subcommand.
//! The actual implementation is in the `tools` module using the `ZigShim` trait.

use clap::Args;
use std::path::PathBuf;

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
