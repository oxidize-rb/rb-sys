//! Zig binary download and packaging for embedding in cargo-gem.
//!
//! This module downloads Zig from the official releases and repacks it
//! for embedding in the cargo-gem binary.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

/// Pinned Zig version to download
pub const ZIG_VERSION: &str = "0.15.2";

/// Zig download index URL
const ZIG_INDEX_URL: &str = "https://ziglang.org/download/index.json";

/// Host platforms we bundle Zig for (maps our key to Zig's key)
const HOST_PLATFORMS: &[(&str, &str)] = &[
    ("x86_64-linux", "x86_64-linux"),
    ("aarch64-linux", "aarch64-linux"),
    ("x86_64-macos", "x86_64-macos"),
    ("aarch64-macos", "aarch64-macos"),
    ("x86_64-windows", "x86_64-windows"),
];

/// Zig download index structure - maps version strings to version info
pub type ZigIndex = HashMap<String, ZigVersionInfo>;

/// Info for a specific Zig version.
/// Contains metadata fields (version, date, docs, src, bootstrap) plus platform-specific entries.
/// We use a custom deserializer to handle the mixed structure.
#[derive(Debug, Deserialize)]
pub struct ZigVersionInfo {
    /// Version string (e.g., "0.15.2")
    #[serde(default)]
    pub version: Option<String>,
    /// Release date
    #[serde(default)]
    pub date: Option<String>,
    /// Platform-specific download info, keyed by platform (e.g., "x86_64-linux")
    #[serde(flatten)]
    pub platforms: HashMap<String, serde_json::Value>,
}

impl ZigVersionInfo {
    /// Get platform info for a specific platform key
    pub fn get_platform(&self, platform: &str) -> Option<ZigPlatformInfo> {
        self.platforms.get(platform).and_then(|v| {
            // Only parse if it looks like platform info (has tarball field)
            if v.is_object() && v.get("tarball").is_some() {
                serde_json::from_value(v.clone()).ok()
            } else {
                None
            }
        })
    }
}

/// Info for a specific platform's Zig release
#[derive(Debug, Clone, Deserialize)]
pub struct ZigPlatformInfo {
    pub tarball: String,
    pub shasum: String,
    #[allow(dead_code)]
    pub size: String,
}

/// Manifest entry for embedded Zig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigManifest {
    pub version: String,
    pub host: String,
    pub sha256: String,
    pub executable: String,
}

/// Download and repack Zig for all host platforms.
///
/// This downloads the official Zig releases and repacks them into smaller
/// archives containing just what we need (the zig binary and lib directory).
pub async fn download_and_repack_zig(
    cache_dir: &Path,
    output_dir: &Path,
) -> Result<HashMap<String, ZigManifest>> {
    // Fetch the index
    tracing::info!("Fetching Zig download index");
    let index = fetch_zig_index().await?;

    let version_info = index
        .get(ZIG_VERSION)
        .with_context(|| format!("Zig version {ZIG_VERSION} not found in download index"))?;

    let mut manifests = HashMap::new();

    for (our_key, zig_key) in HOST_PLATFORMS {
        tracing::info!(
            platform = our_key,
            zig_platform = zig_key,
            "Processing Zig for platform"
        );

        let platform_info = version_info
            .get_platform(zig_key)
            .with_context(|| format!("Platform {zig_key} not found for Zig {ZIG_VERSION}"))?;

        let manifest = download_and_repack_platform(
            our_key,
            zig_key,
            &platform_info,
            cache_dir,
            output_dir,
        )
        .await?;

        manifests.insert(our_key.to_string(), manifest);
    }

    // Write combined manifest
    let manifest_path = output_dir.join("zig.json");
    let combined_manifest = serde_json::json!({
        "version": ZIG_VERSION,
        "platforms": manifests,
    });
    fs::write(&manifest_path, serde_json::to_string_pretty(&combined_manifest)?)
        .with_context(|| format!("Failed to write Zig manifest to {}", manifest_path.display()))?;

    tracing::info!(
        manifest_path = %manifest_path.display(),
        "Wrote Zig manifest"
    );

    Ok(manifests)
}

/// Fetch the Zig download index
async fn fetch_zig_index() -> Result<ZigIndex> {
    let response = reqwest::get(ZIG_INDEX_URL)
        .await
        .context("Failed to fetch Zig download index")?;

    if !response.status().is_success() {
        bail!(
            "Failed to fetch Zig index: HTTP {}",
            response.status()
        );
    }

    let index: ZigIndex = response
        .json()
        .await
        .context("Failed to parse Zig download index")?;

    Ok(index)
}

/// Download and repack a single platform's Zig distribution.
async fn download_and_repack_platform(
    our_key: &str,
    _zig_key: &str,
    platform_info: &ZigPlatformInfo,
    cache_dir: &Path,
    output_dir: &Path,
) -> Result<ZigManifest> {
    let download_cache = cache_dir.join("downloads");
    fs::create_dir_all(&download_cache)?;

    // Determine archive filename from URL
    let url = &platform_info.tarball;
    let archive_name = url
        .rsplit('/')
        .next()
        .context("Invalid Zig tarball URL")?;
    let cached_archive = download_cache.join(archive_name);

    // Download if not cached
    if !cached_archive.exists() {
        tracing::info!(
            url = url,
            dest = %cached_archive.display(),
            "Downloading Zig"
        );
        download_file(url, &cached_archive).await?;
    } else {
        tracing::info!(
            path = %cached_archive.display(),
            "Using cached Zig download"
        );
    }

    // Verify checksum
    let actual_sha256 = compute_sha256(&cached_archive)?;
    if actual_sha256 != platform_info.shasum {
        bail!(
            "SHA256 mismatch for {}:\n  expected: {}\n  actual: {}",
            archive_name,
            platform_info.shasum,
            actual_sha256
        );
    }

    // Create output directory for this platform
    let platform_output_dir = output_dir.join(our_key);
    fs::create_dir_all(&platform_output_dir)?;

    // Repack into our format (zstd-compressed tar)
    let output_archive = platform_output_dir.join("zig.tar.zst");
    repack_zig_archive(&cached_archive, &output_archive, our_key)?;

    // Compute sha256 of our repacked archive
    let repacked_sha256 = compute_sha256(&output_archive)?;

    let executable = if our_key.contains("windows") {
        "zig.exe".to_string()
    } else {
        "zig".to_string()
    };

    Ok(ZigManifest {
        version: ZIG_VERSION.to_string(),
        host: our_key.to_string(),
        sha256: repacked_sha256,
        executable,
    })
}

/// Download a file from a URL
async fn download_file(url: &str, dest: &Path) -> Result<()> {
    let response = reqwest::get(url)
        .await
        .with_context(|| format!("Failed to download {url}"))?;

    if !response.status().is_success() {
        bail!("Failed to download {}: HTTP {}", url, response.status());
    }

    let bytes = response
        .bytes()
        .await
        .with_context(|| format!("Failed to read response from {url}"))?;

    fs::write(dest, &bytes)
        .with_context(|| format!("Failed to write to {}", dest.display()))?;

    Ok(())
}

/// Compute SHA256 hash of a file
fn compute_sha256(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};

    let mut file = fs::File::open(path)
        .with_context(|| format!("Failed to open {} for hashing", path.display()))?;

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Repack a Zig archive into our zstd-compressed format.
///
/// This extracts the original archive and creates a new zstd-compressed tar
/// containing the same files. We keep the full archive structure since Zig
/// needs its lib directory for cross-compilation.
fn repack_zig_archive(input: &Path, output: &Path, _platform: &str) -> Result<()> {
    tracing::info!(
        input = %input.display(),
        output = %output.display(),
        "Repacking Zig archive"
    );

    let input_file = fs::File::open(input)
        .with_context(|| format!("Failed to open {}", input.display()))?;

    // Create output file with zstd compression
    let output_file = fs::File::create(output)
        .with_context(|| format!("Failed to create {}", output.display()))?;
    
    // Use zstd compression level 9 (good balance of speed and compression)
    // Level 19 is way too slow for 50MB+ archives, level 3 compresses poorly
    let encoder = zstd::Encoder::new(output_file, 9)?;
    let mut tar_builder = tar::Builder::new(encoder);

    let input_path_str = input.to_string_lossy();
    
    if input_path_str.ends_with(".tar.xz") {
        // Linux/macOS: .tar.xz
        let decompressor = xz2::read::XzDecoder::new(input_file);
        let mut archive = tar::Archive::new(decompressor);
        repack_tar_entries(&mut archive, &mut tar_builder)?;
    } else if input_path_str.ends_with(".zip") {
        // Windows: .zip
        repack_from_zip(input, &mut tar_builder)?;
    } else {
        bail!("Unsupported Zig archive format: {}", input.display());
    }

    // Finish the tar archive
    let encoder = tar_builder.into_inner()
        .context("Failed to finish tar archive")?;
    encoder.finish()
        .context("Failed to finish zstd compression")?;

    let output_size = fs::metadata(output)?.len();
    tracing::info!(
        output = %output.display(),
        size_mb = output_size as f64 / 1024.0 / 1024.0,
        "Repacked Zig archive"
    );

    Ok(())
}

/// Repack entries from a tar archive
fn repack_tar_entries<R: Read, W: Write>(
    archive: &mut tar::Archive<R>,
    builder: &mut tar::Builder<W>,
) -> Result<()> {
    for entry in archive.entries()? {
        let mut entry = entry?;
        let _path = entry.path()?.into_owned();
        let header = entry.header().clone();

        if header.entry_type().is_dir() {
            builder.append(&header, std::io::empty())?;
        } else {
            builder.append(&header, &mut entry)?;
        }
    }
    Ok(())
}

/// Repack from a zip archive (Windows)
fn repack_from_zip<W: Write>(zip_path: &Path, builder: &mut tar::Builder<W>) -> Result<()> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        let mut header = tar::Header::new_gnu();
        
        if file.is_dir() {
            header.set_entry_type(tar::EntryType::Directory);
            header.set_size(0);
            header.set_mode(0o755);
            header.set_cksum();
            builder.append_data(&mut header, &name, std::io::empty())?;
        } else {
            header.set_entry_type(tar::EntryType::Regular);
            header.set_size(file.size());
            header.set_mode(if name.ends_with(".exe") || name.contains("/zig") { 0o755 } else { 0o644 });
            header.set_cksum();
            
            let mut contents = Vec::new();
            std::io::Read::read_to_end(&mut file, &mut contents)?;
            builder.append_data(&mut header, &name, contents.as_slice())?;
        }
    }

    Ok(())
}
