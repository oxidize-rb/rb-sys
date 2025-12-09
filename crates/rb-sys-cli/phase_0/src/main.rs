mod cache;
mod config;
mod digest;
mod fetchers;
mod lockfile;
mod manifest;
mod oci;
mod rbconfig_parser;
mod zig;

use anyhow::{Context, Result};
use std::path::PathBuf;
use tracing::Instrument;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const MANIFEST_PATH: &str = "data/assets_manifest.toml";
const STAGING_DIR: &str = "data/staging/phase_0";
const LOCKFILE_PATH: &str = "data/derived/phase_0_lock.toml";
const CACHE_DIR: &str = "tmp/cache/phase_0";

#[tokio::main]
async fn main() -> Result<()> {
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
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with(indicatif_layer)
        .init();

    tracing::info!("Phase 0: Fetching assets");

    // Load manifest
    let manifest_path = PathBuf::from(MANIFEST_PATH);
    let manifest = manifest::Phase0Manifest::load(&manifest_path)
        .with_context(|| format!("Failed to load manifest: {}", manifest_path.display()))?;

    tracing::info!(
        "Loaded manifest with {} platforms",
        manifest.platforms().len()
    );

    // Setup directories
    let staging_dir = PathBuf::from(STAGING_DIR);
    let cache_dir = PathBuf::from(CACHE_DIR);
    let lockfile_path = PathBuf::from(LOCKFILE_PATH);

    std::fs::create_dir_all(&staging_dir)?;
    std::fs::create_dir_all(&cache_dir)?;
    std::fs::create_dir_all(lockfile_path.parent().unwrap())?;

    tracing::info!("Staging: {}", staging_dir.display());
    tracing::info!("Cache: {}", cache_dir.display());

    // Create lockfile
    let mut lockfile = lockfile::Lockfile::new();

    // Get all platforms
    let platforms = manifest.platforms();
    tracing::info!("Fetching assets for {} platforms", platforms.len());

    // Fetch platforms in parallel
    use futures::stream::{self, StreamExt};
    use tracing_indicatif::span_ext::IndicatifSpanExt;

    let overall_span = tracing::info_span!("fetch_all");
    overall_span.pb_set_length(platforms.len() as u64);
    overall_span.pb_set_message("Fetching platforms");
    let _overall_enter = overall_span.enter();

    let results = stream::iter(platforms)
        .map(|platform| {
            let manifest = manifest.clone();
            let staging_dir = staging_dir.clone();
            let cache_dir = cache_dir.clone();
            let span = tracing::info_span!("fetch_platform", platform = %platform);

            async move {
                let assets = manifest.assets_for_platform(&platform);
                let mut platform_locks = std::collections::HashMap::new();

                for asset in assets {
                    tracing::info!("[{}] Fetching: {}", platform, asset.name());

                    let lock = fetch_asset_with_retry(&asset, &platform, &staging_dir, &cache_dir)
                        .await
                        .with_context(|| {
                            format!("Failed to fetch asset {} for {}", asset.name(), platform)
                        })?;

                    platform_locks.insert(asset.name().to_string(), lock);
                    tracing::info!("[{}] ✓ {}", platform, asset.name());
                }

                Ok::<_, anyhow::Error>((platform, platform_locks))
            }
            .instrument(span)
        })
        .buffer_unordered(4); // Fetch 4 platforms concurrently

    futures::pin_mut!(results);

    let mut completed = 0;
    while let Some(result) = results.next().await {
        let (platform, platform_locks) = result?;

        // Add to lockfile
        for (name, lock) in platform_locks {
            lockfile.set_asset(&platform, &name, lock);
        }

        completed += 1;
        overall_span.pb_inc(1);
        overall_span.pb_set_message(&format!(
            "Completed {}/{} platforms",
            completed,
            manifest.platforms().len()
        ));
    }

    // Save lockfile
    lockfile.generated_at = chrono::Utc::now();
    lockfile.save(&lockfile_path)?;

    tracing::info!("✓ Phase 0 complete");
    tracing::info!("  Lockfile: {}", lockfile_path.display());

    Ok(())
}

const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 1000;

async fn fetch_asset_with_retry(
    asset: &manifest::AssetRequest,
    platform: &str,
    staging_dir: &PathBuf,
    cache_dir: &PathBuf,
) -> Result<lockfile::AssetLock> {
    let mut attempt = 0;

    loop {
        match fetch_asset_once(asset, platform, staging_dir, cache_dir).await {
            Ok(lock) => return Ok(lock),
            Err(e) if attempt < MAX_RETRIES => {
                let backoff = INITIAL_BACKOFF_MS * 2u64.pow(attempt);
                tracing::warn!(
                    "[{}] Fetch failed (attempt {}/{}): {}",
                    platform,
                    attempt + 1,
                    MAX_RETRIES,
                    e
                );
                tracing::info!("[{}] Retrying in {}ms...", platform, backoff);
                tokio::time::sleep(std::time::Duration::from_millis(backoff)).await;
                attempt += 1;
            }
            Err(e) => {
                return Err(e).with_context(|| format!("Failed after {} retries", MAX_RETRIES));
            }
        }
    }
}

async fn fetch_asset_once(
    asset: &manifest::AssetRequest,
    platform: &str,
    staging_dir: &PathBuf,
    cache_dir: &PathBuf,
) -> Result<lockfile::AssetLock> {
    let asset_dir = staging_dir.join(platform).join(asset.name());
    std::fs::create_dir_all(&asset_dir)?;

    match asset {
        manifest::AssetRequest::OciExtract {
            name,
            image,
            items,
            strip_prefix,
        } => {
            let extractor = fetchers::OciExtractor::new(image, items, strip_prefix.as_deref());
            let files = extractor.extract(&asset_dir, cache_dir).await?;

            Ok(lockfile::AssetLock::OciExtract {
                image: image.clone(),
                verified_at: chrono::Utc::now(),
                files,
            })
        }
        manifest::AssetRequest::Tarball {
            name,
            url,
            digest,
            strip_components: _,
        } => {
            let fetcher = fetchers::TarballFetcher::new(url, digest);
            let (tarball_path, size_bytes) = fetcher.fetch(cache_dir).await?;

            // Move tarball to staging area (will be unpacked in phase_1)
            let dest = asset_dir.join(
                tarball_path
                    .file_name()
                    .ok_or_else(|| anyhow::anyhow!("Invalid tarball path"))?,
            );
            std::fs::copy(&tarball_path, &dest)?;

            Ok(lockfile::AssetLock::Tarball {
                url: url.clone(),
                digest: digest.clone(),
                verified_at: chrono::Utc::now(),
                size_bytes,
            })
        }
        manifest::AssetRequest::TarballExtract {
            name: _,
            url,
            extract,
            digest,
            strip_components: _,
        } => {
            let extractor = fetchers::TarballExtractor::new(url, extract, digest);
            let (extracted_path, size_bytes, source_path) = extractor.fetch(cache_dir).await?;

            // Move extracted file to staging area
            let dest = asset_dir.join(
                extracted_path
                    .file_name()
                    .ok_or_else(|| anyhow::anyhow!("Invalid extracted file path"))?,
            );
            std::fs::copy(&extracted_path, &dest)?;

            Ok(lockfile::AssetLock::TarballExtract {
                url: url.clone(),
                digest: digest.clone(),
                source_path,
                verified_at: chrono::Utc::now(),
                size_bytes,
            })
        }
    }
}
