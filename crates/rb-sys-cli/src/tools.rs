use crate::assets::manifest::ToolInfo;
use crate::assets::AssetManager;
use anyhow::{Context, Result};
use std::path::PathBuf;
use tracing::debug;

/// Describe the current host platform in the same format used by the embedded tool manifest
/// (e.g., "x86_64-apple-darwin", "aarch64-unknown-linux-gnu").
pub fn current_host_platform() -> String {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    match (arch, os) {
        ("x86_64", "macos") => "x86_64-apple-darwin".to_string(),
        ("aarch64", "macos") => "aarch64-apple-darwin".to_string(),
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu".to_string(),
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu".to_string(),
        ("x86_64", "windows") => "x86_64-pc-windows-gnu".to_string(),
        ("aarch64", "windows") => "aarch64-pc-windows-gnullvm".to_string(),
        // Fallback to a generic tag for less common hosts
        _ => format!("{arch}-{os}"),
    }
}

/// Resolve the tools in the manifest that match the current host.
pub fn tools_for_host(asset_manager: &AssetManager) -> Vec<&ToolInfo> {
    let host = current_host_platform();
    asset_manager
        .tools()
        .iter()
        .filter(|tool| tool.host_platform == host)
        .collect()
}

/// Find a specific tool for the current host
pub fn find_tool<'a>(asset_manager: &'a AssetManager, tool_name: &str) -> Option<&'a ToolInfo> {
    let host = current_host_platform();
    asset_manager
        .tools()
        .iter()
        .find(|tool| tool.host_platform == host && tool.name == tool_name)
}

/// Extract and return the path to an embedded tool
pub fn extract_tool(asset_manager: &AssetManager, tool_name: &str) -> Result<Option<PathBuf>> {
    let tool = match find_tool(asset_manager, tool_name) {
        Some(t) => t,
        None => {
            debug!(tool = tool_name, host = %current_host_platform(), "No embedded tool found for host");
            return Ok(None);
        }
    };

    let tools_dir = asset_manager.cache_dir().join("tools");
    let extracted_path = asset_manager
        .extract_tool(tool, &tools_dir)
        .with_context(|| format!("Failed to extract tool: {}", tool_name))?;

    Ok(Some(extracted_path))
}
