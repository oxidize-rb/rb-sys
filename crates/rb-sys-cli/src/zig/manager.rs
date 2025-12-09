//! Zig binary manager for cargo-gem.
//!
//! This module provides Zig path resolution for cross-compilation builds.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Resolve the Zig path to use for builds.
///
/// Priority order:
/// 1. Explicit path from --zig-path or ZIG_PATH env var (if provided and not default)
/// 2. Embedded Zig from unified assets
/// 3. System Zig via `which zig`
pub fn resolve_zig_path(explicit_path: Option<&Path>) -> Result<PathBuf> {
    // If explicit path is provided and is not the default "zig", use it
    if let Some(path) = explicit_path {
        let path_str = path.to_string_lossy();
        if path_str != "zig" && !path_str.is_empty() {
            tracing::debug!(
                zig_path = %path.display(),
                "Using explicitly provided Zig path"
            );
            return Ok(path.to_path_buf());
        }
    }

    // Try unified assets embedded Zig
    match try_unified_assets_zig() {
        Ok(Some(path)) => {
            tracing::info!(
                zig_path = %path.display(),
                "Using Zig from embedded assets"
            );
            return Ok(path);
        }
        Ok(None) => {
            // No Zig in unified assets for this host
            tracing::debug!("No Zig found in embedded assets for this host");
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "Failed to extract Zig from embedded assets"
            );
        }
    }

    // Fall back to system Zig
    which::which("zig").context(
        "Zig not found. Please either:\n  \
             1. Install Zig (https://ziglang.org/download/)\n  \
             2. Use a cargo-gem build with embedded Zig\n  \
             3. Set ZIG_PATH to point to your Zig installation",
    )
}

/// Try to extract and use Zig from the unified embedded assets.
fn try_unified_assets_zig() -> Result<Option<PathBuf>> {
    use crate::assets::AssetManager;
    use crate::tools;

    let assets = AssetManager::new()?;
    let zig_path = tools::extract_tool(&assets, "zig")?;

    if let Some(zig_dir) = zig_path {
        // Find the zig executable within the extracted directory
        #[cfg(windows)]
        let zig_exe = zig_dir.join("zig.exe");

        #[cfg(not(windows))]
        let zig_exe = zig_dir.join("zig");

        if zig_exe.exists() {
            return Ok(Some(zig_exe));
        }

        // If zig is not directly in the root, search for it
        for entry in std::fs::read_dir(&zig_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                #[cfg(windows)]
                let candidate = path.join("zig.exe");

                #[cfg(not(windows))]
                let candidate = path.join("zig");

                if candidate.exists() {
                    return Ok(Some(candidate));
                }
            }
        }

        anyhow::bail!(
            "Zig extracted but executable not found in {}",
            zig_dir.display()
        );
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_zig_path_explicit() {
        let explicit = PathBuf::from("/custom/path/to/zig");
        let result = resolve_zig_path(Some(&explicit));
        // Should return the explicit path (even if it doesn't exist, for testing)
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), explicit);
    }
}
