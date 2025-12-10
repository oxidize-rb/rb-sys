//! Zig CC/CXX wrapper command implementation.
//!
//! This module re-exports the argument types for the zig-cc and zig-cxx subcommands.
//! The actual implementation is in the `tools` module using the `ZigShim` trait.

use clap::Args;
use std::path::PathBuf;

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
