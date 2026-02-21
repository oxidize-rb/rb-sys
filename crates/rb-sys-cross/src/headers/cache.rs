use std::path::PathBuf;

use anyhow::{Context, Result};

/// Return the cache directory for header bundles.
/// Uses `~/.cache/rb-sys-cross/headers/` (respects XDG_CACHE_HOME on Linux).
pub fn cache_dir() -> Result<PathBuf> {
    let base = dirs_path()?;
    Ok(base.join("headers"))
}

/// Return the path where headers for a specific platform+version would be cached.
pub fn header_dir(ruby_platform: &str, ruby_version: &str) -> Result<PathBuf> {
    Ok(cache_dir()?.join(ruby_platform).join(ruby_version))
}

/// Check if headers are already cached for the given platform+version.
pub fn is_cached(ruby_platform: &str, ruby_version: &str) -> Result<bool> {
    let dir = header_dir(ruby_platform, ruby_version)?;
    // Check for the rbconfig.json marker file
    Ok(dir.join("rbconfig.json").exists())
}

fn dirs_path() -> Result<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        return Ok(PathBuf::from(xdg).join("rb-sys-cross"));
    }
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join(".cache").join("rb-sys-cross"))
}
