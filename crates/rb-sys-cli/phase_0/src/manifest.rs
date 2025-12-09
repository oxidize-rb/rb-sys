use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Unified manifest structure (replaces both phase_0_manifest.toml and tools.json)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Phase0Manifest {
    pub manifest: ManifestMeta,
    #[serde(default)]
    pub common: Vec<Asset>,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ManifestMeta {
    pub version: u32,
}

/// Unified asset definition (fetch + runtime metadata)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Asset {
    /// Asset name (e.g., "zig", "libclang")
    pub name: String,
    /// Version string
    pub version: String,
    /// Host platform (Rust target triple)
    pub host: String,
    /// Category: "tool", "sdk", "ruby", etc.
    pub category: String,

    // Fetch specification (input to phase_0)
    pub fetch_type: String,
    pub fetch_url: String,
    pub fetch_digest: String,
    #[serde(default)]
    pub fetch_extract: Option<String>,
    #[serde(default = "default_strip_components")]
    pub strip_components: usize,

    // Runtime specification (used by phase_1 and runtime)
    pub archive_path: String,
}

fn default_strip_components() -> usize {
    1
}

/// Convert Asset to legacy AssetRequest for fetcher compatibility
#[derive(Debug, Clone)]
pub enum AssetRequest {
    Tarball {
        name: String,
        url: String,
        digest: String,
        strip_components: usize,
    },
    TarballExtract {
        name: String,
        url: String,
        extract: String,
        digest: String,
        strip_components: usize,
    },
    OciExtract {
        name: String,
        image: String,
        items: Vec<String>,
        strip_prefix: Option<String>,
    },
}

impl AssetRequest {
    pub fn name(&self) -> &str {
        match self {
            AssetRequest::OciExtract { name, .. } => name,
            AssetRequest::Tarball { name, .. } => name,
            AssetRequest::TarballExtract { name, .. } => name,
        }
    }

    pub fn asset_type(&self) -> &str {
        match self {
            AssetRequest::OciExtract { .. } => "oci_extract",
            AssetRequest::Tarball { .. } => "tarball",
            AssetRequest::TarballExtract { .. } => "tarball_extract",
        }
    }
}

impl Asset {
    /// Convert to AssetRequest for phase_0 processing
    pub fn to_request(&self) -> Result<AssetRequest> {
        match self.fetch_type.as_str() {
            "tarball" => Ok(AssetRequest::Tarball {
                name: self.name.clone(),
                url: self.fetch_url.clone(),
                digest: self.fetch_digest.clone(),
                strip_components: self.strip_components,
            }),
            "tarball_extract" => {
                let extract = self
                    .fetch_extract
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("fetch_extract required for tarball_extract"))?;
                Ok(AssetRequest::TarballExtract {
                    name: self.name.clone(),
                    url: self.fetch_url.clone(),
                    extract,
                    digest: self.fetch_digest.clone(),
                    strip_components: self.strip_components,
                })
            }
            "oci_extract" => {
                // Parse OCI-specific fields when implemented
                anyhow::bail!("oci_extract not yet implemented in unified manifest")
            }
            other => anyhow::bail!("Unknown fetch_type: {}", other),
        }
    }
}

impl Phase0Manifest {
    /// Load manifest from TOML file
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read manifest: {}", path.as_ref().display()))?;

        let manifest: Phase0Manifest = toml::from_str(&content)
            .with_context(|| format!("Failed to parse manifest: {}", path.as_ref().display()))?;

        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate manifest structure and constraints
    fn validate(&self) -> Result<()> {
        // Group by platform for validation
        let mut by_platform: HashMap<&str, Vec<&Asset>> = HashMap::new();

        for asset in &self.assets {
            by_platform.entry(&asset.host).or_default().push(asset);
        }

        // Check for duplicate asset names within each platform
        for (platform, assets) in &by_platform {
            let mut seen = std::collections::HashSet::new();
            for asset in assets {
                let key = (&asset.name, &asset.category);
                if !seen.insert(key) {
                    anyhow::bail!(
                        "Duplicate asset '{}/{}' in platform '{}'",
                        asset.name,
                        asset.category,
                        platform
                    );
                }
            }

            // Validate asset-specific constraints
            for asset in assets {
                // Digest must be in format "algorithm:hex"
                if !asset.fetch_digest.contains(':') {
                    anyhow::bail!(
                        "Asset '{}' digest '{}' must be in format 'algorithm:hex'",
                        asset.name,
                        asset.fetch_digest
                    );
                }
                let parts: Vec<&str> = asset.fetch_digest.split(':').collect();
                if parts.len() != 2 {
                    anyhow::bail!(
                        "Asset '{}' digest '{}' must have exactly one colon separator",
                        asset.name,
                        asset.fetch_digest
                    );
                }
                let algorithm = parts[0];
                let supported = ["sha256", "sha512", "blake3"];
                if !supported.contains(&algorithm) {
                    anyhow::bail!(
                        "Asset '{}' has unsupported digest algorithm '{}'. Supported: {:?}",
                        asset.name,
                        algorithm,
                        supported
                    );
                }

                // Validate fetch_type specific fields
                match asset.fetch_type.as_str() {
                    "tarball_extract" => {
                        if asset.fetch_extract.is_none() {
                            anyhow::bail!(
                                "Asset '{}' with fetch_type 'tarball_extract' must have fetch_extract field",
                                asset.name
                            );
                        }
                    }
                    "tarball" | "oci_extract" => {}
                    other => {
                        anyhow::bail!("Asset '{}' has unknown fetch_type: {}", asset.name, other)
                    }
                }
            }
        }

        Ok(())
    }

    /// Get assets for a specific platform, including common assets
    pub fn assets_for_platform(&self, platform: &str) -> Vec<AssetRequest> {
        let mut requests = Vec::new();

        // Add common assets first
        for asset in &self.common {
            if let Ok(req) = asset.to_request() {
                requests.push(req);
            }
        }

        // Add platform-specific assets
        for asset in &self.assets {
            if asset.host == platform {
                if let Ok(req) = asset.to_request() {
                    requests.push(req);
                }
            }
        }

        requests
    }

    /// Get all platforms (unique host values)
    pub fn platforms(&self) -> Vec<String> {
        let mut platforms: Vec<String> = self
            .assets
            .iter()
            .map(|a| a.host.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        platforms.sort();
        platforms
    }

    /// Get all assets for a platform with full metadata
    pub fn assets_with_metadata(&self, platform: &str) -> Vec<&Asset> {
        self.assets.iter().filter(|a| a.host == platform).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tarball() {
        let toml = r#"
        [manifest]
        version = 1
        
        [[assets]]
        name = "zig"
        version = "0.15.2"
        host = "x86_64-unknown-linux-gnu"
        category = "tool"
        fetch_type = "tarball"
        fetch_url = "https://example.com/zig.tar.xz"
        fetch_digest = "sha256:abc123"
        archive_path = "tools/x86_64-unknown-linux-gnu/zig.tar.zst"
        "#;

        let manifest: Phase0Manifest = toml::from_str(toml).unwrap();
        let assets = manifest.assets_for_platform("x86_64-unknown-linux-gnu");
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].name(), "zig");
    }

    #[test]
    fn test_parse_tarball_extract() {
        let toml = r#"
        [manifest]
        version = 1
        
        [[assets]]
        name = "libclang"
        version = "19.1.5"
        host = "x86_64-unknown-linux-gnu"
        category = "tool"
        fetch_type = "tarball_extract"
        fetch_url = "https://example.com/llvm.tar.xz"
        fetch_extract = "**/lib/libclang.so*"
        fetch_digest = "sha256:def456"
        archive_path = "tools/x86_64-unknown-linux-gnu/libclang.tar.zst"
        "#;

        let manifest: Phase0Manifest = toml::from_str(toml).unwrap();
        let assets = manifest.assets_for_platform("x86_64-unknown-linux-gnu");
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].name(), "libclang");
    }

    #[test]
    fn test_common_assets() {
        let toml = r#"
        [manifest]
        version = 1
        
        [[common]]
        name = "docs"
        version = "1.0"
        host = "any"
        category = "docs"
        fetch_type = "tarball"
        fetch_url = "https://example.com/docs.tar.xz"
        fetch_digest = "sha256:abc123"
        archive_path = "docs/common.tar.zst"
        
        [[assets]]
        name = "zig"
        version = "0.15.2"
        host = "x86_64-unknown-linux-gnu"
        category = "tool"
        fetch_type = "tarball"
        fetch_url = "https://example.com/zig.tar.xz"
        fetch_digest = "sha256:def456"
        archive_path = "tools/x86_64-unknown-linux-gnu/zig.tar.zst"
        "#;

        let manifest: Phase0Manifest = toml::from_str(toml).unwrap();
        let assets = manifest.assets_for_platform("x86_64-unknown-linux-gnu");
        assert_eq!(assets.len(), 2);
        assert_eq!(assets[0].name(), "docs"); // common first
        assert_eq!(assets[1].name(), "zig");
    }

    #[test]
    fn test_platforms_list() {
        let toml = r#"
        [manifest]
        version = 1
        
        [[assets]]
        name = "zig"
        version = "0.15.2"
        host = "x86_64-unknown-linux-gnu"
        category = "tool"
        fetch_type = "tarball"
        fetch_url = "https://example.com/zig.tar.xz"
        fetch_digest = "sha256:abc123"
        archive_path = "tools/x86_64-unknown-linux-gnu/zig.tar.zst"
        
        [[assets]]
        name = "zig"
        version = "0.15.2"
        host = "aarch64-apple-darwin"
        category = "tool"
        fetch_type = "tarball"
        fetch_url = "https://example.com/zig.tar.xz"
        fetch_digest = "sha256:def456"
        archive_path = "tools/aarch64-apple-darwin/zig.tar.zst"
        "#;

        let manifest: Phase0Manifest = toml::from_str(toml).unwrap();
        let platforms = manifest.platforms();
        assert_eq!(platforms.len(), 2);
        assert!(platforms.contains(&"x86_64-unknown-linux-gnu".to_string()));
        assert!(platforms.contains(&"aarch64-apple-darwin".to_string()));
    }

    #[test]
    fn test_validation_duplicate_names() {
        let toml = r#"
        [manifest]
        version = 1
        
        [[assets]]
        name = "zig"
        version = "0.15.2"
        host = "x86_64-unknown-linux-gnu"
        category = "tool"
        fetch_type = "tarball"
        fetch_url = "https://example.com/zig1.tar.xz"
        fetch_digest = "sha256:abc123"
        archive_path = "tools/x86_64-unknown-linux-gnu/zig.tar.zst"
        
        [[assets]]
        name = "zig"
        version = "0.15.3"
        host = "x86_64-unknown-linux-gnu"
        category = "tool"
        fetch_type = "tarball"
        fetch_url = "https://example.com/zig2.tar.xz"
        fetch_digest = "sha256:def456"
        archive_path = "tools/x86_64-unknown-linux-gnu/zig2.tar.zst"
        "#;

        let manifest: Phase0Manifest = toml::from_str(toml).unwrap();
        assert!(manifest.validate().is_err());
    }
}
