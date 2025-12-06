//! Zig dlltool wrapper command implementation.
//!
//! Emulates the GNU `dlltool` utility by invoking `zig cc` with the
//! appropriate `-Wl,--def`/`--out-implib` arguments so the build system can
//! generate import libraries when cross-compiling for MinGW.

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info};

use super::target::RustTarget;

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
    let target = RustTarget::parse(&args.target)?;

    debug!(target = %target, args = ?args.args, "Running zig dlltool wrapper");

    let parsed = parse_dlltool_args(&args.args)?;

    if parsed.def_file.is_none() || parsed.output_lib.is_none() {
        anyhow::bail!("dlltool requires both --input-def/-d and --output-lib/-l")
    }

    let mut cmd = Command::new(&args.zig_path);
    cmd.arg("cc");
    cmd.arg("-target").arg(target.to_zig_target());
    cmd.arg("-shared");
    cmd.arg("-fno-sanitize=all");

    if let Some(def_file) = parsed.def_file.as_ref() {
        cmd.arg(format!("-Wl,--def,{}", def_file.display()));
    }

    if let Some(output_lib) = parsed.output_lib.as_ref() {
        cmd.arg(format!("-Wl,--out-implib,{}", output_lib.display()));
    }

    cmd.arg("-o");
    cmd.arg(dummy_output());

    info!(command = ?cmd, "Invoking zig to emulate dlltool");

    let status = cmd
        .status()
        .context("Failed to execute zig for dlltool emulation")?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn dummy_output() -> &'static str {
    if cfg!(windows) {
        "NUL"
    } else {
        "/dev/null"
    }
}

struct DlltoolArgsParsed {
    def_file: Option<PathBuf>,
    output_lib: Option<PathBuf>,
}

fn parse_dlltool_args(args: &[String]) -> Result<DlltoolArgsParsed> {
    let mut parsed = DlltoolArgsParsed {
        def_file: None,
        output_lib: None,
    };

    let mut iter = args.iter().peekable();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-d" | "--input-def" => {
                parsed.def_file = iter.next().map(PathBuf::from);
            }
            "-l" | "--output-lib" => {
                parsed.output_lib = iter.next().map(PathBuf::from);
            }
            "-D" => {
                // DLL name is informational; skip next arg
                iter.next();
            }
            "-k" | "--kill-at" => {
                // ignored flag
            }
            _ if arg.starts_with("-d") && arg.len() > 2 => {
                parsed.def_file = Some(PathBuf::from(&arg[2..]));
            }
            _ if arg.starts_with("-l") && arg.len() > 2 && !arg.starts_with("-l:") => {
                parsed.output_lib = Some(PathBuf::from(&arg[2..]));
            }
            _ => {
                debug!(arg = %arg, "Ignoring unknown dlltool argument");
            }
        }
    }

    Ok(parsed)
}
