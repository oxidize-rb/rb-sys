use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Lockfile that records verified fetches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lockfile {
    /// Generated timestamp
    pub generated_at: DateTime<Utc>,
    /// Platforms mapped to their verified assets
    #[serde(flatten)]
    pub platforms: HashMap<String, PlatformLock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformLock {
    /// Assets for this platform
    #[serde(flatten)]
    pub assets: HashMap<String, AssetLock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AssetLock {
    OciExtract {
        /// OCI image reference used
        image: String,
        /// Timestamp when extracted
        verified_at: DateTime<Utc>,
        /// Files extracted with their BLAKE3 digests
        files: Vec<FileDigest>,
    },
    Tarball {
        /// URL downloaded from
        url: String,
        /// Verified digest (algorithm:hex)
        digest: String,
        /// Timestamp when verified
        verified_at: DateTime<Utc>,
        /// Size in bytes
        size_bytes: u64,
    },
    TarballExtract {
        /// URL downloaded from
        url: String,
        /// Verified digest of the extracted file (algorithm:hex)
        digest: String,
        /// Path within tarball where file was found
        source_path: String,
        /// Timestamp when verified
        verified_at: DateTime<Utc>,
        /// Size of extracted file in bytes
        size_bytes: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDigest {
    /// Relative path within the asset
    pub path: String,
    /// BLAKE3 digest (hex)
    pub blake3: String,
    /// File size in bytes
    pub size_bytes: u64,
}

impl Lockfile {
    pub fn new() -> Self {
        Self {
            generated_at: Utc::now(),
            platforms: HashMap::new(),
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read lockfile: {}", path.as_ref().display()))?;

        toml::from_str(&content)
            .with_context(|| format!("Failed to parse lockfile: {}", path.as_ref().display()))
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Failed to serialize lockfile")?;

        std::fs::write(path.as_ref(), content)
            .with_context(|| format!("Failed to write lockfile: {}", path.as_ref().display()))?;

        Ok(())
    }

    pub fn set_asset(&mut self, platform: &str, name: &str, lock: AssetLock) {
        self.platforms
            .entry(platform.to_string())
            .or_insert_with(|| PlatformLock {
                assets: HashMap::new(),
            })
            .assets
            .insert(name.to_string(), lock);
    }

    pub fn get_asset(&self, platform: &str, name: &str) -> Option<&AssetLock> {
        self.platforms
            .get(platform)
            .and_then(|p| p.assets.get(name))
    }
}

impl Default for Lockfile {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockfile_serialization() {
        let mut lockfile = Lockfile::new();
        lockfile.set_asset(
            "x86_64-unknown-linux-gnu",
            "zig",
            AssetLock::Tarball {
                url: "https://example.com/zig.tar.xz".to_string(),
                digest: "sha256:abc123".to_string(),
                verified_at: Utc::now(),
                size_bytes: 1024,
            },
        );

        let toml = toml::to_string(&lockfile).unwrap();
        let parsed: Lockfile = toml::from_str(&toml).unwrap();

        assert!(parsed
            .get_asset("x86_64-unknown-linux-gnu", "zig")
            .is_some());
    }
}
