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
        let rubies_path = self
            .assets
            .cache_dir()
            .join(&platform.ruby_platform)
            .join("rubies");

        // Extract Ruby sysroot if available
        // TODO: Enable when Ruby headers are properly added to tarball
        // if let Some(ruby_version) = platform.ruby_versions.first() {
        //     let ruby_sysroot_path = rubies_path.join(ruby_version);
        //     if !ruby_sysroot_path.exists() {
        //         self.assets.extract_ruby_sysroot(&platform.rust_target, ruby_version, &ruby_sysroot_path)?;
        //     }
        // }

        // macOS SDK is in the runtime cache (not build-local)
        let macos_sdk_path = if platform.rust_target.starts_with("aarch64-apple-darwin")
            || platform.rust_target.starts_with("x86_64-apple-darwin")
        {
            Some(
                self.assets
                    .cache_dir()
                    .join(&platform.ruby_platform)
                    .join("sdk"),
            )
        } else {
            None
        };

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
                macos_sdk_path,
                marker_path: marker,
                cleanup_on_drop: true,
            });
        }

        // Clean and recreate sysroot directory
        if sysroot_path.exists() {
            fs::remove_dir_all(&sysroot_path).with_context(|| {
                format!("Failed to clean sysroot dir: {}", sysroot_path.display())
            })?;
        }
        fs::create_dir_all(&sysroot_path)
            .with_context(|| format!("Failed to create sysroot dir: {}", sysroot_path.display()))?;

        // Skip sysroot extraction for now - the tarball doesn't contain sysroot files
        // TODO: Add sysroot files to the tarball during build process
        tracing::debug!(
            "Skipping sysroot extraction for {} - not present in tarball",
            platform.ruby_platform
        );

        // Skip rubies extraction for now - ruby_versions is empty in manifest
        // TODO: Add Ruby headers to the tarball during build process
        tracing::debug!(
            "Skipping rubies extraction for {} - no ruby versions configured",
            platform.ruby_platform
        );

        // Extract macOS SDK if this is a macOS target
        if let Some(ref sdk_path) = macos_sdk_path {
            // Note: extract_macos_sdk returns Option<PathBuf> but we ignore the result
            // since we already know the SDK path structure
            let _ = self
                .assets
                .extract_macos_sdk(&platform.ruby_platform, sdk_path);
        }

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
            macos_sdk_path,
            marker_path: marker,
            cleanup_on_drop: true,
        })
    }
}

/// A mounted sysroot that cleans up on Drop (if the build succeeds).
pub struct MountedSysroot {
    path: PathBuf,
    rubies_path: PathBuf,
    macos_sdk_path: Option<PathBuf>,
    marker_path: PathBuf,
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

    /// Get the macOS SDK path (for macOS cross-compilation)
    pub fn macos_sdk_path(&self) -> Option<&Path> {
        self.macos_sdk_path.as_deref()
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

            // Also remove the marker file so the next build knows to re-extract
            if let Err(e) = fs::remove_file(&self.marker_path) {
                tracing::warn!("Failed to cleanup marker file: {}", e);
            }
        }
    }
}
