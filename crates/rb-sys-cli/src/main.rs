mod build;
mod extractor;
mod rbconfig_parser;
mod shim_generator;
mod toolchain;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

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

    /// Extract Ruby headers from a Docker image
    Extract {
        /// OCI image reference (e.g., ghcr.io/rake-compiler/rake-compiler-dock-image:1.3.0-mri-x86_64-linux)
        image: String,
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
    // Initialize logging - allows "RUST_LOG=debug cargo gem build..."
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
        
    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Build(config) => {
            build::build(&config)?;
        }

        Commands::Extract { image } => {
            extractor::extract_headers(&image).await?;
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
        println!("\n📍 Cache location: {}", extractor::get_cache_dir()?.display());
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
