mod bindings;
mod codegen;
mod transform;

use anyhow::Result;
use std::path::PathBuf;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const STAGING_DIR: &str = "data/staging/phase_0";
const OUTPUT_DIR: &str = "data/staging/phase_1";
const LOCKFILE_PATH: &str = "data/derived/phase_0_lock.toml";
const TOOLCHAINS_JSON: &str = "data/toolchains.json";
const DERIVED_DIR: &str = "data/derived";

fn main() -> Result<()> {
    // Setup tracing with progress bars
    let indicatif_layer = IndicatifLayer::new();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(indicatif_layer.get_stderr_writer())
                .with_target(false),
        )
        .with(indicatif_layer)
        .init();

    tracing::info!("Phase 1: Transform and enhance");

    // Step 1: Generate toolchain mappings
    {
        let span = tracing::info_span!("generate_toolchains");
        let _enter = span.enter();

        let toolchains_json = PathBuf::from(TOOLCHAINS_JSON);
        let derived_dir = PathBuf::from(DERIVED_DIR);

        codegen::generate_toolchains(&toolchains_json, &derived_dir)?;
        tracing::info!("✓ Generated toolchain mappings");
    }

    // Step 2: Transform phase_0 staging to normalized assets
    {
        let span = tracing::info_span!("transform");
        let _enter = span.enter();

        let staging_dir = PathBuf::from(STAGING_DIR);
        let output_dir = PathBuf::from(OUTPUT_DIR);
        let lockfile_path = PathBuf::from(LOCKFILE_PATH);

        std::fs::create_dir_all(&output_dir)?;

        transform::transform_assets(&staging_dir, &output_dir, &lockfile_path)?;

        tracing::info!("✓ Transformed assets");
        tracing::info!("  Staging: {}", staging_dir.display());
        tracing::info!("  Output: {}", output_dir.display());
    }

    // Step 3: Generate pre-compiled Ruby bindings for all platforms
    {
        let span = tracing::info_span!("generate_bindings");
        let _enter = span.enter();

        let cache_dir = PathBuf::from(STAGING_DIR);
        let output_dir = PathBuf::from(OUTPUT_DIR).join("assets");

        bindings::generate_all_bindings(&cache_dir, &output_dir)?;

        tracing::info!("✓ Generated bindings");
    }

    tracing::info!("✓ Phase 1 complete");

    Ok(())
}
