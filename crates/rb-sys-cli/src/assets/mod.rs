pub mod manifest;

use anyhow::{bail, Context, Result};
use manifest::{Manifest, ToolInfo};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Embedded compressed asset archive (only when embed-assets feature is enabled)
#[cfg(feature = "embed-assets")]
static EMBEDDED_ASSETS: &[u8] = include_bytes!("../embedded/assets.tar.zst");

/// Embedded Ruby sysroots archive (only when embed-assets feature is enabled)
#[cfg(feature = "embed-assets")]
static EMBEDDED_RUBIES: &[u8] = include_bytes!("../embedded/rubies.tar.zst");

/// Embedded manifest (for quick lookups without decompressing)
#[cfg(feature = "embed-assets")]
static EMBEDDED_MANIFEST: &str = include_str!("../embedded/manifest.json");

/// Get embedded assets data (from include_bytes or external file)
fn get_embedded_assets() -> Result<&'static [u8]> {
    #[cfg(feature = "embed-assets")]
    {
        Ok(EMBEDDED_ASSETS)
    }

    #[cfg(not(feature = "embed-assets"))]
    {
        bail!(
            "Assets not embedded in binary. Set RB_SYS_ASSETS_PATH environment variable \
             or rebuild with --features embed-assets"
        )
    }
}

/// Get embedded manifest (from include_str or external file)
fn get_embedded_manifest() -> Result<&'static str> {
    #[cfg(feature = "embed-assets")]
    {
        Ok(EMBEDDED_MANIFEST)
    }

    #[cfg(not(feature = "embed-assets"))]
    {
        bail!(
            "Manifest not embedded in binary. Set RB_SYS_MANIFEST_PATH environment variable \
             or rebuild with --features embed-assets"
        )
    }
}

/// Get embedded rubies data (from include_bytes or external file)
#[allow(dead_code)]
fn get_embedded_rubies() -> Result<&'static [u8]> {
    #[cfg(feature = "embed-assets")]
    {
        Ok(EMBEDDED_RUBIES)
    }

    #[cfg(not(feature = "embed-assets"))]
    {
        bail!(
            "Rubies not embedded in binary. Set RB_SYS_RUBIES_PATH environment variable \
             or rebuild with --features embed-assets"
        )
    }
}

/// Manages embedded assets and lazy extraction
pub struct AssetManager {
    cache_dir: PathBuf,
    manifest: Manifest,
}

impl AssetManager {
    /// Create a new AssetManager
    pub fn new() -> Result<Self> {
        let cache_dir = get_runtime_cache_dir()?;
        let manifest_str = get_embedded_manifest()?;
        let manifest: Manifest =
            serde_json::from_str(manifest_str).context("Failed to parse embedded manifest")?;

        Ok(Self {
            cache_dir,
            manifest,
        })
    }

    /// Get the manifest
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Get tool entries from the embedded manifest
    pub fn tools(&self) -> &[ToolInfo] {
        &self.manifest.tools
    }

    /// Find the latest Ruby version available for a platform
    #[allow(dead_code)]
    pub fn latest_ruby_version(&self, ruby_platform: &str) -> Result<String> {
        // For now, return a hardcoded version to avoid tarball reading
        // TODO: Use manifest-based version detection
        match ruby_platform {
            "aarch64-linux" | "x86_64-linux" | "x86_64-linux-musl" | "aarch64-linux-musl"
            | "arm-linux" => Ok("3.3.9".to_string()),
            "x86_64-darwin" | "arm64-darwin" => Ok("3.3.9".to_string()),
            "x64-mingw-ucrt" | "x64-mingw32" | "aarch64-mingw-ucrt" => Ok("3.3.9".to_string()),
            _ => {
                // Fallback to manifest lookup
                for platform in self.manifest.platforms.values() {
                    if platform.ruby_platform == ruby_platform {
                        if !platform.ruby_versions.is_empty() {
                            return Ok(platform.ruby_versions[0].clone());
                        }
                    }
                }
                bail!("No Ruby versions found for platform: {}", ruby_platform);
            }
        }
    }

    /// Extract rbconfig.json for a specific platform/version from embedded assets
    /// If not found, generate a minimal rbconfig.json from the platform/version
    pub fn extract_rbconfig(&self, ruby_platform: &str, ruby_version: &str) -> Result<String> {
        let assets_data = get_embedded_assets()?;
        let decoder = match zstd::Decoder::new(assets_data) {
            Ok(decoder) => decoder,
            Err(e) => {
                debug!(
                    "Failed to create zstd decoder for rbconfig extraction: {}",
                    e
                );
                return generate_minimal_rbconfig(ruby_platform, ruby_version);
            }
        };
        let mut archive = tar::Archive::new(decoder);

        let rbconfig_path = format!("assets/bindings/{ruby_platform}/{ruby_version}/rbconfig.json");

        // First try to find existing rbconfig.json
        let entries = match archive.entries() {
            Ok(entries) => entries,
            Err(e) => {
                debug!("Failed to read tar entries for rbconfig: {}", e);
                return generate_minimal_rbconfig(ruby_platform, ruby_version);
            }
        };

        for entry_result in entries {
            let mut entry = match entry_result {
                Ok(entry) => entry,
                Err(e) => {
                    debug!("Failed to read tar entry for rbconfig: {}", e);
                    continue;
                }
            };
            let path = match entry.path() {
                Ok(path) => path,
                Err(e) => {
                    debug!("Failed to get entry path for rbconfig: {}", e);
                    continue;
                }
            };
            let path_str = path.to_string_lossy();

            if path_str == rbconfig_path {
                let mut content = String::new();
                if let Err(e) = entry.read_to_string(&mut content) {
                    debug!("Failed to read rbconfig.json content: {}", e);
                    return generate_minimal_rbconfig(ruby_platform, ruby_version);
                }
                return Ok(content);
            }
        }

        // If not found, generate minimal rbconfig.json from platform/version
        tracing::debug!(
            platform = %ruby_platform,
            version = %ruby_version,
            "rbconfig.json not found, generating minimal version"
        );

        generate_minimal_rbconfig(ruby_platform, ruby_version)
    }

    /// Extract sysroot files directly from embedded tarball to destination
    /// Extract Ruby binaries/libraries from embedded tarball to destination
    /// This extracts the entire rubies directory for the platform to the cache
    /// Extract macOS SDK from embedded tarball to destination
    /// This extracts the macOS SDK directory for the platform to the cache
    pub fn extract_macos_sdk(&self, platform: &str, dest_dir: &Path) -> Result<Option<PathBuf>> {
        // Check if already extracted (marker file)
        let marker = dest_dir.join(".macos-sdk-extracted");
        if marker.exists() {
            debug!(
                platform = %platform,
                dest = %dest_dir.display(),
                "macOS SDK already extracted"
            );
            // Return the path to the latest SDK
            return Ok(Some(self.find_latest_macos_sdk(dest_dir)?));
        }

        // First try to extract from tarball
        let tarball_result = self.extract_macos_sdk_from_tarball(platform, dest_dir);

        match tarball_result {
            Ok(result) => return Ok(result),
            Err(e) => {
                debug!(
                    platform = %platform,
                    error = %e,
                    "Failed to extract from tarball, trying loose files"
                );
                // Fall back to copying from embedded directory
                self.extract_macos_sdk_from_embedded_dir(platform, dest_dir)
            }
        }
    }

    /// Extract macOS SDK from embedded tarball
    fn extract_macos_sdk_from_tarball(
        &self,
        platform: &str,
        dest_dir: &Path,
    ) -> Result<Option<PathBuf>> {
        let assets_data = match get_embedded_assets() {
            Ok(data) => data,
            Err(e) => {
                debug!(
                    "Failed to get embedded assets for macOS SDK extraction: {}",
                    e
                );
                return Ok(None);
            }
        };
        let decoder = match zstd::Decoder::new(assets_data) {
            Ok(decoder) => decoder,
            Err(e) => {
                debug!(
                    "Failed to create zstd decoder for macOS SDK extraction: {}",
                    e
                );
                return Ok(None);
            }
        };
        let mut archive = tar::Archive::new(decoder);

        let sdk_prefix = format!("assets/{platform}/macos_sdk/");
        debug!(
            platform = %platform,
            prefix = %sdk_prefix,
            dest = %dest_dir.display(),
            "Extracting macOS SDK from tarball"
        );

        let mut extracted_count = 0;
        let entries = match archive.entries() {
            Ok(entries) => entries,
            Err(e) => {
                debug!("Failed to read tar entries for macOS SDK: {}", e);
                return Ok(None);
            }
        };

        for entry_result in entries {
            let mut entry = match entry_result {
                Ok(entry) => entry,
                Err(e) => {
                    debug!("Failed to read tar entry for macOS SDK: {}", e);
                    continue;
                }
            };
            let path = match entry.path() {
                Ok(path) => path,
                Err(e) => {
                    debug!("Failed to get entry path for macOS SDK: {}", e);
                    continue;
                }
            };
            let path_str = path.to_string_lossy();

            // Only extract SDK files for this platform
            if let Some(relative) = path_str.strip_prefix(&sdk_prefix) {
                let dest_path = dest_dir.join(relative);
                extracted_count += 1;

                if entry.header().entry_type().is_dir() {
                    fs::create_dir_all(&dest_path).with_context(|| {
                        format!("Failed to create directory: {}", dest_path.display())
                    })?;
                } else {
                    if let Some(parent) = dest_path.parent() {
                        fs::create_dir_all(parent).with_context(|| {
                            format!("Failed to create parent directory: {}", parent.display())
                        })?;
                    }
                    let mut file = fs::File::create(&dest_path).with_context(|| {
                        format!("Failed to create file: {}", dest_path.display())
                    })?;
                    std::io::copy(&mut entry, &mut file).with_context(|| {
                        format!("Failed to write file: {}", dest_path.display())
                    })?;

                    // Preserve permissions on Unix
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(mode) = entry.header().mode() {
                            let perms = fs::Permissions::from_mode(mode);
                            let _ = fs::set_permissions(&dest_path, perms);
                        }
                    }
                }
            }
        }

        if extracted_count == 0 {
            debug!(
                platform = %platform,
                "No macOS SDK found in tarball"
            );
            return Ok(None);
        }

        // Write marker file
        let marker = dest_dir.join(".macos-sdk-extracted");
        fs::write(&marker, "")
            .with_context(|| format!("Failed to write marker file: {}", marker.display()))?;

        info!(
            platform = %platform,
            count = extracted_count,
            dest = %dest_dir.display(),
            "Extracted macOS SDK from tarball"
        );

        // Return the path to the latest SDK
        Ok(Some(self.find_latest_macos_sdk(dest_dir)?))
    }

    /// Extract macOS SDK by copying from embedded directory (fallback)
    fn extract_macos_sdk_from_embedded_dir(
        &self,
        platform: &str,
        dest_dir: &Path,
    ) -> Result<Option<PathBuf>> {
        // Map Ruby platform name to Rust target name for embedded directory
        let rust_target = match platform {
            "x86_64-darwin" => "x86_64-apple-darwin",
            "aarch64-darwin" => "aarch64-apple-darwin",
            _ => platform,
        };

        // Get the embedded directory path
        let embedded_dir = self.embedded_dir();
        let platform_dir = embedded_dir.join(rust_target);
        let sdk_dir = platform_dir.join("sdk");

        if !sdk_dir.exists() {
            debug!(
                platform = %platform,
                rust_target = %rust_target,
                sdk_dir = %sdk_dir.display(),
                "No macOS SDK found in embedded directory"
            );
            return Ok(None);
        }

        debug!(
            platform = %platform,
            src = %sdk_dir.display(),
            dest = %dest_dir.display(),
            "Copying macOS SDK from embedded directory"
        );

        // Copy the SDK directory
        self.copy_dir_recursively(&sdk_dir, dest_dir)?;

        // Write marker file
        let marker = dest_dir.join(".macos-sdk-extracted");
        fs::write(&marker, "")
            .with_context(|| format!("Failed to write marker file: {}", marker.display()))?;

        info!(
            platform = %platform,
            src = %sdk_dir.display(),
            dest = %dest_dir.display(),
            "Copied macOS SDK from embedded directory"
        );

        // Return the path to the latest SDK
        Ok(Some(self.find_latest_macos_sdk(dest_dir)?))
    }

    /// Get the embedded directory path
    fn embedded_dir(&self) -> PathBuf {
        // The embedded directory is at the crate root
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/embedded")
    }

    /// Recursively copy a directory
    fn copy_dir_recursively(&self, src: &Path, dst: &Path) -> Result<()> {
        if !dst.exists() {
            fs::create_dir_all(dst)?;
        }

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                self.copy_dir_recursively(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }

        Ok(())
    }

    /// Find the latest macOS SDK in the given directory
    fn find_latest_macos_sdk(&self, sdk_dir: &Path) -> Result<PathBuf> {
        if !sdk_dir.exists() {
            bail!("macOS SDK directory does not exist: {}", sdk_dir.display());
        }

        // Check if the directory itself contains SDK files (direct extraction)
        let usr_include = sdk_dir.join("usr/include");
        let frameworks = sdk_dir.join("System/Library/Frameworks");
        if usr_include.exists() && frameworks.exists() {
            debug!(
                sdk = %sdk_dir.display(),
                "Found macOS SDK (direct extraction)"
            );
            return Ok(sdk_dir.to_path_buf());
        }

        // Otherwise, look for SDK subdirectories
        let mut sdk_versions = Vec::new();

        for entry in fs::read_dir(sdk_dir).context("Failed to read SDK directory")? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                // Match SDK names like "MacOSX14.0.sdk", "MacOSX13.3.sdk"
                if name.starts_with("MacOSX") && name.ends_with(".sdk") {
                    sdk_versions.push(path);
                }
            }
        }

        if sdk_versions.is_empty() {
            bail!("No macOS SDK found in {}", sdk_dir.display());
        }

        // Sort by version (latest first)
        sdk_versions.sort_by(|a, b| {
            let a_name = a.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let b_name = b.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let a_version = a_name
                .strip_prefix("MacOSX")
                .and_then(|s| s.strip_suffix(".sdk"))
                .unwrap_or("");
            let b_version = b_name
                .strip_prefix("MacOSX")
                .and_then(|s| s.strip_suffix(".sdk"))
                .unwrap_or("");
            b_version.cmp(a_version)
        });

        let latest_sdk = sdk_versions[0].clone();
        debug!(
            sdk = %latest_sdk.display(),
            "Selected latest macOS SDK"
        );

        Ok(latest_sdk)
    }

    /// Extract a tool archive from embedded assets with BLAKE3 verification
    pub fn extract_tool(&self, tool: &ToolInfo, dest_dir: &Path) -> Result<PathBuf> {
        let tool_cache_dir = dest_dir
            .join(&tool.host_platform)
            .join(&tool.name)
            .join(&tool.version);
        let marker_file = tool_cache_dir.join(".extracted");

        // Check if already extracted
        if marker_file.exists() {
            debug!(
                tool = %tool.name,
                version = %tool.version,
                path = %tool_cache_dir.display(),
                "Tool already extracted"
            );
            return Ok(tool_cache_dir);
        }

        info!(
            tool = %tool.name,
            version = %tool.version,
            "Extracting tool from embedded assets"
        );

        // Create temp directory for extraction
        let temp_dir = tempfile::tempdir_in(dest_dir)
            .context("Failed to create temp directory for tool extraction")?;

        // Extract tool directory from embedded assets
        let tool_temp_dir = self.extract_tool_directory(tool, temp_dir.path())?;

        // Copy the extracted tool to final destination
        self.copy_dir_recursively(&tool_temp_dir, &tool_cache_dir)?;

        // Write marker file
        fs::write(
            &marker_file,
            format!(
                "Extracted {} {} at {}",
                tool.name,
                tool.version,
                chrono::Utc::now()
            ),
        )
        .with_context(|| format!("Failed to write marker file: {}", marker_file.display()))?;

        info!(
            tool = %tool.name,
            path = %tool_cache_dir.display(),
            "Tool extracted successfully"
        );

        Ok(tool_cache_dir)
    }

    /// Extract Ruby sysroot for a specific platform and version
    /// Extract Ruby sysroot from the embedded rubies archive
    /// Extract a tool directory from the embedded tarball
    /// Note: Currently tools are stored as extracted directories, not nested archives
    fn extract_tool_directory(&self, tool: &ToolInfo, temp_dir: &Path) -> Result<PathBuf> {
        let assets_data = get_embedded_assets()?;
        let decoder = zstd::Decoder::new(assets_data).context("Failed to create zstd decoder")?;
        let mut archive = tar::Archive::new(decoder);

        let tool_prefix = format!("assets/{}/{}", tool.host_platform, tool.name);
        let temp_tool_dir = temp_dir.join(&tool.name);

        fs::create_dir_all(&temp_tool_dir).with_context(|| {
            format!(
                "Failed to create tool directory: {}",
                temp_tool_dir.display()
            )
        })?;

        let mut extracted_count = 0;
        for entry in archive.entries().context("Failed to read tar entries")? {
            let mut entry = entry.context("Failed to read tar entry")?;
            let path = entry.path().context("Failed to get entry path")?;
            let path_str = path.to_string_lossy();

            // Extract files that belong to this tool
            if let Some(relative) = path_str.strip_prefix(&tool_prefix) {
                if relative.is_empty() {
                    continue; // Skip the directory itself
                }

                let dest_path = temp_tool_dir.join(relative);

                if entry.header().entry_type().is_dir() {
                    fs::create_dir_all(&dest_path).with_context(|| {
                        format!("Failed to create directory: {}", dest_path.display())
                    })?;
                } else {
                    if let Some(parent) = dest_path.parent() {
                        fs::create_dir_all(parent).with_context(|| {
                            format!("Failed to create parent directory: {}", parent.display())
                        })?;
                    }
                    let mut file = fs::File::create(&dest_path).with_context(|| {
                        format!("Failed to create file: {}", dest_path.display())
                    })?;
                    std::io::copy(&mut entry, &mut file).with_context(|| {
                        format!("Failed to write file: {}", dest_path.display())
                    })?;

                    // Preserve permissions on Unix
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(mode) = entry.header().mode() {
                            let perms = fs::Permissions::from_mode(mode);
                            let _ = fs::set_permissions(&dest_path, perms);
                        }
                    }
                }
                extracted_count += 1;
            }
        }

        if extracted_count == 0 {
            bail!("No files found for tool {} in embedded assets", tool.name);
        }

        debug!(
            tool = %tool.name,
            count = extracted_count,
            "Extracted tool directory"
        );

        Ok(temp_tool_dir)
    }
}

/// Generate a minimal rbconfig.json for cross-compilation
fn generate_minimal_rbconfig(ruby_platform: &str, ruby_version: &str) -> Result<String> {
    // Parse version components
    let version_parts: Vec<&str> = ruby_version.split('.').collect();
    let major = version_parts.get(0).unwrap_or(&"3");
    let minor = version_parts.get(1).unwrap_or(&"0");
    let teeny = version_parts.get(2).unwrap_or(&"0");

    // Create minimal rbconfig.json
    let rbconfig = serde_json::json!({
        "config": {
            "platform": ruby_platform,
            "RUBY_PROGRAM_VERSION": ruby_version,
            "MAJOR": major,
            "MINOR": minor,
            "TEENY": teeny,
            "PATCHLEVEL": "0",
            "ruby_version": format!("{}.{}", major, minor),
            "arch": ruby_platform,
            "host": ruby_platform,
            "target": ruby_platform,
            "CROSS_COMPILING": "yes"
        }
    });

    serde_json::to_string(&rbconfig).context("Failed to serialize minimal rbconfig.json")
}

/// Get the runtime cache directory
fn get_runtime_cache_dir() -> Result<PathBuf> {
    let cache_dir = if let Ok(override_dir) = std::env::var("RB_SYS_RUNTIME_CACHE_DIR") {
        PathBuf::from(override_dir)
    } else if let Some(cache_home) = std::env::var_os("XDG_CACHE_HOME") {
        PathBuf::from(cache_home).join("rb-sys/cli")
    } else if let Some(home_dir) = dirs::home_dir() {
        home_dir.join(".cache/rb-sys/cli")
    } else {
        anyhow::bail!("Could not determine cache directory (no HOME or XDG_CACHE_HOME)")
    };

    fs::create_dir_all(&cache_dir)
        .with_context(|| format!("Failed to create cache directory: {}", cache_dir.display()))?;

    Ok(cache_dir)
}

/// Clear the runtime cache
pub fn clear_cache() -> Result<()> {
    let cache_dir = get_runtime_cache_dir()?;
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir).with_context(|| {
            format!("Failed to remove cache directory: {}", cache_dir.display())
        })?;
        println!("✅ Cleared runtime cache: {}", cache_dir.display());
    } else {
        println!(
            "ℹ️  Cache directory does not exist: {}",
            cache_dir.display()
        );
    }
    Ok(())
}

/// Get the cache directory path
pub fn get_cache_dir() -> Result<PathBuf> {
    get_runtime_cache_dir()
}
