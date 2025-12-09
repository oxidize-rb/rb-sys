use anyhow::{Context, Result};
use std::fs::{self, File};
use std::path::Path;
use walkdir::WalkDir;

use crate::config::Config;

/// Directories to include in the embedded assets tarball (legacy mode).
///
/// We only include:
/// - `bindings/` - Pre-generated Ruby bindings (the main output of phase_1)
///
/// We explicitly exclude:
/// - Platform directories (rubies/, sysroot/) - Too large, only headers needed
/// - `zig/` - Only needed during phase_1 for binding generation, not at runtime
/// - `downloads/` - Temporary download cache
const INCLUDE_DIRS: &[&str] = &["bindings"];

/// Build embedded assets archive from phase_1 output
///
/// Supports two modes:
/// 1. New mode: phase_1_dir contains normalized assets from transform step
/// 2. Legacy mode: cache_dir contains legacy OCI extraction output
pub fn build_assets(config_path: &Path, cache_dir: &Path, embedded_dir: &Path) -> Result<()> {
    let _config = Config::load(config_path)?;

    // Create embedded directory
    fs::create_dir_all(embedded_dir)
        .with_context(|| format!("Failed to create directory: {}", embedded_dir.display()))?;

    // Check if we're in new mode (phase_1 output exists)
    let phase_1_assets = Path::new("data/staging/phase_1/assets");
    let use_new_mode = phase_1_assets.exists();

    if use_new_mode {
        build_assets_new_mode(phase_1_assets, embedded_dir)?;
    } else {
        build_assets_legacy_mode(cache_dir, embedded_dir)?;
    }

    Ok(())
}

fn build_assets_new_mode(phase_1_assets: &Path, embedded_dir: &Path) -> Result<()> {
    let archive_path = embedded_dir.join("assets.tar.xz");

    tracing::info!("Creating embedded asset archive (new mode, tar.xz)");

    let file = File::create(&archive_path)
        .with_context(|| format!("Failed to create archive: {}", archive_path.display()))?;

    // Use xz compression level 6 (good balance of size and speed)
    let encoder = xz2::write::XzEncoder::new(file, 6);
    let mut tar = tar::Builder::new(encoder);

    // Add entire phase_1/assets directory
    add_directory_to_tar_deterministic(&mut tar, phase_1_assets, "assets")?;

    tar.into_inner()
        .context("Failed to finish tar archive")?
        .finish()
        .context("Failed to finish xz encoder")?;

    let metadata = fs::metadata(&archive_path).context("Failed to get archive metadata")?;
    let size_mb = metadata.len() as f64 / 1_048_576.0;

    tracing::info!("Created embedded asset archive: {:.1} MB", size_mb);

    // Copy runtime_manifest.json to embedded dir
    let manifest_src = Path::new("data/derived/runtime_manifest.json");
    let manifest_dest = embedded_dir.join("runtime_manifest.json");

    if manifest_src.exists() {
        fs::copy(manifest_src, &manifest_dest)
            .with_context(|| format!("Failed to copy manifest to {}", manifest_dest.display()))?;
        tracing::info!("Copied runtime_manifest.json");
    }

    Ok(())
}

fn build_assets_legacy_mode(cache_dir: &Path, embedded_dir: &Path) -> Result<()> {
    let archive_path = embedded_dir.join("assets.tar.zst");

    tracing::info!("Creating embedded asset archive (legacy mode, tar.zst)");

    let file = File::create(&archive_path)
        .with_context(|| format!("Failed to create archive: {}", archive_path.display()))?;

    // Use zstd level 10 for good compression with reasonable speed
    let encoder = zstd::Encoder::new(file, 10)
        .context("Failed to create zstd encoder")?
        .auto_finish();

    let mut tar = tar::Builder::new(encoder);

    // Only add specific directories we want to embed
    for dir_name in INCLUDE_DIRS {
        let path = cache_dir.join(dir_name);

        if !path.exists() {
            tracing::warn!("Directory {} not found, skipping", dir_name);
            continue;
        }

        tracing::debug!("Adding {} to archive", dir_name);
        add_directory_to_tar(&mut tar, &path, dir_name)?;
    }

    tar.into_inner().context("Failed to finish tar archive")?;

    let metadata = fs::metadata(&archive_path).context("Failed to get archive metadata")?;
    let size_mb = metadata.len() as f64 / 1_048_576.0;

    tracing::info!("Created embedded asset archive: {:.1} MB", size_mb);

    // Also copy the manifest to embedded dir
    let manifest_src = cache_dir.join("manifest.json");
    let manifest_dest = embedded_dir.join("manifest.json");

    if manifest_src.exists() {
        fs::copy(&manifest_src, &manifest_dest)
            .with_context(|| format!("Failed to copy manifest to {}", manifest_dest.display()))?;
    }

    Ok(())
}

/// Add directory to tar with maximum determinism
/// - Fixed timestamps (mtime=0)
/// - Fixed uid/gid (0)
/// - Fixed modes (755 for dirs, 644 for files, 755 for executables)
/// - Sorted entries
fn add_directory_to_tar_deterministic<W: std::io::Write>(
    tar: &mut tar::Builder<W>,
    dir_path: &Path,
    base_name: &str,
) -> Result<()> {
    // Collect all entries and sort them for determinism
    let mut entries: Vec<_> = WalkDir::new(dir_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();

    // Sort by path for deterministic ordering
    entries.sort_by(|a, b| a.path().cmp(b.path()));

    for entry in entries {
        let path = entry.path();
        let relative_path = path
            .strip_prefix(dir_path)
            .context("Failed to strip prefix")?;

        // Skip the root directory itself
        if relative_path.as_os_str().is_empty() {
            continue;
        }

        let archive_path = Path::new(base_name).join(relative_path);

        if path.is_dir() {
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(tar::EntryType::Directory);
            header.set_size(0);
            header.set_mode(0o755);
            header.set_mtime(0);
            header.set_uid(0);
            header.set_gid(0);
            header.set_cksum();

            tar.append_data(&mut header, &archive_path, &mut std::io::empty())
                .with_context(|| format!("Failed to add directory: {}", path.display()))?;
        } else if path.is_file() {
            let mut file = File::open(path)
                .with_context(|| format!("Failed to open file: {}", path.display()))?;

            let metadata = fs::metadata(path)?;
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(tar::EntryType::Regular);
            header.set_size(metadata.len());

            // Determine if file should be executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = metadata.permissions().mode();
                // If owner has execute, make it 755, else 644
                if mode & 0o100 != 0 {
                    header.set_mode(0o755);
                } else {
                    header.set_mode(0o644);
                }
            }
            #[cfg(not(unix))]
            {
                header.set_mode(0o644);
            }

            header.set_mtime(0);
            header.set_uid(0);
            header.set_gid(0);
            header.set_cksum();

            tar.append_data(&mut header, &archive_path, &mut file)
                .with_context(|| format!("Failed to add file: {}", path.display()))?;
        }
    }

    Ok(())
}

fn add_directory_to_tar<W: std::io::Write>(
    tar: &mut tar::Builder<W>,
    dir_path: &Path,
    base_name: &str,
) -> Result<()> {
    // Collect all entries and sort them for determinism
    let mut entries: Vec<_> = WalkDir::new(dir_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();

    // Sort by path for deterministic ordering
    entries.sort_by(|a, b| a.path().cmp(b.path()));

    for entry in entries {
        let path = entry.path();
        let relative_path = path
            .strip_prefix(dir_path)
            .context("Failed to strip prefix")?;

        // Skip the root directory itself
        if relative_path.as_os_str().is_empty() {
            continue;
        }

        let archive_path = Path::new(base_name).join(relative_path);

        if path.is_dir() {
            tar.append_dir(&archive_path, path)
                .with_context(|| format!("Failed to add directory: {}", path.display()))?;
        } else if path.is_file() {
            let mut file = File::open(path)
                .with_context(|| format!("Failed to open file: {}", path.display()))?;

            // Create header with normalized metadata for determinism
            let metadata = fs::metadata(path)?;
            let mut header = tar::Header::new_gnu();
            header.set_size(metadata.len());
            header.set_mode(0o644); // Normalized mode
            header.set_mtime(0); // Normalized timestamp
            header.set_uid(0);
            header.set_gid(0);
            header.set_cksum();

            tar.append_data(&mut header, &archive_path, &mut file)
                .with_context(|| format!("Failed to add file: {}", path.display()))?;
        }
    }

    Ok(())
}
