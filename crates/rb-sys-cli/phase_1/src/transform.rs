use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Transform phase_0 staging artifacts into phase_1 normalized assets
pub fn transform_assets(
    staging_dir: &Path,
    output_dir: &Path,
    lockfile_path: &Path,
) -> Result<RuntimeManifest> {
    tracing::info!("Transforming phase_0 assets from {}", staging_dir.display());

    // Load phase_0 lockfile
    let lockfile_content = std::fs::read_to_string(lockfile_path)
        .with_context(|| format!("Failed to read lockfile: {}", lockfile_path.display()))?;
    let lockfile: Phase0Lockfile = toml::from_str(&lockfile_content)
        .with_context(|| format!("Failed to parse lockfile: {}", lockfile_path.display()))?;

    // Create output directory
    std::fs::create_dir_all(output_dir)?;
    let assets_dir = output_dir.join("assets");
    std::fs::create_dir_all(&assets_dir)?;

    let mut runtime_manifest = RuntimeManifest::new();

    // Process each platform
    for (platform, platform_lock) in &lockfile.platforms {
        if platform == "common" {
            continue; // Skip common for now, handle separately
        }

        tracing::info!("Processing platform: {}", platform);

        for (asset_name, asset_lock) in &platform_lock.assets {
            let staging_asset_dir = staging_dir.join(platform).join(asset_name);

            if !staging_asset_dir.exists() {
                tracing::warn!(
                    "Staging directory not found: {}",
                    staging_asset_dir.display()
                );
                continue;
            }

            match asset_lock {
                Phase0AssetLock::Tarball { .. } => {
                    // Unpack tarball and normalize
                    transform_tarball(&staging_asset_dir, &assets_dir, platform, asset_name)?;
                }
                Phase0AssetLock::OciExtract { files, .. } => {
                    // Already extracted, just copy and normalize
                    transform_extracted(&staging_asset_dir, &assets_dir, platform, asset_name)?;
                }
                Phase0AssetLock::TarballExtract { .. } => {
                    // Single file extracted from tarball, just copy and normalize
                    transform_extracted(&staging_asset_dir, &assets_dir, platform, asset_name)?;
                }
            }

            // Add to runtime manifest
            let asset_path = assets_dir.join(platform).join(asset_name);
            let asset_info = compute_asset_info(&asset_path)?;
            runtime_manifest.add_asset(platform, asset_name, asset_info);

            tracing::info!("  ✓ {}", asset_name);
        }
    }

    Ok(runtime_manifest)
}

fn transform_tarball(
    staging_dir: &Path,
    output_dir: &Path,
    platform: &str,
    asset_name: &str,
) -> Result<()> {
    // Find the tarball in staging dir
    let entries = std::fs::read_dir(staging_dir)?;
    let mut tarball_path = None;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            tarball_path = Some(path);
            break;
        }
    }

    let tarball_path = tarball_path
        .ok_or_else(|| anyhow::anyhow!("No tarball found in {}", staging_dir.display()))?;

    tracing::debug!("Unpacking tarball: {}", tarball_path.display());

    // Determine compression from extension
    let file = std::fs::File::open(&tarball_path)?;

    let extract_dir = output_dir.join(platform).join(asset_name);
    std::fs::create_dir_all(&extract_dir)?;

    // Special handling for SDK assets - filter during extraction
    let is_sdk = asset_name == "sdk";

    // Unpack based on file extension
    if tarball_path.extension().and_then(|s| s.to_str()) == Some("xz")
        || tarball_path.to_str().unwrap_or("").ends_with(".tar.xz")
    {
        use xz2::read::XzDecoder;
        let decoder = XzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);
        if is_sdk {
            unpack_sdk_archive(&mut archive, &extract_dir, 1)?;
        } else {
            unpack_archive(&mut archive, &extract_dir, 1)?;
        }
    } else if tarball_path.extension().and_then(|s| s.to_str()) == Some("gz")
        || tarball_path.to_str().unwrap_or("").ends_with(".tar.gz")
    {
        use flate2::read::GzDecoder;
        let decoder = GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);
        if is_sdk {
            unpack_sdk_archive(&mut archive, &extract_dir, 1)?;
        } else {
            unpack_archive(&mut archive, &extract_dir, 1)?;
        }
    } else if tarball_path.extension().and_then(|s| s.to_str()) == Some("zst")
        || tarball_path.to_str().unwrap_or("").ends_with(".tar.zst")
    {
        use zstd::stream::read::Decoder;
        let decoder = Decoder::new(file)?;
        let mut archive = tar::Archive::new(decoder);
        if is_sdk {
            unpack_sdk_archive(&mut archive, &extract_dir, 1)?;
        } else {
            unpack_archive(&mut archive, &extract_dir, 1)?;
        }
    } else if tarball_path.extension().and_then(|s| s.to_str()) == Some("zip") {
        // Handle ZIP files
        use zip::ZipArchive;
        let mut archive = ZipArchive::new(file)?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = extract_dir.join(file.name());

            if file.is_dir() {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    std::fs::create_dir_all(p)?;
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }
    } else {
        // Assume uncompressed tar
        let mut archive = tar::Archive::new(file);
        if is_sdk {
            unpack_sdk_archive(&mut archive, &extract_dir, 1)?;
        } else {
            unpack_archive(&mut archive, &extract_dir, 1)?;
        }
    }

    // Normalize permissions
    normalize_permissions(&extract_dir)?;

    Ok(())
}

fn transform_extracted(
    staging_dir: &Path,
    output_dir: &Path,
    platform: &str,
    asset_name: &str,
) -> Result<()> {
    let dest_dir = output_dir.join(platform).join(asset_name);
    std::fs::create_dir_all(&dest_dir)?;

    // Special handling for SDK assets - filter during copy
    if asset_name == "sdk" {
        copy_sdk_filtered(staging_dir, &dest_dir)?;
    } else {
        // Copy recursively
        copy_dir_all(staging_dir, &dest_dir)?;
    }

    // Normalize permissions
    normalize_permissions(&dest_dir)?;

    Ok(())
}

fn unpack_archive<R: std::io::Read>(
    archive: &mut tar::Archive<R>,
    dest: &Path,
    strip_components: usize,
) -> Result<()> {
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;

        // Strip components
        let components: Vec<_> = path.components().collect();
        if components.len() <= strip_components {
            continue;
        }

        let stripped: PathBuf = components[strip_components..].iter().collect();

        // Security check
        if stripped.is_absolute()
            || stripped
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            tracing::warn!("Skipping unsafe path: {}", stripped.display());
            continue;
        }

        let dest_path = dest.join(&stripped);

        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        entry.unpack(&dest_path)?;
    }

    Ok(())
}

/// Unpack SDK archive with filtering to reduce size
fn unpack_sdk_archive<R: std::io::Read>(
    archive: &mut tar::Archive<R>,
    dest: &Path,
    strip_components: usize,
) -> Result<()> {
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;

        // Strip components
        let components: Vec<_> = path.components().collect();
        if components.len() <= strip_components {
            continue;
        }

        let stripped: PathBuf = components[strip_components..].iter().collect();

        // Security check
        if stripped.is_absolute()
            || stripped
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            tracing::warn!("Skipping unsafe path: {}", stripped.display());
            continue;
        }

        // SDK filtering: only keep essential files
        if !should_keep_sdk_path(&stripped) {
            continue;
        }

        let dest_path = dest.join(&stripped);

        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        entry.unpack(&dest_path)?;
    }

    Ok(())
}

/// Determine if a path should be kept during SDK extraction
fn should_keep_sdk_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();

    // Always keep root-level SDK metadata
    if path.components().count() == 1 {
        return path_str.ends_with(".json") || path_str.ends_with(".plist");
    }

    // Keep all headers (essential for compilation)
    if path_str.starts_with("usr/include/") {
        return true;
    }

    // Keep all .tbd stub libraries (essential for linking)
    if path_str.starts_with("usr/lib/") && path_str.ends_with(".tbd") {
        return true;
    }

    // Keep Swift runtime stubs (needed for some frameworks)
    if path_str.starts_with("usr/lib/swift/") {
        return true;
    }

    // Framework filtering - only keep essential frameworks
    if path_str.starts_with("System/Library/Frameworks/") {
        let framework_name = extract_framework_name(&path_str);
        return is_essential_framework(&framework_name);
    }

    // Reject everything else (PrivateFrameworks, iOSSupport, docs, etc.)
    false
}

/// Extract framework name from path like "System/Library/Frameworks/Foo.framework/..."
fn extract_framework_name(path: &str) -> &str {
    if let Some(start) = path.find("System/Library/Frameworks/") {
        let after_prefix = &path[start + "System/Library/Frameworks/".len()..];
        if let Some(end) = after_prefix.find(".framework") {
            &after_prefix[..end]
        } else if let Some(end) = after_prefix.find('/') {
            &after_prefix[..end]
        } else {
            after_prefix
        }
    } else {
        ""
    }
}

/// Check if a framework is essential for Ruby gem cross-compilation
fn is_essential_framework(framework: &str) -> bool {
    matches!(
        framework,
        "Foundation"
            | "CoreFoundation"
            | "Security"
            | "IOKit"
            | "CoreServices"
            | "SystemConfiguration"
            | "CFNetwork"
            | "DiskArbitration"
            | "Accelerate"
            | "Metal"
            | "OpenCL"
            | "OpenGL"
            | "ApplicationServices"
            | "CoreGraphics"
            | "CoreText"
            | "ImageIO"
            | "QuartzCore"
    )
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    use walkdir::WalkDir;

    for entry in WalkDir::new(src) {
        let entry = entry?;
        let rel_path = entry.path().strip_prefix(src)?;
        let dest_path = dst.join(rel_path);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &dest_path)?;
        }
    }

    Ok(())
}

/// Copy SDK directory with filtering to reduce size
fn copy_sdk_filtered(src: &Path, dst: &Path) -> Result<()> {
    use walkdir::WalkDir;

    for entry in WalkDir::new(src) {
        let entry = entry?;
        let rel_path = entry.path().strip_prefix(src)?;

        // Filter SDK paths
        if !should_keep_sdk_path(rel_path) {
            continue;
        }

        let dest_path = dst.join(rel_path);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &dest_path)?;
        }
    }

    Ok(())
}

fn normalize_permissions(dir: &Path) -> Result<()> {
    use walkdir::WalkDir;

    for entry in WalkDir::new(dir) {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let mut perms = metadata.permissions();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = perms.mode();

            // Remove group/world write, keep user perms and exec bits
            let new_mode = mode & 0o755;
            perms.set_mode(new_mode);

            std::fs::set_permissions(entry.path(), perms)?;
        }
    }

    Ok(())
}

fn compute_asset_info(asset_dir: &Path) -> Result<RuntimeAssetInfo> {
    use walkdir::WalkDir;

    let mut file_count = 0u64;
    let mut total_size = 0u64;
    let mut files = Vec::new();

    for entry in WalkDir::new(asset_dir) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let rel_path = entry.path().strip_prefix(asset_dir)?;
        let metadata = entry.metadata()?;
        let size = metadata.len();

        // Compute BLAKE3
        let file = std::fs::File::open(entry.path())?;
        let mut hasher = blake3::Hasher::new();
        std::io::copy(&mut std::io::BufReader::new(file), &mut hasher)?;
        let blake3 = hasher.finalize().to_hex().to_string();

        files.push(RuntimeFileInfo {
            path: rel_path.to_string_lossy().to_string(),
            blake3,
            size_bytes: size,
        });

        file_count += 1;
        total_size += size;
    }

    Ok(RuntimeAssetInfo {
        file_count,
        total_size_bytes: total_size,
        files,
    })
}

// Phase 0 lockfile structures (minimal, just what we need to read)
#[derive(Debug, Deserialize)]
struct Phase0Lockfile {
    generated_at: String,
    #[serde(flatten)]
    platforms: HashMap<String, Phase0PlatformLock>,
}

#[derive(Debug, Deserialize)]
struct Phase0PlatformLock {
    #[serde(flatten)]
    assets: HashMap<String, Phase0AssetLock>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Phase0AssetLock {
    OciExtract {
        files: Vec<Phase0FileDigest>,
    },
    Tarball {},
    TarballExtract {
        url: String,
        digest: String,
        source_path: String,
        verified_at: String,
        size_bytes: u64,
    },
}

#[derive(Debug, Deserialize)]
struct Phase0FileDigest {
    path: String,
    blake3: String,
    size_bytes: u64,
}

// Runtime manifest structures
#[derive(Debug, Serialize)]
pub struct RuntimeManifest {
    pub generated_at: String,
    #[serde(flatten)]
    pub platforms: HashMap<String, RuntimePlatform>,
}

#[derive(Debug, Serialize)]
pub struct RuntimePlatform {
    #[serde(flatten)]
    pub assets: HashMap<String, RuntimeAssetInfo>,
}

#[derive(Debug, Serialize)]
pub struct RuntimeAssetInfo {
    pub file_count: u64,
    pub total_size_bytes: u64,
    pub files: Vec<RuntimeFileInfo>,
}

#[derive(Debug, Serialize)]
pub struct RuntimeFileInfo {
    pub path: String,
    pub blake3: String,
    pub size_bytes: u64,
}

impl RuntimeManifest {
    pub fn new() -> Self {
        Self {
            generated_at: chrono::Utc::now().to_rfc3339(),
            platforms: HashMap::new(),
        }
    }

    pub fn add_asset(&mut self, platform: &str, asset_name: &str, info: RuntimeAssetInfo) {
        self.platforms
            .entry(platform.to_string())
            .or_insert_with(|| RuntimePlatform {
                assets: HashMap::new(),
            })
            .assets
            .insert(asset_name.to_string(), info);
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
