use anyhow::{Context, Result};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const ASSETS_DIR: &str = "data/staging/phase_1/assets";
const RUNTIME_MANIFEST: &str = "data/derived/runtime_manifest.json";
const OUTPUT_ARCHIVE: &str = "crates/rb-sys-cli/src/embedded/assets.tar.xz";
const OUTPUT_MANIFEST: &str = "crates/rb-sys-cli/src/embedded/runtime_manifest.json";

fn main() -> Result<()> {
    // Setup tracing
    tracing_subscriber::fmt().with_target(false).init();

    tracing::info!("Phase 2: Bundle assets");

    let assets_dir = PathBuf::from(ASSETS_DIR);
    let runtime_manifest = PathBuf::from(RUNTIME_MANIFEST);
    let output_archive = PathBuf::from(OUTPUT_ARCHIVE);
    let output_manifest = PathBuf::from(OUTPUT_MANIFEST);

    // Verify inputs exist
    if !assets_dir.exists() {
        anyhow::bail!("Assets directory not found: {}", assets_dir.display());
    }
    if !runtime_manifest.exists() {
        anyhow::bail!("Runtime manifest not found: {}", runtime_manifest.display());
    }

    // Create output directory
    fs::create_dir_all(output_archive.parent().unwrap())
        .context("Failed to create output directory")?;

    tracing::info!("Creating deterministic tar.xz archive");

    let file = File::create(&output_archive)
        .with_context(|| format!("Failed to create archive: {}", output_archive.display()))?;

    // Use xz compression level 6 (good balance of size and speed)
    let encoder = xz2::write::XzEncoder::new(file, 6);
    let mut tar = tar::Builder::new(encoder);

    // Add assets directory with deterministic properties
    add_directory_deterministic(&mut tar, &assets_dir, "assets")?;

    tar.into_inner()
        .context("Failed to finish tar archive")?
        .finish()
        .context("Failed to finish xz encoder")?;

    let metadata = fs::metadata(&output_archive).context("Failed to get archive metadata")?;
    let size_mb = metadata.len() as f64 / 1_048_576.0;

    tracing::info!("✓ Created archive: {:.1} MB", size_mb);

    // Copy runtime manifest to embedded dir
    fs::copy(&runtime_manifest, &output_manifest)
        .with_context(|| format!("Failed to copy manifest to {}", output_manifest.display()))?;

    tracing::info!("✓ Copied runtime manifest");
    tracing::info!("✓ Phase 2 complete");
    tracing::info!("  Archive: {}", output_archive.display());
    tracing::info!("  Manifest: {}", output_manifest.display());

    Ok(())
}

/// Add directory to tar with maximum determinism
/// - Fixed timestamps (mtime=0)
/// - Fixed uid/gid (0)
/// - Fixed modes (755 for dirs, 644 for files, 755 for executables)
/// - Sorted entries
fn add_directory_deterministic<W: std::io::Write>(
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
