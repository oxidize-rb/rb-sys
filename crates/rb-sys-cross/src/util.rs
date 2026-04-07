use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use flate2::write::GzEncoder;
use flate2::Compression;
use sha2::{Digest, Sha256};

/// Recursively copy a directory tree.
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Gzip-compress a byte slice.
pub fn gz_compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut enc = GzEncoder::new(Vec::new(), Compression::default());
    enc.write_all(data).context("gzip write")?;
    enc.finish().context("gzip finish")
}

/// Compute the hex-encoded SHA256 digest of a byte slice.
pub fn hex_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}
