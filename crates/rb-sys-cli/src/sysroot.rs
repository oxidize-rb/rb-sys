use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::assets::AssetManager;

/// Manages sysroots for hermetic builds.
pub struct SysrootManager {
    assets: AssetManager,
}

impl SysrootManager {
    pub fn new(assets: AssetManager) -> Self {
        Self { assets }
    }

    /// Extracts the sysroot for this rust target into a build-local directory.
    /// Returns a guard that cleans up on Drop (on success).
    pub fn mount(&self, rust_target: &str, target_dir: &Path) -> Result<MountedSysroot> {
        let platform = self
            .assets
            .manifest()
            .platform_for_rust_target(rust_target)?;

        // Build-local sysroot: target/rb-sys/<rust-target>/sysroot
        let build_root = target_dir.join("rb-sys").join(rust_target);
        let sysroot_path = build_root.join("sysroot");

        // Ruby headers are in the runtime cache (not build-local)
        let rubies_path = self.assets.cache_dir().join(&platform.ruby_platform).join("rubies");

        // Check if already extracted (marker file)
        let marker = build_root.join(".sysroot-extracted");
        if marker.exists() {
            tracing::debug!(
                "Sysroot for {} already extracted at {}",
                rust_target,
                sysroot_path.display()
            );
            return Ok(MountedSysroot {
                path: sysroot_path,
                rubies_path,
                cleanup_on_drop: true,
            });
        }

        // Clean and recreate sysroot directory
        if sysroot_path.exists() {
            fs::remove_dir_all(&sysroot_path)
                .with_context(|| format!("Failed to clean sysroot dir: {}", sysroot_path.display()))?;
        }
        fs::create_dir_all(&sysroot_path)
            .with_context(|| format!("Failed to create sysroot dir: {}", sysroot_path.display()))?;

        // Extract sysroot files directly from embedded tarball
        self.assets
            .extract_sysroot(&platform.ruby_platform, &sysroot_path)?;

        // Write marker file
        fs::create_dir_all(&build_root)?;
        fs::write(&marker, "")?;

        tracing::debug!(
            "Extracted sysroot for {} to {}",
            rust_target,
            sysroot_path.display()
        );

        Ok(MountedSysroot {
            path: sysroot_path,
            rubies_path,
            cleanup_on_drop: true,
        })
    }
}

/// A mounted sysroot that cleans up on Drop (if the build succeeds).
pub struct MountedSysroot {
    path: PathBuf,
    rubies_path: PathBuf,
    cleanup_on_drop: bool,
}

impl MountedSysroot {
    /// Get the sysroot path (for libs, headers like OpenSSL)
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the rubies path (for Ruby headers)
    pub fn rubies_path(&self) -> &Path {
        &self.rubies_path
    }

    /// Call this on build failure to keep the sysroot around for inspection.
    pub fn keep(mut self) {
        self.cleanup_on_drop = false;
        // consume self
    }
}

impl Drop for MountedSysroot {
    fn drop(&mut self) {
        if self.cleanup_on_drop {
            if let Err(e) = fs::remove_dir_all(&self.path) {
                tracing::warn!("Failed to cleanup sysroot: {}", e);
            } else {
                tracing::debug!("Cleaned up sysroot at {}", self.path.display());
            }
        }
    }
}
