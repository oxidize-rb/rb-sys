pub mod manifest;

use anyhow::{Context, Result};
use manifest::Manifest;
use std::fs;
use std::path::{Path, PathBuf};

/// Embedded compressed asset archive
static EMBEDDED_ASSETS: &[u8] = include_bytes!("../embedded/assets.tar.zst");

/// Embedded manifest (for quick lookups without decompressing)
static EMBEDDED_MANIFEST: &str = include_str!("../embedded/manifest.json");

/// Manages embedded assets and lazy extraction
pub struct AssetManager {
    cache_dir: PathBuf,
    manifest: Manifest,
}

impl AssetManager {
    /// Create a new AssetManager
    pub fn new() -> Result<Self> {
        let cache_dir = get_runtime_cache_dir()?;
        let manifest: Manifest = serde_json::from_str(EMBEDDED_MANIFEST)
            .context("Failed to parse embedded manifest")?;

        Ok(Self {
            cache_dir,
            manifest,
        })
    }

    /// Get the manifest
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Extract sysroot files directly from embedded tarball to destination
    pub fn extract_sysroot(&self, ruby_platform: &str, dest_dir: &Path) -> Result<()> {
        let decoder = zstd::Decoder::new(EMBEDDED_ASSETS)
            .context("Failed to create zstd decoder")?;
        let mut archive = tar::Archive::new(decoder);

        let sysroot_prefix = format!("{ruby_platform}/sysroot/");

        for entry in archive.entries().context("Failed to read tar entries")? {
            let mut entry = entry.context("Failed to read tar entry")?;
            let path = entry.path().context("Failed to get entry path")?;
            let path_str = path.to_string_lossy();

            // Only extract sysroot files for this platform
            if let Some(relative) = path_str.strip_prefix(&sysroot_prefix) {
                let dest_path = dest_dir.join(relative);

                if entry.header().entry_type().is_dir() {
                    fs::create_dir_all(&dest_path).with_context(|| {
                        format!("Failed to create directory: {}", dest_path.display())
                    })?;
                } else {
                    if let Some(parent) = dest_path.parent() {
                        fs::create_dir_all(parent).with_context(|| {
                            format!("Failed to create parent directory: {}", parent.display())
                        })?;
                    }
                    let mut file = fs::File::create(&dest_path)
                        .with_context(|| format!("Failed to create file: {}", dest_path.display()))?;
                    std::io::copy(&mut entry, &mut file)
                        .with_context(|| format!("Failed to write file: {}", dest_path.display()))?;

                    // Preserve permissions on Unix
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(mode) = entry.header().mode() {
                            let perms = fs::Permissions::from_mode(mode);
                            let _ = fs::set_permissions(&dest_path, perms);
                        }
                    }
                }
            }
        }

        Ok(())
    }

}

/// Get the runtime cache directory
fn get_runtime_cache_dir() -> Result<PathBuf> {
    let cache_dir = if let Ok(override_dir) = std::env::var("RB_SYS_RUNTIME_CACHE_DIR") {
        PathBuf::from(override_dir)
    } else if let Some(cache_home) = std::env::var_os("XDG_CACHE_HOME") {
        PathBuf::from(cache_home).join("rb-sys/cli")
    } else if let Some(home_dir) = dirs::home_dir() {
        home_dir.join(".cache/rb-sys/cli")
    } else {
        anyhow::bail!("Could not determine cache directory (no HOME or XDG_CACHE_HOME)")
    };

    fs::create_dir_all(&cache_dir)
        .with_context(|| format!("Failed to create cache directory: {}", cache_dir.display()))?;

    Ok(cache_dir)
}

/// Clear the runtime cache
pub fn clear_cache() -> Result<()> {
    let cache_dir = get_runtime_cache_dir()?;
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)
            .with_context(|| format!("Failed to remove cache directory: {}", cache_dir.display()))?;
        println!("✅ Cleared runtime cache: {}", cache_dir.display());
    } else {
        println!("ℹ️  Cache directory does not exist: {}", cache_dir.display());
    }
    Ok(())
}

/// Get the cache directory path
pub fn get_cache_dir() -> Result<PathBuf> {
    get_runtime_cache_dir()
}
