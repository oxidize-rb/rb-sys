mod assets;
mod build;
pub mod generated_mappings;
mod platform;
mod sysroot;
mod toolchain;
mod zig;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{fmt, EnvFilter};

/// Setup logging based on verbose flag or RUST_LOG environment variable
fn setup_logging(verbose: bool) {
    // RUST_LOG env var takes precedence if set
    let filter = if std::env::var("RUST_LOG").is_ok() {
        EnvFilter::from_default_env()
    } else if verbose {
        // Verbose mode: show debug logs for rb-sys-cli
        EnvFilter::new("rb_sys_cli=debug,rb_sys_build=debug")
    } else {
        // Default: info level
        EnvFilter::new("rb_sys_cli=info,rb_sys_build=info")
    };

    fmt().with_env_filter(filter).with_target(false).init();
}

#[derive(Parser)]
#[command(name = "cargo-gem")]
#[command(bin_name = "cargo-gem")]
#[command(version, about = "Cross-compile Rust native gems with ease", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a native gem for a specific target platform
    #[command(alias = "b")]
    Build(build::BuildConfig),

    /// List supported target platforms
    #[command(alias = "ls")]
    List,

    /// Cache management
    Cache {
        #[command(subcommand)]
        action: CacheCommands,
    },

    /// Internal: Zig CC wrapper (called by shims)
    #[command(hide = true)]
    ZigCc(zig::cc::ZigCcArgs),

    /// Internal: Zig C++ wrapper (called by shims)
    #[command(hide = true)]
    ZigCxx(zig::cc::ZigCcArgs),

    /// Internal: Zig AR wrapper (called by shims)
    #[command(hide = true)]
    ZigAr(zig::ar::ZigArArgs),

    /// Internal: Zig LD wrapper (called by shims)
    #[command(hide = true)]
    ZigLd(zig::ld::ZigLdArgs),

    /// Internal: Zig dlltool wrapper (called by shims)
    #[command(hide = true)]
    ZigDlltool(zig::dlltool::ZigDlltoolArgs),
}

#[derive(Subcommand)]
enum CacheCommands {
    /// Clear the cache directory
    Clear,

    /// Show cache directory location
    Path,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging based on verbose flag or RUST_LOG env var
    let verbose = matches!(&cli.command, Commands::Build(config) if config.verbose);
    setup_logging(verbose);

    match cli.command {
        Commands::Build(config) => {
            build::build(&config)?;
        }

        Commands::List => {
            build::list_targets()?;
        }

        Commands::Cache { action } => match action {
            CacheCommands::Clear => {
                assets::clear_cache()?;
            }
            CacheCommands::Path => {
                let cache_dir = assets::get_cache_dir()?;
                println!("{}", cache_dir.display());
            }
        },

        // Zig wrapper commands (called by shims)
        Commands::ZigCc(args) => {
            zig::cc::run_cc(args, false)?;
        }
        Commands::ZigCxx(args) => {
            zig::cc::run_cc(args, true)?;
        }
        Commands::ZigAr(args) => {
            zig::ar::run_ar(args)?;
        }
        Commands::ZigLd(args) => {
            zig::ld::run_ld(args)?;
        }
        Commands::ZigDlltool(args) => {
            zig::dlltool::run_dlltool(args)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
