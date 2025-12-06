use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Configuration loaded from data/derived/rb-sys-cli.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub toolchains: Vec<Toolchain>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Toolchain {
    #[serde(rename = "ruby-platform")]
    pub ruby_platform: String,

    #[serde(rename = "rust-target")]
    pub rust_target: String,

    #[serde(rename = "sysroot-paths")]
    pub sysroot_paths: Vec<String>,

    pub oci: OciInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciInfo {
    pub tag: String,
    pub digest: String,
    pub reference: String,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        let config: Config = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        
        Ok(config)
    }
}
