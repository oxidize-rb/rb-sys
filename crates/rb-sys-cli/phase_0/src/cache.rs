use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Manifest tracking image digests for cache invalidation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub rake_compiler_dock_version: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub platforms: HashMap<String, PlatformInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub ruby_platform: String,
    pub rust_target: String,
    pub image: String,
    pub image_digest: String,
    pub ruby_versions: Vec<String>,
    pub has_sysroot: bool,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub extracted_at: chrono::DateTime<chrono::Utc>,
}

impl Manifest {
    pub fn new() -> Self {
        Self {
            version: 1,
            rake_compiler_dock_version: "1.10.0".to_string(),
            created_at: chrono::Utc::now(),
            platforms: HashMap::new(),
        }
    }

    pub fn get_digest(&self, ruby_platform: &str) -> Option<&String> {
        self.platforms.get(ruby_platform).map(|p| &p.image_digest)
    }

    pub fn set_platform(&mut self, ruby_platform: String, info: PlatformInfo) {
        self.platforms.insert(ruby_platform, info);
    }
}

/// Load manifest from file, or create new if it doesn't exist
pub fn load_manifest(path: &Path) -> Result<Manifest> {
    if path.exists() {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read manifest: {}", path.display()))?;
        let manifest: Manifest = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse manifest: {}", path.display()))?;
        Ok(manifest)
    } else {
        Ok(Manifest::new())
    }
}

/// Save manifest to file
pub fn save_manifest(path: &Path, manifest: &Manifest) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let content = serde_json::to_string_pretty(manifest)
        .context("Failed to serialize manifest")?;
    
    fs::write(path, content)
        .with_context(|| format!("Failed to write manifest: {}", path.display()))?;

    Ok(())
}

/// Get the default cache directory (in the repo root, not ~/.cache)
pub fn get_default_cache_dir() -> Result<PathBuf> {
    // CARGO_MANIFEST_DIR is crates/rb-sys-cli/phase_0
    // We want repo_root/.cache/cli
    let cache_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()  // crates/rb-sys-cli
        .unwrap()
        .parent()  // crates
        .unwrap()
        .parent()  // repo root
        .unwrap()
        .join(".cache/cli");

    fs::create_dir_all(&cache_dir)
        .with_context(|| format!("Failed to create cache directory: {}", cache_dir.display()))?;

    Ok(cache_dir)
}

/// Check if a platform needs to be re-extracted based on digest
pub fn needs_extraction(
    cache_dir: &Path,
    ruby_platform: &str,
    current_digest: &str,
    manifest: &Manifest,
) -> bool {
    // Check if digest changed
    if let Some(cached_digest) = manifest.get_digest(ruby_platform) {
        if cached_digest == current_digest {
            // Digest matches, check if files exist
            let platform_dir = cache_dir.join(ruby_platform);
            let digest_marker = platform_dir.join(".digest");
            
            if digest_marker.exists() {
                if let Ok(stored_digest) = fs::read_to_string(&digest_marker) {
                    return stored_digest.trim() != current_digest;
                }
            }
            
            // Marker doesn't exist or is invalid, need to extract
            return true;
        }
    }
    
    // Digest changed or not in manifest
    true
}

/// Write digest marker file for a platform
pub fn write_digest_marker(cache_dir: &Path, ruby_platform: &str, digest: &str) -> Result<()> {
    let platform_dir = cache_dir.join(ruby_platform);
    fs::create_dir_all(&platform_dir)
        .with_context(|| format!("Failed to create platform directory: {}", platform_dir.display()))?;
    
    let marker_path = platform_dir.join(".digest");
    fs::write(&marker_path, digest)
        .with_context(|| format!("Failed to write digest marker: {}", marker_path.display()))?;
    
    Ok(())
}
