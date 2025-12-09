use crate::blake3_hash::Blake3Hash;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Manifest embedded in the binary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub rake_compiler_dock_version: String,
    pub platforms: HashMap<String, PlatformInfo>,
    #[serde(default)]
    pub tools: Vec<ToolInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub ruby_platform: String,
    pub rust_target: String,
    pub image: String,
    pub image_digest: String,
    pub ruby_versions: Vec<String>,
    pub has_sysroot: bool,
    #[serde(default)]
    pub ruby_sysroot_archive: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    /// Host platform that this tool binary targets (e.g., x86_64-apple-darwin)
    pub host_platform: String,
    /// Relative path within the embedded archive where the tool payload lives
    pub archive_path: String,
    /// BLAKE3 hash of the tool archive for integrity verification
    pub blake3: Blake3Hash,
    /// Optional notes about the tool payload
    #[serde(default)]
    pub notes: Option<String>,
}

impl Manifest {
    /// Find platform info by rust target
    pub fn platform_for_rust_target(&self, rust_target: &str) -> Result<&PlatformInfo> {
        self.platforms
            .values()
            .find(|p| p.rust_target == rust_target)
            .with_context(|| format!("No platform found for rust target: {rust_target}"))
    }
}
