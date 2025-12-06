use anyhow::{Context, Result};
use std::fs::{self, File};
use std::path::Path;
use walkdir::WalkDir;

use crate::config::Config;

pub fn build_assets(config_path: &Path, cache_dir: &Path, embedded_dir: &Path) -> Result<()> {
    let _config = Config::load(config_path)?;

    // Create embedded directory
    fs::create_dir_all(embedded_dir)
        .with_context(|| format!("Failed to create directory: {}", embedded_dir.display()))?;

    let archive_path = embedded_dir.join("assets.tar.zst");

    tracing::info!("Creating embedded asset archive");

    let file = File::create(&archive_path)
        .with_context(|| format!("Failed to create archive: {}", archive_path.display()))?;

    // Use zstd level 10 for good compression with reasonable speed
    // Note: zstd crate doesn't expose multithread API, but compression is still fast
    let encoder = zstd::Encoder::new(file, 10)
        .context("Failed to create zstd encoder")?
        .auto_finish();

    let mut tar = tar::Builder::new(encoder);

    // Add each platform directory from cache
    for entry in fs::read_dir(cache_dir)
        .with_context(|| format!("Failed to read cache directory: {}", cache_dir.display()))?
    {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        // Skip non-directories and special files
        if !path.is_dir() {
            continue;
        }

        let dir_name = path.file_name().unwrap().to_string_lossy();

        // Skip if it's not a platform directory
        if dir_name.starts_with('.') {
            continue;
        }

        tracing::debug!("Adding {} to archive", dir_name);

        // Add all files in the platform directory (deterministically)
        add_directory_to_tar(&mut tar, &path, &dir_name)?;
    }

    tar.into_inner()
        .context("Failed to finish tar archive")?;

    let metadata = fs::metadata(&archive_path)
        .context("Failed to get archive metadata")?;
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
