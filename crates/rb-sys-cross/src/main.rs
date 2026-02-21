mod build;
mod cargo_metadata;
mod cmd;
mod gem;
mod headers;
mod platform;
mod profile;
mod rbconfig;
mod toolchain;
mod util;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "rb-sys-cross",
    about = "Cross-compile Ruby native extensions without Docker",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Cross-compile a Ruby native extension
    Build(cmd::build::BuildOpts),

    /// Install toolchain prerequisites (zig, cargo-zigbuild, Rust targets)
    Setup {
        /// Platforms to set up (default: all)
        #[arg(long, short = 'p')]
        platform: Vec<String>,
    },

    /// List supported cross-compilation platforms
    ListPlatforms,

    /// Manage cached Ruby headers
    Headers {
        #[command(subcommand)]
        command: HeadersCommands,
    },
}

#[derive(Subcommand)]
enum HeadersCommands {
    /// List cached header bundles
    List,

    /// Build Ruby headers from source using zig
    Build {
        /// Target platform. e.g. aarch64-linux
        #[arg(long, short = 'p', required = true)]
        platform: String,

        /// Ruby version. e.g. 3.3.8
        #[arg(long, short = 'r', required = true)]
        ruby_version: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build(opts) => cmd::build::run(opts),

        Commands::Setup { platform } => cmd::setup::run(&platform),

        Commands::ListPlatforms => cmd::list_platforms::run(),

        Commands::Headers { command } => match command {
            HeadersCommands::List => cmd::headers::run_list(),
            HeadersCommands::Build {
                platform,
                ruby_version,
            } => cmd::headers::run_build(&platform, &ruby_version),
        },
    }
}
