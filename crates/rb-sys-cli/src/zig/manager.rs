//! Zig binary manager for bundled Zig distribution.
//!
//! This module manages the bundled Zig compiler binary that is embedded in the
//! cargo-gem binary. It handles lazy extraction to a cache directory and provides
//! the path to the Zig executable for cross-compilation.

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Pinned Zig version bundled with cargo-gem
pub const ZIG_VERSION: &str = "0.15.2";

/// Host platform key for the current compilation target.
/// This is set at compile time based on the target triple.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub const HOST_PLATFORM: &str = "x86_64-linux";

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub const HOST_PLATFORM: &str = "aarch64-linux";

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub const HOST_PLATFORM: &str = "x86_64-macos";

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub const HOST_PLATFORM: &str = "aarch64-macos";

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub const HOST_PLATFORM: &str = "x86_64-windows";

// Fallback for unsupported platforms (will fail at runtime with a clear error)
#[cfg(not(any(
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "linux", target_arch = "aarch64"),
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "windows", target_arch = "x86_64"),
)))]
pub const HOST_PLATFORM: &str = "unsupported";

/// Embedded Zig binary (compressed with zstd).
/// Only available when the `bundled-zig` feature is enabled.
#[cfg(feature = "bundled-zig")]
mod embedded {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    pub static ZIG_ARCHIVE: &[u8] = include_bytes!("../embedded/tools/x86_64-linux/zig.tar.zst");

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    pub static ZIG_ARCHIVE: &[u8] = include_bytes!("../embedded/tools/aarch64-linux/zig.tar.zst");

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    pub static ZIG_ARCHIVE: &[u8] = include_bytes!("../embedded/tools/x86_64-macos/zig.tar.zst");

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    pub static ZIG_ARCHIVE: &[u8] = include_bytes!("../embedded/tools/aarch64-macos/zig.tar.zst");

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    pub static ZIG_ARCHIVE: &[u8] = include_bytes!("../embedded/tools/x86_64-windows/zig.tar.zst");

    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64"),
    )))]
    pub static ZIG_ARCHIVE: &[u8] = &[];
}

/// Zig tool information from the embedded manifest
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ZigManifest {
    pub version: String,
    pub host: String,
    pub sha256: String,
    #[serde(default)]
    pub executable: String,
}

/// Combined manifest for all platforms
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ZigManifestFile {
    pub version: String,
    pub platforms: std::collections::HashMap<String, ZigManifest>,
}

/// Manages the bundled Zig binary.
///
/// ZigManager handles:
/// - Lazy extraction of the embedded Zig binary to the cache directory
/// - Version verification and cache invalidation
/// - Providing the path to the Zig executable
pub struct ZigManager {
    cache_dir: PathBuf,
}

impl ZigManager {
    /// Create a new ZigManager with the default cache directory.
    pub fn new() -> Result<Self> {
        let cache_dir = get_tools_cache_dir()?;
        Ok(Self { cache_dir })
    }

    /// Check if Zig is bundled in this build.
    pub fn is_bundled() -> bool {
        #[cfg(feature = "bundled-zig")]
        {
            !embedded::ZIG_ARCHIVE.is_empty()
        }
        #[cfg(not(feature = "bundled-zig"))]
        {
            false
        }
    }

    /// Get the path to the Zig executable, extracting if necessary.
    ///
    /// This will:
    /// 1. Check if Zig is already extracted and matches the expected version
    /// 2. If not, extract the embedded Zig binary to the cache
    /// 3. Verify the extraction was successful
    /// 4. Return the path to the zig executable
    pub fn ensure(&self) -> Result<PathBuf> {
        if !Self::is_bundled() {
            bail!(
                "Zig is not bundled in this cargo-gem build.\n\n\
                 This build was compiled without the embedded Zig toolchain.\n\
                 Please either:\n  \
                 1. Install Zig manually and set ZIG_PATH or --zig-path\n  \
                 2. Use a cargo-gem build that includes the bundled Zig"
            );
        }

        if HOST_PLATFORM == "unsupported" {
            bail!(
                "Bundled Zig is not available for this host platform.\n\n\
                 cargo-gem bundles Zig for:\n  \
                 - Linux x86_64 and aarch64\n  \
                 - macOS x86_64 and aarch64\n  \
                 - Windows x86_64\n\n\
                 Please install Zig manually and set ZIG_PATH or --zig-path"
            );
        }

        let zig_dir = self.zig_install_dir();
        let zig_exe = self.zig_executable_path();
        let marker_path = zig_dir.join(".extracted");

        // Check if already extracted with correct version
        if marker_path.exists() && zig_exe.exists() {
            if let Ok(marker_content) = fs::read_to_string(&marker_path) {
                if marker_content.trim() == ZIG_VERSION {
                    tracing::debug!(
                        zig_path = %zig_exe.display(),
                        version = ZIG_VERSION,
                        "Using cached Zig"
                    );
                    return Ok(zig_exe);
                }
            }
            // Version mismatch - remove old installation
            tracing::info!("Zig version changed, re-extracting");
            let _ = fs::remove_dir_all(&zig_dir);
        }

        // Extract the embedded Zig
        tracing::info!(
            version = ZIG_VERSION,
            host = HOST_PLATFORM,
            dest = %zig_dir.display(),
            "Extracting bundled Zig"
        );

        self.extract_zig(&zig_dir)?;

        // Write version marker
        fs::write(&marker_path, ZIG_VERSION)
            .context("Failed to write Zig version marker")?;

        // Verify the executable exists
        if !zig_exe.exists() {
            bail!(
                "Zig extraction succeeded but executable not found at: {}\n\
                 This is a bug in cargo-gem. Please report it.",
                zig_exe.display()
            );
        }

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o755);
            fs::set_permissions(&zig_exe, perms)
                .context("Failed to make Zig executable")?;
        }

        tracing::info!(
            zig_path = %zig_exe.display(),
            "Zig extracted successfully"
        );

        Ok(zig_exe)
    }

    /// Get the installation directory for Zig.
    fn zig_install_dir(&self) -> PathBuf {
        self.cache_dir
            .join("zig")
            .join(ZIG_VERSION)
            .join(HOST_PLATFORM)
    }

    /// Get the path to the Zig executable.
    fn zig_executable_path(&self) -> PathBuf {
        let zig_dir = self.zig_install_dir();
        
        #[cfg(windows)]
        {
            zig_dir.join("zig.exe")
        }
        
        #[cfg(not(windows))]
        {
            zig_dir.join("zig")
        }
    }

    /// Extract the embedded Zig archive to the destination directory.
    #[cfg(feature = "bundled-zig")]
    fn extract_zig(&self, dest_dir: &Path) -> Result<()> {
        // Create destination directory
        fs::create_dir_all(dest_dir)
            .with_context(|| format!("Failed to create Zig directory: {}", dest_dir.display()))?;

        // Decompress zstd
        let decoder = zstd::Decoder::new(embedded::ZIG_ARCHIVE)
            .context("Failed to create zstd decoder for Zig archive")?;

        // Extract tar with strip-components=1 to remove the zig-<platform>-<version>/ prefix
        let mut archive = tar::Archive::new(decoder);
        archive.set_overwrite(true); // Allow overwriting in case of partial extraction
        
        for entry in archive.entries().context("Failed to read Zig archive entries")? {
            let mut entry = entry.context("Failed to read Zig archive entry")?;
            
            // Get the path and strip the first component
            let path = entry.path().context("Failed to get entry path")?;
            let stripped_path: PathBuf = path.components().skip(1).collect();
            
            if stripped_path.as_os_str().is_empty() {
                continue; // Skip the root directory entry
            }
            
            let dest_path = dest_dir.join(&stripped_path);
            let entry_type = entry.header().entry_type();
            
            if entry_type.is_dir() {
                fs::create_dir_all(&dest_path)
                    .with_context(|| format!("Failed to create directory: {}", dest_path.display()))?;
            } else if entry_type.is_file() {
                // Ensure parent directory exists
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
                }
                
                // Remove existing file if present (handles permission issues)
                if dest_path.exists() {
                    let _ = fs::remove_file(&dest_path);
                }
                
                // Extract the file
                let mut file = fs::File::create(&dest_path)
                    .with_context(|| format!("Failed to create file: {}", dest_path.display()))?;
                std::io::copy(&mut entry, &mut file)
                    .with_context(|| format!("Failed to write file: {}", dest_path.display()))?;
                
                // Set permissions on Unix
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = entry.header().mode().unwrap_or(0o644);
                    let perms = fs::Permissions::from_mode(mode);
                    fs::set_permissions(&dest_path, perms)
                        .with_context(|| format!("Failed to set permissions on: {}", dest_path.display()))?;
                }
            }
            // Skip symlinks and other entry types for now
        }

        Ok(())
    }

    #[cfg(not(feature = "bundled-zig"))]
    fn extract_zig(&self, _dest_dir: &Path) -> Result<()> {
        bail!("Zig extraction not available - bundled-zig feature not enabled")
    }
}

/// Get the tools cache directory.
///
/// This is where we extract bundled tools like Zig.
/// Respects RB_SYS_RUNTIME_CACHE_DIR environment variable.
fn get_tools_cache_dir() -> Result<PathBuf> {
    let cache_dir = if let Ok(override_dir) = std::env::var("RB_SYS_RUNTIME_CACHE_DIR") {
        PathBuf::from(override_dir).join("tools")
    } else if let Some(cache_home) = std::env::var_os("XDG_CACHE_HOME") {
        PathBuf::from(cache_home).join("rb-sys/cli/tools")
    } else if let Some(home_dir) = dirs::home_dir() {
        home_dir.join(".cache/rb-sys/cli/tools")
    } else {
        bail!("Could not determine cache directory (no HOME or XDG_CACHE_HOME)")
    };

    fs::create_dir_all(&cache_dir)
        .with_context(|| format!("Failed to create tools cache directory: {}", cache_dir.display()))?;

    Ok(cache_dir)
}

/// Resolve the Zig path to use for builds.
///
/// Priority order:
/// 1. Explicit path from --zig-path or ZIG_PATH env var (if provided and not default)
/// 2. Bundled Zig (if available)
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

    // Try bundled Zig
    if ZigManager::is_bundled() {
        let manager = ZigManager::new()?;
        match manager.ensure() {
            Ok(path) => return Ok(path),
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "Failed to use bundled Zig, falling back to system"
                );
            }
        }
    }

    // Fall back to system Zig
    which::which("zig")
        .context(
            "Zig not found. Please either:\n  \
             1. Install Zig (https://ziglang.org/download/)\n  \
             2. Use a cargo-gem build with bundled Zig\n  \
             3. Set ZIG_PATH to point to your Zig installation"
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_platform_is_valid() {
        // This test ensures the current build has a valid host platform
        assert!(
            HOST_PLATFORM == "x86_64-linux"
                || HOST_PLATFORM == "aarch64-linux"
                || HOST_PLATFORM == "x86_64-macos"
                || HOST_PLATFORM == "aarch64-macos"
                || HOST_PLATFORM == "x86_64-windows"
                || HOST_PLATFORM == "unsupported",
            "Unexpected HOST_PLATFORM: {HOST_PLATFORM}"
        );
    }

    #[test]
    fn test_resolve_zig_path_explicit() {
        let explicit = PathBuf::from("/custom/path/to/zig");
        let result = resolve_zig_path(Some(&explicit));
        // Should return the explicit path (even if it doesn't exist, for testing)
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), explicit);
    }
}
