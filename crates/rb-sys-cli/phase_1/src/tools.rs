use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Unified manifest loader - reads from assets_manifest.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetsManifest {
    pub manifest: ManifestMeta,
    #[serde(default)]
    pub common: Vec<Asset>,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestMeta {
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub name: String,
    pub version: String,
    pub host: String,
    pub category: String,
    pub fetch_type: String,
    pub fetch_url: String,
    pub fetch_digest: String,
    #[serde(default)]
    pub fetch_extract: Option<String>,
    #[serde(default = "default_strip_components")]
    pub strip_components: usize,
    pub archive_path: String,
}

fn default_strip_components() -> usize {
    1
}

impl AssetsManifest {
    /// Load from assets_manifest.toml
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read manifest: {}", path.display()))?;

        let manifest: AssetsManifest = toml::from_str(&content)
            .with_context(|| format!("Failed to parse manifest: {}", path.display()))?;

        Ok(manifest)
    }

    /// Get all assets (tools) for a specific host platform
    pub fn tools_for_host(&self, host_platform: &str) -> Vec<&Asset> {
        self.assets
            .iter()
            .filter(|a| a.host == host_platform && a.category == "tool")
            .collect()
    }

    /// Get all tools (from all platforms)
    pub fn all_tools(&self) -> Vec<&Asset> {
        self.assets
            .iter()
            .filter(|a| a.category == "tool")
            .collect()
    }

    /// Convert to RuntimeTool entries for phase_1 manifest generation
    pub fn to_runtime_tools(&self) -> Vec<crate::manifest::RuntimeTool> {
        let mut tools = Vec::new();

        for asset in &self.assets {
            // Only include tools, not SDKs or other assets
            if asset.category == "tool" {
                tools.push(crate::manifest::RuntimeTool {
                    name: asset.name.clone(),
                    version: asset.version.clone(),
                    host_platform: asset.host.clone(),
                    archive_path: asset.archive_path.clone(),
                    blake3: "".to_string(), // Will be populated by phase_0
                    notes: None,
                });
            }
        }

        tools.sort_by(|a, b| {
            a.host_platform
                .cmp(&b.host_platform)
                .then(a.name.cmp(&b.name))
        });

        tools
    }
}

// Legacy ToolsManifest for backward compatibility (deprecated)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deprecated(note = "Use AssetsManifest instead")]
pub struct ToolsManifest {
    pub version: u32,
    pub tools: HashMap<String, HashMap<String, ToolEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[deprecated(note = "Use Asset instead")]
pub struct ToolEntry {
    pub version: String,
    pub blake3: String,
    pub archive_path: String,
    #[serde(default)]
    pub notes: Option<String>,
}
