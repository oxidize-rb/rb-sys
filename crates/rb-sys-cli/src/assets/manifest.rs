use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Manifest embedded in the binary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub rake_compiler_dock_version: String,
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
}

impl Manifest {
    /// Find platform info by rust target
    pub fn platform_for_rust_target(&self, rust_target: &str) -> Result<&PlatformInfo> {
        self.platforms
            .values()
            .find(|p| p.rust_target == rust_target)
            .with_context(|| format!("No platform found for rust target: {rust_target}"))
    }

    /// Find platform info by ruby platform
    pub fn platform_for_ruby_platform(&self, ruby_platform: &str) -> Result<&PlatformInfo> {
        self.platforms
            .get(ruby_platform)
            .with_context(|| format!("No platform found for ruby platform: {ruby_platform}"))
    }
}
