mod build;
mod extractor;
pub mod generated_mappings;
mod platform;
mod rbconfig_parser;
mod toolchain;
mod zig;

use anyhow::Result;
use clap::{Parser, Subcommand};
use generated_mappings::Toolchain;
use tracing::info;
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

    /// Extract Ruby headers and sysroot from rake-compiler-dock image
    Extract {
        /// Target platform (e.g., x86_64-linux, aarch64-linux) or full image reference
        #[arg(value_name = "TARGET")]
        target: String,
    },

    /// List supported target platforms
    #[command(alias = "ls")]
    List {
        /// What to list
        #[command(subcommand)]
        what: ListCommands,
    },

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
}

#[derive(Subcommand)]
enum ListCommands {
    /// List supported target platforms
    Targets,

    /// List cached Ruby versions
    Rubies,
}

#[derive(Subcommand)]
enum CacheCommands {
    /// Clear the cache directory
    Clear,

    /// Show cache directory location
    Path,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging based on verbose flag or RUST_LOG env var
    let verbose = matches!(&cli.command, Commands::Build(config) if config.verbose);
    setup_logging(verbose);

    match cli.command {
        Commands::Build(config) => {
            build::build(&config)?;
        }

        Commands::Extract { target } => {
            extract_target(&target).await?;
        }

        Commands::List { what } => match what {
            ListCommands::Targets => {
                build::list_targets()?;
            }
            ListCommands::Rubies => {
                list_cached_rubies()?;
            }
        },

        Commands::Cache { action } => match action {
            CacheCommands::Clear => {
                extractor::clear_cache()?;
            }
            CacheCommands::Path => {
                let cache_dir = extractor::get_cache_dir()?;
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
    }

    Ok(())
}

/// Extract Ruby headers and sysroot for a target
async fn extract_target(target: &str) -> Result<()> {
    // First, try to find a toolchain by ruby-platform or rust-target
    let toolchain =
        Toolchain::from_ruby_platform(target).or_else(|| Toolchain::from_rust_target(target));

    match toolchain {
        Some(tc) => {
            // Found a known toolchain, use the new extraction with sysroot support
            extractor::extract_for_toolchain(tc).await?;
        }
        None => {
            // Check if it looks like a full image reference
            if target.contains('/') || target.contains(':') {
                // Looks like an image reference, use legacy extraction
                extractor::extract_headers(target).await?;
            } else {
                // Unknown target
                anyhow::bail!(
                    "Unknown target: '{}'\n\nSupported targets:\n{}",
                    target,
                    Toolchain::all_supported()
                        .map(|t| format!("  - {} ({})", t.ruby_platform(), t.rust_target()))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
            }
        }
    }

    Ok(())
}

fn list_cached_rubies() -> Result<()> {
    info!("Listing cached Ruby versions");

    println!("📚 Cached Ruby versions:\n");

    let rubies = extractor::list_cached_rubies()?;

    if rubies.is_empty() {
        println!("   No cached Ruby versions found.");
        println!("\n💡 Extract Ruby headers with:");
        println!("   cargo gem extract <image-ref>");
    } else {
        for ruby in rubies {
            println!("  • {}", ruby);
        }
        println!(
            "\n📍 Cache location: {}",
            extractor::get_cache_dir()?.display()
        );
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
