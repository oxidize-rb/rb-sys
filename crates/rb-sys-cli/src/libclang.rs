///! libclang manager for bundled libclang distribution.
///!
///! This module manages the bundled libclang runtime that is embedded in
///! cargo-gem for use with bindgen during cross-compilation.
use anyhow::{Context, Result};
use std::path::PathBuf;
use tracing::{debug, info};

use crate::assets::AssetManager;
use crate::tools;

/// Try to extract and configure libclang from embedded assets.
///
/// Returns the path to the libclang library directory if available.
/// This path should be set as LIBCLANG_PATH for bindgen.
pub fn try_embedded_libclang() -> Result<Option<PathBuf>> {
    let assets = AssetManager::new()?;
    let libclang_path = tools::extract_tool(&assets, "libclang")?;

    if let Some(libclang_dir) = libclang_path {
        // libclang directory structure varies by platform:
        // - Linux: lib/libclang.so
        // - macOS: lib/libclang.dylib
        // - Windows: bin/libclang.dll

        let lib_dir = libclang_dir.join("lib");
        if lib_dir.exists() {
            debug!(libclang_path = %lib_dir.display(), "Found embedded libclang");
            return Ok(Some(lib_dir));
        }

        let bin_dir = libclang_dir.join("bin");
        if bin_dir.exists() {
            debug!(libclang_path = %bin_dir.display(), "Found embedded libclang in bin/");
            return Ok(Some(bin_dir));
        }

        info!(
            path = %libclang_dir.display(),
            "libclang extracted but lib/ or bin/ directory not found"
        );
    }

    Ok(None)
}

/// Configure environment variables for bindgen to use embedded libclang.
///
/// This sets LIBCLANG_PATH if embedded libclang is available and not already set.
pub fn configure_bindgen_env(
    env_vars: &mut std::collections::HashMap<String, String>,
) -> Result<()> {
    // Only set if not already provided by user
    if env_vars.contains_key("LIBCLANG_PATH") {
        debug!("LIBCLANG_PATH already set, skipping embedded libclang");
        return Ok(());
    }

    if let Ok(Some(libclang_path)) = try_embedded_libclang() {
        let path_str = libclang_path.display().to_string();
        env_vars.insert("LIBCLANG_PATH".to_string(), path_str.clone());
        info!(libclang_path = %path_str, "Configured bindgen to use embedded libclang");
    } else {
        debug!("No embedded libclang available, bindgen will use system libclang");
    }

    Ok(())
}
