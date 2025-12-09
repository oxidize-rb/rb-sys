use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::config::Config;

/// Runtime manifest (normalized, deterministic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeManifest {
    pub version: u32,
    pub rake_compiler_dock_version: String,
    pub platforms: BTreeMap<String, PlatformInfo>,
    #[serde(default)]
    pub tools: Vec<RuntimeTool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub ruby_platform: String,
    pub rust_target: String,
    pub image: String,
    pub image_digest: String,
    pub ruby_versions: Vec<String>,
    pub has_sysroot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeTool {
    pub name: String,
    pub version: String,
    pub host_platform: String,
    pub archive_path: String,
    pub blake3: String, // Will be converted to Blake3Hash at runtime
    #[serde(default)]
    pub notes: Option<String>,
}

/// Phase 0 build manifest (with timestamps)
#[derive(Debug, Clone, Deserialize)]
struct BuildManifest {
    platforms: BTreeMap<String, BuildPlatformInfo>,
}

#[derive(Debug, Clone, Deserialize)]
struct BuildPlatformInfo {
    ruby_platform: String,
    rust_target: String,
    image: String,
    image_digest: String,
    ruby_versions: Vec<String>,
    has_sysroot: bool,
}

pub fn generate_manifest(config_path: &Path, cache_dir: &Path, derived_dir: &Path) -> Result<()> {
    let config = Config::load(config_path)?;

    // Load phase_0 build manifest
    let build_manifest_path = cache_dir.join("manifest.json");
    let build_manifest_content = fs::read_to_string(&build_manifest_path).with_context(|| {
        format!(
            "Failed to read build manifest: {}",
            build_manifest_path.display()
        )
    })?;
    let build_manifest: BuildManifest = serde_json::from_str(&build_manifest_content)
        .with_context(|| "Failed to parse build manifest".to_string())?;

    // Create normalized runtime manifest
    let mut platforms = BTreeMap::new();

    for toolchain in &config.toolchains {
        if let Some(build_info) = build_manifest.platforms.get(&toolchain.ruby_platform) {
            platforms.insert(
                toolchain.ruby_platform.clone(),
                PlatformInfo {
                    ruby_platform: build_info.ruby_platform.clone(),
                    rust_target: build_info.rust_target.clone(),
                    image: build_info.image.clone(),
                    image_digest: build_info.image_digest.clone(),
                    ruby_versions: build_info.ruby_versions.clone(),
                    has_sysroot: build_info.has_sysroot,
                },
            );
        }
    }

    // Load tools manifest if present
    let tools_manifest_path = Path::new("data/tools.json");
    let tools = if tools_manifest_path.exists() {
        let tools_manifest = crate::tools::ToolsManifest::load(tools_manifest_path)?;
        tools_manifest.to_runtime_tools()
    } else {
        Vec::new()
    };

    let runtime_manifest = RuntimeManifest {
        version: 1,
        rake_compiler_dock_version: "1.10.0".to_string(),
        platforms,
        tools,
    };

    // Write to derived directory (checked in)
    fs::create_dir_all(derived_dir)
        .with_context(|| format!("Failed to create directory: {}", derived_dir.display()))?;

    let dest_path = derived_dir.join("rb-sys-cli-manifest.json");
    let content = serde_json::to_string_pretty(&runtime_manifest)
        .context("Failed to serialize runtime manifest")?;

    fs::write(&dest_path, content)
        .with_context(|| format!("Failed to write {}", dest_path.display()))?;

    Ok(())
}
