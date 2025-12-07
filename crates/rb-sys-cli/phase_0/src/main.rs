mod cache;
mod config;
mod oci;
mod zig;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::Instrument;
use tracing_indicatif::IndicatifLayer;
use tracing_indicatif::style::ProgressStyle;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Parser)]
#[command(name = "rb-sys-cli-phase-0")]
#[command(about = "Phase 0: Download and extract OCI images for rb-sys-cli")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to rb-sys-cli.json config file
    #[arg(long, default_value = "data/derived/rb-sys-cli.json")]
    config: PathBuf,

    /// Cache directory for extracted assets
    #[arg(long)]
    cache_dir: Option<PathBuf>,

    /// Skip extraction (for testing)
    #[arg(long)]
    skip_extraction: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Download and repack Zig for all host platforms
    DownloadZig {
        /// Output directory for repacked Zig archives
        #[arg(long, default_value = "crates/rb-sys-cli/src/embedded/tools")]
        output_dir: PathBuf,

        /// Cache directory for downloads
        #[arg(long)]
        cache_dir: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Setup tracing with indicatif progress bars
    let indicatif_layer = IndicatifLayer::new();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(indicatif_layer.get_stderr_writer())
                .with_target(false),
        )
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .with(indicatif_layer)
        .init();

    // Handle subcommands
    if let Some(command) = args.command {
        return match command {
            Commands::DownloadZig { output_dir, cache_dir } => {
                let cache_dir = cache_dir
                    .or_else(|| std::env::var("RB_SYS_BUILD_CACHE_DIR").ok().map(PathBuf::from))
                    .unwrap_or_else(|| cache::get_default_cache_dir().unwrap());
                
                std::fs::create_dir_all(&output_dir)
                    .with_context(|| format!("Failed to create output dir: {}", output_dir.display()))?;
                
                zig::download_and_repack_zig(&cache_dir, &output_dir).await?;
                tracing::info!("Zig download complete");
                Ok(())
            }
        };
    }

    // Default behavior: extract OCI images
    // Load config
    let config = config::Config::load(&args.config)
        .with_context(|| format!("Failed to load config from {}", args.config.display()))?;

    // Determine cache directory
    let cache_dir = if let Some(dir) = args.cache_dir {
        dir
    } else if let Ok(dir) = std::env::var("RB_SYS_BUILD_CACHE_DIR") {
        PathBuf::from(dir)
    } else {
        cache::get_default_cache_dir()?
    };

    tracing::info!("Using cache directory: {}", cache_dir.display());

    // Load or create manifest
    let manifest_path = cache_dir.join("manifest.json");
    let mut manifest = cache::load_manifest(&manifest_path)?;
    let mut any_changed = false;

    // Collect platforms that need extraction
    let mut platforms_to_extract = Vec::new();
    for toolchain in &config.toolchains {
        if args.skip_extraction || std::env::var("RB_SYS_SKIP_EXTRACTION").is_ok() {
            tracing::info!("Skipping extraction for {} (skip flag set)", toolchain.ruby_platform);
            continue;
        }

        if cache::needs_extraction(&cache_dir, &toolchain.ruby_platform, &toolchain.oci.digest, &manifest) {
            platforms_to_extract.push(toolchain.clone());
        } else {
            tracing::info!("{} is up to date", toolchain.ruby_platform);
        }
    }

    // Extract platforms in parallel (up to 8 concurrent)
    if !platforms_to_extract.is_empty() {
        use futures::stream::{self, StreamExt};
        use tracing_indicatif::span_ext::IndicatifSpanExt;

        let total = platforms_to_extract.len() as u64;
        
        // Create overall progress span with percentage bar
        let overall_span = tracing::info_span!("phase_0_extract");
        overall_span.pb_set_style(
            &ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
                .unwrap()
                .progress_chars("=>-")
        );
        overall_span.pb_set_length(total);
        overall_span.pb_set_message("Extracting toolchains");
        let _overall_enter = overall_span.enter();

        let results = stream::iter(platforms_to_extract)
            .map(|toolchain| {
                let cache_dir = cache_dir.clone();
                let span = tracing::info_span!("extract", platform = %toolchain.ruby_platform);

                async move {
                    let ruby_versions = oci::extract_platform(
                        &toolchain.oci.reference,
                        &toolchain.ruby_platform,
                        &toolchain.sysroot_paths,
                        &cache_dir,
                    )
                    .await?;

                    Ok::<_, anyhow::Error>((toolchain, ruby_versions))
                }
                .instrument(span)
            })
            .buffer_unordered(8);

        futures::pin_mut!(results);

        let mut completed = 0u64;
        while let Some(result) = results.next().await {
            let (toolchain, ruby_versions) =
                result.with_context(|| "Failed to extract platform")?;

            // Update manifest
            manifest.set_platform(
                toolchain.ruby_platform.clone(),
                cache::PlatformInfo {
                    ruby_platform: toolchain.ruby_platform.clone(),
                    rust_target: toolchain.rust_target.clone(),
                    image: toolchain.oci.tag.clone(),
                    image_digest: toolchain.oci.digest.clone(),
                    ruby_versions,
                    has_sysroot: !toolchain.sysroot_paths.is_empty(),
                    extracted_at: chrono::Utc::now(),
                },
            );

            // Write digest marker
            cache::write_digest_marker(&cache_dir, &toolchain.ruby_platform, &toolchain.oci.digest)?;

            completed += 1;
            overall_span.pb_inc(1);
            overall_span.pb_set_message(&format!(
                "Extracted {}/{} toolchains (latest: {})",
                completed,
                total,
                toolchain.ruby_platform
            ));

            any_changed = true;
        }

        overall_span.pb_set_finish_message("All toolchains extracted");
    }

    // Save manifest if anything changed
    if any_changed {
        cache::save_manifest(&manifest_path, &manifest)?;
        tracing::info!("Updated manifest at {}", manifest_path.display());
    }

    tracing::info!("Phase 0 complete");

    Ok(())
}
