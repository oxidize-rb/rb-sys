//! Zig AR wrapper command implementation.
//!
//! This module re-exports the argument types for the zig-ar subcommand.
//! The actual implementation is in the `tools` module using the `ZigShim` trait.

use clap::Args;
use std::path::PathBuf;

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
