mod oci_extract;
mod tarball;
mod tarball_extract;

pub use oci_extract::OciExtractor;
pub use tarball::TarballFetcher;
pub use tarball_extract::TarballExtractor;

use crate::lockfile::{AssetLock, Lockfile};
use crate::manifest::AssetRequest;
use anyhow::Result;
use std::path::Path;

/// Fetch an asset and update the lockfile
pub async fn fetch_asset(
    asset: &AssetRequest,
    platform: &str,
    staging_dir: &Path,
    cache_dir: &Path,
    lockfile: &mut Lockfile,
) -> Result<()> {
    let asset_dir = staging_dir.join(platform).join(asset.name());
    std::fs::create_dir_all(&asset_dir)?;

    match asset {
        AssetRequest::OciExtract {
            name,
            image,
            items,
            strip_prefix,
        } => {
            let extractor = OciExtractor::new(image, items, strip_prefix.as_deref());
            let files = extractor.extract(&asset_dir, cache_dir).await?;

            lockfile.set_asset(
                platform,
                name,
                AssetLock::OciExtract {
                    image: image.clone(),
                    verified_at: chrono::Utc::now(),
                    files,
                },
            );
        }
        AssetRequest::Tarball {
            name,
            url,
            digest,
            strip_components: _,
        } => {
            let fetcher = TarballFetcher::new(url, digest);
            let (tarball_path, size_bytes) = fetcher.fetch(cache_dir).await?;

            // Move tarball to staging area (will be unpacked in phase_1)
            let dest = asset_dir.join(
                tarball_path
                    .file_name()
                    .ok_or_else(|| anyhow::anyhow!("Invalid tarball path"))?,
            );
            std::fs::copy(&tarball_path, &dest)?;

            lockfile.set_asset(
                platform,
                name,
                AssetLock::Tarball {
                    url: url.clone(),
                    digest: digest.clone(),
                    verified_at: chrono::Utc::now(),
                    size_bytes,
                },
            );
        }
        AssetRequest::TarballExtract {
            name,
            url,
            extract,
            digest,
            strip_components: _,
        } => {
            let extractor = TarballExtractor::new(url, extract, digest);
            let (extracted_path, size_bytes, source_path) = extractor.fetch(cache_dir).await?;

            // Move extracted file to staging area
            let dest = asset_dir.join(
                extracted_path
                    .file_name()
                    .ok_or_else(|| anyhow::anyhow!("Invalid extracted file path"))?,
            );
            std::fs::copy(&extracted_path, &dest)?;

            lockfile.set_asset(
                platform,
                name,
                AssetLock::TarballExtract {
                    url: url.clone(),
                    digest: digest.clone(),
                    source_path,
                    verified_at: chrono::Utc::now(),
                    size_bytes,
                },
            );
        }
    }

    Ok(())
}
