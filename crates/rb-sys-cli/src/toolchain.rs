use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Toolchain information from toolchains.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainInfo {
    #[serde(rename = "ruby-platform")]
    pub ruby_platform: String,

    #[serde(rename = "rust-target")]
    pub rust_target: String,

    #[serde(default)]
    pub aliases: Vec<String>,

    #[serde(default = "default_true")]
    pub supported: bool,

    #[serde(rename = "rake-compiler-dock", default)]
    pub rake_compiler_dock: Option<HashMap<String, String>>,
}

fn default_true() -> bool {
    true
}

impl ToolchainInfo {
    /// Load all toolchains from the embedded JSON
    pub fn load_all() -> Result<Vec<ToolchainInfo>> {
        let json = include_str!("../../../data/toolchains.json");
        let data: serde_json::Value = serde_json::from_str(json)?;

        let toolchains = data["toolchains"]
            .as_array()
            .context("Missing toolchains array")?;

        let mut result = Vec::new();
        for tc in toolchains {
            result.push(serde_json::from_value(tc.clone())?);
        }

        Ok(result)
    }

    /// Find a toolchain by Rust target triple
    pub fn find_by_rust_target(rust_target: &str) -> Result<ToolchainInfo> {
        let toolchains = Self::load_all()?;

        toolchains
            .into_iter()
            .find(|tc| {
                tc.rust_target == rust_target
                    || tc.aliases.iter().any(|alias| alias == rust_target)
            })
            .with_context(|| format!("No toolchain found for Rust target: {}", rust_target))
    }

    /// Find a toolchain by Ruby platform
    pub fn find_by_ruby_platform(ruby_platform: &str) -> Result<ToolchainInfo> {
        let toolchains = Self::load_all()?;

        toolchains
            .into_iter()
            .find(|tc| tc.ruby_platform == ruby_platform)
            .with_context(|| format!("No toolchain found for Ruby platform: {}", ruby_platform))
    }

    /// Get the Zig target triple (strips 'unknown' vendor)
    pub fn zig_target(&self) -> String {
        self.rust_target.replace("-unknown-", "-")
    }

    /// List all supported platforms
    pub fn list_supported() -> Result<Vec<ToolchainInfo>> {
        let toolchains = Self::load_all()?;
        Ok(toolchains
            .into_iter()
            .filter(|tc| tc.supported)
            .collect())
    }
}

/// Detect the current host platform's Rust target triple
pub fn detect_host_target() -> String {
    // Get the target triple from the build environment
    std::env::var("TARGET").unwrap_or_else(|_| {
        // Fallback to detecting based on OS and architecture
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        match (os, arch) {
            ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
            ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
            ("linux", "arm") => "arm-unknown-linux-gnueabihf",
            ("macos", "x86_64") => "x86_64-apple-darwin",
            ("macos", "aarch64") => "aarch64-apple-darwin",
            ("windows", "x86_64") => "x86_64-pc-windows-msvc",
            ("windows", "i686") => "i686-pc-windows-msvc",
            _ => panic!("Unsupported host platform: {} {}", os, arch),
        }
        .to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_all_toolchains() {
        let toolchains = ToolchainInfo::load_all().unwrap();
        assert!(!toolchains.is_empty());
    }

    #[test]
    fn test_find_by_rust_target() {
        let tc = ToolchainInfo::find_by_rust_target("x86_64-unknown-linux-gnu").unwrap();
        assert_eq!(tc.rust_target, "x86_64-unknown-linux-gnu");
    }

    #[test]
    fn test_zig_target() {
        let tc = ToolchainInfo::find_by_rust_target("x86_64-unknown-linux-gnu").unwrap();
        let zig_target = tc.zig_target();
        assert_eq!(zig_target, "x86_64-linux-gnu");
        assert!(!zig_target.contains("unknown"));
    }

    #[test]
    fn test_list_supported() {
        let supported = ToolchainInfo::list_supported().unwrap();
        assert!(!supported.is_empty());
        assert!(supported.iter().all(|tc| tc.supported));
    }
}
