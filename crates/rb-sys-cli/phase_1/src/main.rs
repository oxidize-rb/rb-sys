mod assets;
mod bindings;
mod codegen;
mod config;
mod manifest;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Parser)]
#[command(name = "rb-sys-cli-phase-1")]
#[command(about = "Phase 1: Generate codegen and package assets for rb-sys-cli")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate toolchain mappings Rust code
    GenerateToolchains {
        /// Path to toolchains.json
        #[arg(long, default_value = "data/toolchains.json")]
        toolchains_json: PathBuf,

        /// Output directory for generated code
        #[arg(long, default_value = "data/derived")]
        derived_dir: PathBuf,
    },

    /// Generate runtime manifest (normalized, deterministic)
    GenerateManifest {
        /// Path to rb-sys-cli.json config
        #[arg(long, default_value = "data/derived/rb-sys-cli.json")]
        config: PathBuf,

        /// Cache directory with phase_0 outputs
        #[arg(long)]
        cache_dir: Option<PathBuf>,

        /// Output directory
        #[arg(long, default_value = "data/derived")]
        derived_dir: PathBuf,
    },

    /// Build embedded assets tarball
    BuildAssets {
        /// Path to rb-sys-cli.json config
        #[arg(long, default_value = "data/derived/rb-sys-cli.json")]
        config: PathBuf,

        /// Cache directory with phase_0 outputs
        #[arg(long)]
        cache_dir: Option<PathBuf>,

        /// Output directory for embedded files
        #[arg(long, default_value = "crates/rb-sys-cli/src/embedded")]
        embedded_dir: PathBuf,
    },

    /// Generate pre-generated Ruby bindings for all platforms and versions
    GenerateBindings {
        /// Path to rb-sys-cli.json config
        #[arg(long, default_value = "data/derived/rb-sys-cli.json")]
        config: PathBuf,

        /// Cache directory with phase_0 outputs
        #[arg(long)]
        cache_dir: Option<PathBuf>,
    },

    /// Run all phase 1 tasks
    All {
        /// Path to toolchains.json
        #[arg(long, default_value = "data/toolchains.json")]
        toolchains_json: PathBuf,

        /// Path to rb-sys-cli.json config
        #[arg(long, default_value = "data/derived/rb-sys-cli.json")]
        config: PathBuf,

        /// Cache directory with phase_0 outputs
        #[arg(long)]
        cache_dir: Option<PathBuf>,

        /// Output directory for derived files
        #[arg(long, default_value = "data/derived")]
        derived_dir: PathBuf,

        /// Output directory for embedded files
        #[arg(long, default_value = "crates/rb-sys-cli/src/embedded")]
        embedded_dir: PathBuf,
    },
}

fn main() -> Result<()> {
    // Setup tracing with progress bars
    let indicatif_layer = IndicatifLayer::new();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(indicatif_layer.get_stderr_writer())
                .with_target(false)
        )
        .with(indicatif_layer)
        .init();

    let args = Args::parse();

    match args.command {
        Commands::GenerateToolchains {
            toolchains_json,
            derived_dir,
        } => {
            let span = tracing::info_span!("generate_toolchains", indicatif.pb_show = tracing::field::Empty);
            let _enter = span.enter();
            
            codegen::generate_toolchains(&toolchains_json, &derived_dir)?;
            tracing::info!("✓ Generated toolchain mappings");
        }

        Commands::GenerateManifest {
            config,
            cache_dir,
            derived_dir,
        } => {
            let span = tracing::info_span!("generate_manifest", indicatif.pb_show = tracing::field::Empty);
            let _enter = span.enter();
            
            let cache_dir = resolve_cache_dir(cache_dir)?;
            manifest::generate_manifest(&config, &cache_dir, &derived_dir)?;
            tracing::info!("✓ Generated runtime manifest");
        }

        Commands::BuildAssets {
            config,
            cache_dir,
            embedded_dir,
        } => {
            let span = tracing::info_span!("build_assets", indicatif.pb_show = tracing::field::Empty);
            let _enter = span.enter();
            
            let cache_dir = resolve_cache_dir(cache_dir)?;
            assets::build_assets(&config, &cache_dir, &embedded_dir)?;
            tracing::info!("✓ Built embedded assets");
        }

        Commands::GenerateBindings { config, cache_dir } => {
            let span = tracing::info_span!("generate_bindings", indicatif.pb_show = tracing::field::Empty);
            let _enter = span.enter();

            let cache_dir = resolve_cache_dir(cache_dir)?;
            let cfg = config::Config::load(&config)?;
            let bindings_output_dir = cache_dir.join("bindings");
            bindings::generate_all_bindings(&cache_dir, &bindings_output_dir, &cfg)?;
            tracing::info!("✓ Generated pre-generated bindings");
        }

        Commands::All {
            toolchains_json,
            config,
            cache_dir,
            derived_dir,
            embedded_dir,
        } => {
            tracing::info!("Running all phase 1 tasks");
            
            {
                let span = tracing::info_span!("generate_toolchains", indicatif.pb_show = tracing::field::Empty);
                let _enter = span.enter();
                codegen::generate_toolchains(&toolchains_json, &derived_dir)?;
            }
            
            let cache_dir = resolve_cache_dir(cache_dir)?;
            
            {
                let span = tracing::info_span!("generate_manifest", indicatif.pb_show = tracing::field::Empty);
                let _enter = span.enter();
                manifest::generate_manifest(&config, &cache_dir, &derived_dir)?;
            }

            {
                let span = tracing::info_span!("generate_bindings", indicatif.pb_show = tracing::field::Empty);
                let _enter = span.enter();
                let cfg = config::Config::load(&config)?;
                let bindings_output_dir = cache_dir.join("bindings");
                bindings::generate_all_bindings(&cache_dir, &bindings_output_dir, &cfg)?;
            }
            
            {
                let span = tracing::info_span!("build_assets", indicatif.pb_show = tracing::field::Empty);
                let _enter = span.enter();
                assets::build_assets(&config, &cache_dir, &embedded_dir)?;
            }
            
            tracing::info!("✓ All phase 1 tasks complete");
        }
    }

    Ok(())
}

fn resolve_cache_dir(cache_dir: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(dir) = cache_dir {
        Ok(dir)
    } else if let Ok(dir) = std::env::var("RB_SYS_BUILD_CACHE_DIR") {
        Ok(PathBuf::from(dir))
    } else if let Some(cache_home) = std::env::var_os("XDG_CACHE_HOME") {
        Ok(PathBuf::from(cache_home).join("rb-sys/cli"))
    } else if let Some(home_dir) = dirs::home_dir() {
        Ok(home_dir.join(".cache/rb-sys/cli"))
    } else {
        anyhow::bail!("Could not determine cache directory")
    }
}
