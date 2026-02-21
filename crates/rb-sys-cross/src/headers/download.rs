use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use tar::Archive;

use super::{build, cache};
use crate::platform::Platform;

/// The GitHub org/repo where header bundles are hosted.
const HEADERS_REPO: &str = "oxidize-rb/rb-sys";

/// Download and cache Ruby headers for the given platform and version.
/// Returns the path to the cached header directory.
pub fn ensure_headers(ruby_platform: &str, ruby_version: &str) -> Result<PathBuf> {
    if cache::is_cached(ruby_platform, ruby_version)? {
        return cache::header_dir(ruby_platform, ruby_version);
    }

    let dest = cache::header_dir(ruby_platform, ruby_version)?;
    std::fs::create_dir_all(&dest)
        .with_context(|| format!("creating cache dir {}", dest.display()))?;

    let url = header_bundle_url(ruby_platform, ruby_version);
    eprintln!("Downloading Ruby headers: {url}");

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("{ruby_platform} ruby-{ruby_version}"));

    let response = reqwest::blocking::get(&url).with_context(|| format!("fetching {url}"))?;

    if !response.status().is_success() {
        // Clean up empty dir on failure
        let _ = std::fs::remove_dir_all(&dest);

        // Fall back to building from source if zig is available
        if response.status().as_u16() == 404 {
            eprintln!(
                "Headers not available for download (HTTP 404). \
                 Attempting to build from source with zig..."
            );
            if let Ok(plat) = Platform::find(ruby_platform) {
                if plat.zig_supported {
                    return build::build_ruby_headers(plat, ruby_version);
                }
            }
        }

        bail!(
            "failed to download headers from {url}: HTTP {}.\n\
             Headers may not be available yet for this platform/version.\n\
             See https://github.com/{HEADERS_REPO}/releases for available bundles.",
            response.status()
        );
    }

    let bytes = response.bytes().context("reading response body")?;
    let gz = GzDecoder::new(bytes.as_ref());
    let mut archive = Archive::new(gz);
    archive
        .unpack(&dest)
        .with_context(|| format!("extracting headers to {}", dest.display()))?;

    pb.finish_with_message(format!("{ruby_platform} ruby-{ruby_version} ✓"));

    // Verify we got the expected rbconfig.json
    if !dest.join("rbconfig.json").exists() {
        let _ = std::fs::remove_dir_all(&dest);
        bail!(
            "downloaded header bundle is missing rbconfig.json.\n\
             The bundle at {url} may be malformed."
        );
    }

    Ok(dest)
}

/// Load rbconfig.json from the cached headers directory.
pub fn load_rbconfig(header_dir: &std::path::Path) -> Result<crate::rbconfig::RbConfig> {
    let path = header_dir.join("rbconfig.json");
    crate::rbconfig::RbConfig::from_json_file(&path)
}

/// Construct the URL for a header bundle.
fn header_bundle_url(ruby_platform: &str, ruby_version: &str) -> String {
    format!(
        "https://github.com/{HEADERS_REPO}/releases/download/headers-v0.1.0/ruby-headers-{ruby_platform}-{ruby_version}.tar.gz"
    )
}

/// List all cached header bundles.
pub fn list_cached() -> Result<Vec<(String, String)>> {
    let cache = cache::cache_dir()?;
    let mut result = Vec::new();

    if !cache.exists() {
        return Ok(result);
    }

    let read_cache = std::fs::read_dir(&cache).context("reading cache dir")?;
    for platform_entry in read_cache {
        let platform_entry = platform_entry?;
        if !platform_entry.file_type()?.is_dir() {
            continue;
        }
        let platform = platform_entry.file_name().to_string_lossy().to_string();
        let read_platform =
            std::fs::read_dir(platform_entry.path()).context("reading platform dir")?;
        for version_entry in read_platform {
            let version_entry = version_entry?;
            if !version_entry.file_type()?.is_dir() {
                continue;
            }
            let version = version_entry.file_name().to_string_lossy().to_string();
            if version_entry.path().join("rbconfig.json").exists() {
                result.push((platform.clone(), version));
            }
        }
    }

    result.sort();
    Ok(result)
}
