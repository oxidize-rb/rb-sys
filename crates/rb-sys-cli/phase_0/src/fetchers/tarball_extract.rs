use crate::digest;
use anyhow::{Context, Result};
use globset::{Glob, GlobMatcher};
use std::io::Read;
use std::path::{Path, PathBuf};

pub struct TarballExtractor {
    url: String,
    pattern: String,
    digest: String,
}

impl TarballExtractor {
    pub fn new(url: &str, pattern: &str, digest: &str) -> Self {
        Self {
            url: url.to_string(),
            pattern: pattern.to_string(),
            digest: digest.to_string(),
        }
    }

    /// Extract a single file from tarball matching pattern, verify digest, and return path + size
    pub async fn fetch(&self, cache_dir: &Path) -> Result<(PathBuf, u64, String)> {
        // Parse digest
        let (_algorithm, expected_hex) = digest::parse_digest(&self.digest)?;

        // Compile glob pattern
        let glob = Glob::new(&self.pattern)
            .with_context(|| format!("Invalid glob pattern: {}", self.pattern))?;
        let matcher = glob.compile_matcher();

        // Generate cache key from pattern and digest
        let cache_key = format!(
            "{}-{}",
            &expected_hex[..16],
            self.pattern
                .replace("**", "")
                .replace('/', "_")
                .replace('*', "star")
        );

        let cache_path = cache_dir.join("extracted").join(&cache_key);

        // Check if already cached and valid
        if cache_path.exists() {
            tracing::debug!("Found cached extracted file: {}", cache_path.display());
            let file = std::fs::File::open(&cache_path)?;
            let size = file.metadata()?.len();

            match digest::verify_digest(&self.digest, file) {
                Ok(_) => {
                    tracing::info!("Using cached extracted file for pattern: {}", self.pattern);
                    // We don't know the source path from cache, use empty string
                    return Ok((cache_path, size, String::new()));
                }
                Err(e) => {
                    tracing::warn!("Cached file has invalid digest, re-extracting: {}", e);
                    std::fs::remove_file(&cache_path)?;
                }
            }
        }

        // Check if tarball is cached
        let tarball_cache_key = format!(
            "{}-{}",
            &expected_hex[..16],
            self.url
                .rsplit('/')
                .next()
                .unwrap_or("download")
                .replace(|c: char| !c.is_alphanumeric() && c != '.', "_")
        );
        let tarball_cache_path = cache_dir.join("tarballs").join(&tarball_cache_key);

        let bytes = if tarball_cache_path.exists() {
            tracing::info!("Using cached tarball: {}", tarball_cache_path.display());
            std::fs::read(&tarball_cache_path).with_context(|| {
                format!(
                    "Failed to read cached tarball: {}",
                    tarball_cache_path.display()
                )
            })?
        } else {
            // Download the compressed file
            tracing::info!("Downloading: {}", self.url);
            std::fs::create_dir_all(cache_path.parent().unwrap())?;
            std::fs::create_dir_all(tarball_cache_path.parent().unwrap())?;

            let client = reqwest::Client::builder()
                .user_agent("rb-sys-cli-phase-0")
                .build()?;

            let response = client
                .get(&self.url)
                .send()
                .await
                .with_context(|| format!("Failed to download: {}", self.url))?
                .error_for_status()
                .with_context(|| format!("HTTP error downloading: {}", self.url))?;

            let downloaded_bytes = response
                .bytes()
                .await
                .with_context(|| format!("Failed to read response body from: {}", self.url))?;

            tracing::info!("Downloaded {} bytes", downloaded_bytes.len());

            // Save to tarball cache for future use
            std::fs::write(&tarball_cache_path, &downloaded_bytes).with_context(|| {
                format!("Failed to cache tarball: {}", tarball_cache_path.display())
            })?;

            downloaded_bytes.to_vec()
        };

        tracing::info!("Decompressing {} bytes...", bytes.len());

        // Decompress XZ -> TAR and extract matching file
        let (extracted_path, source_path) =
            self.decompress_and_extract(&bytes, &matcher, &cache_path)?;

        // Verify digest
        tracing::debug!("Verifying digest of extracted file...");
        let file = std::fs::File::open(&extracted_path)?;
        digest::verify_digest(&self.digest, file).with_context(|| {
            format!(
                "Digest verification failed for extracted file from: {}",
                self.url
            )
        })?;

        let size = std::fs::metadata(&extracted_path)?.len();
        tracing::info!(
            "Extracted and verified: {} from {} ({} bytes)",
            source_path,
            self.url,
            size
        );

        Ok((extracted_path, size, source_path))
    }

    fn decompress_and_extract(
        &self,
        compressed_data: &[u8],
        matcher: &GlobMatcher,
        cache_path: &Path,
    ) -> Result<(PathBuf, String)> {
        // Decompress XZ
        let mut decompressor = xz2::read::XzDecoder::new(compressed_data);

        // Read decompressed data into memory (or we could use a temp file for very large archives)
        let mut tar_data = Vec::new();
        decompressor
            .read_to_end(&mut tar_data)
            .with_context(|| "Failed to decompress XZ archive")?;

        tracing::debug!("Decompressed to {} bytes", tar_data.len());

        // Now extract from tar
        self.extract_from_tar_data(&tar_data, matcher, cache_path)
    }

    fn extract_from_tar_data(
        &self,
        tar_data: &[u8],
        matcher: &GlobMatcher,
        cache_path: &Path,
    ) -> Result<(PathBuf, String)> {
        let mut archive = tar::Archive::new(tar_data);

        let temp_extract_path = cache_path.with_extension("tmp");
        let mut entry_count = 0;
        let mut checked_paths: Vec<String> = Vec::new();
        let mut symlink_target: Option<String> = None;

        for entry_result in archive.entries()? {
            let mut entry = entry_result.context("Failed to read tar entry")?;
            let path = entry.path()?.to_path_buf();
            let path_str = path.to_string_lossy().to_string();
            entry_count += 1;

            // Only log paths that might be related to libclang
            if path_str.contains("libclang")
                && !path_str.contains("libclang_rt")
                && !path_str.ends_with(".a")
            {
                tracing::debug!("Found libclang path: {}", path_str);
                checked_paths.push(path_str.clone());
            }

            if matcher.is_match(&path_str) {
                let entry_type = entry.header().entry_type();

                // Handle symlinks by storing the target and continuing to look for it
                if entry_type.is_symlink() || entry_type.is_hard_link() {
                    if let Ok(link_name) = entry.link_name() {
                        if let Some(target) = link_name {
                            let target_str = target.to_string_lossy().to_string();
                            tracing::debug!("Found symlink {} -> {}", path_str, target_str);

                            // Resolve the target path relative to the symlink's directory
                            if let Some(parent) = path.parent() {
                                let resolved_target = parent.join(&target_str);
                                symlink_target =
                                    Some(resolved_target.to_string_lossy().to_string());
                                tracing::info!(
                                    "Will look for symlink target: {}",
                                    symlink_target.as_ref().unwrap()
                                );
                            } else {
                                symlink_target = Some(target_str);
                            }
                            continue;
                        }
                    }
                    tracing::debug!(
                        "Skipping symlink/hardlink without valid target: {}",
                        path_str
                    );
                    continue;
                }

                // Check if it's a regular file (not directory)
                if !entry_type.is_file() {
                    tracing::debug!(
                        "Skipping non-file match: {} (type: {:?})",
                        path_str,
                        entry_type
                    );
                    continue;
                }

                tracing::info!("Found matching file: {}", path_str);

                // Extract to temp file
                let mut temp_file = std::fs::File::create(&temp_extract_path)?;
                std::io::copy(&mut entry, &mut temp_file)?;
                drop(temp_file);

                // Move to final location
                std::fs::rename(&temp_extract_path, cache_path)?;

                return Ok((cache_path.to_path_buf(), path_str));
            }

            // Check if this entry is the target of a symlink we found
            if let Some(ref target) = symlink_target {
                if path_str == *target || path_str.ends_with(target) {
                    let entry_type = entry.header().entry_type();
                    if entry_type.is_file() {
                        tracing::info!("Found symlink target file: {}", path_str);

                        // Extract to temp file
                        let mut temp_file = std::fs::File::create(&temp_extract_path)?;
                        std::io::copy(&mut entry, &mut temp_file)?;
                        drop(temp_file);

                        // Move to final location
                        std::fs::rename(&temp_extract_path, cache_path)?;

                        return Ok((cache_path.to_path_buf(), path_str));
                    }
                }
            }
        }

        tracing::warn!(
            "Searched {} entries, found {} libclang-related paths: {:?}",
            entry_count,
            checked_paths.len(),
            checked_paths
        );
        anyhow::bail!(
            "No file matching pattern '{}' found in archive: {} (searched {} entries, found libclang paths: {:?})",
            self.pattern,
            self.url,
            entry_count,
            checked_paths
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_pattern_compile() {
        let glob = Glob::new("**/libclang.so*").unwrap();
        let matcher = glob.compile_matcher();

        assert!(matcher.is_match("lib/libclang.so"));
        assert!(matcher.is_match("usr/lib/libclang.so.19"));
        assert!(matcher.is_match("clang+llvm-19.1.5/lib/libclang.so.19.1.5"));
        assert!(!matcher.is_match("lib/libclang.a"));
    }

    #[test]
    fn test_glob_pattern_with_wrapper_dir() {
        let glob = Glob::new("**/lib/libclang.so*").unwrap();
        let matcher = glob.compile_matcher();

        // Official builds (no wrapper)
        assert!(matcher.is_match("lib/libclang.so"));
        assert!(matcher.is_match("lib/libclang.so.19"));
        assert!(matcher.is_match("lib/libclang.so.19.1"));

        // Community builds (with wrapper dir)
        assert!(matcher.is_match("clang+llvm-19.1.5-aarch64-linux-gnu/lib/libclang.so"));
        assert!(matcher.is_match("clang+llvm-19.1.5-aarch64-linux-gnu/lib/libclang.so.19.1.5"));
    }

    #[test]
    fn test_symlink_detection_logic() {
        use tar::{Builder, Header};
        use tempfile::TempDir;

        // Create a tar archive with a symlink and its target
        let temp_dir = TempDir::new().unwrap();
        let tar_path = temp_dir.path().join("test.tar");
        let mut tar_file = std::fs::File::create(&tar_path).unwrap();
        let mut builder = Builder::new(&mut tar_file);

        // Add the actual file first (libclang.so.19.1)
        let file_content = b"fake libclang library content";
        let mut header = Header::new_gnu();
        header.set_size(file_content.len() as u64);
        header.set_mode(0o644);
        header.set_entry_type(tar::EntryType::Regular);
        header.set_cksum();
        builder
            .append_data(&mut header, "lib/libclang.so.19.1", &file_content[..])
            .unwrap();

        // Add a symlink pointing to the versioned file
        let mut link_header = Header::new_gnu();
        link_header.set_size(0);
        link_header.set_mode(0o777);
        link_header.set_entry_type(tar::EntryType::Symlink);
        link_header.set_link_name("libclang.so.19.1").unwrap();
        link_header.set_cksum();
        builder
            .append_data(&mut link_header, "lib/libclang.so", &[][..])
            .unwrap();

        builder.finish().unwrap();
        drop(builder);
        drop(tar_file);

        // Read the tar file and verify our logic can detect symlinks
        let tar_bytes = std::fs::read(&tar_path).unwrap();
        let mut archive = tar::Archive::new(&tar_bytes[..]);

        let mut found_symlink = false;
        let mut found_target = false;

        for entry in archive.entries().unwrap() {
            let entry = entry.unwrap();
            let entry_type = entry.header().entry_type();
            let path = entry.path().unwrap();
            let path_str = path.to_string_lossy();

            if path_str == "lib/libclang.so" {
                assert!(
                    entry_type.is_symlink(),
                    "libclang.so should be detected as symlink"
                );
                found_symlink = true;

                // Verify we can read the link target
                if let Ok(link_name) = entry.link_name() {
                    if let Some(target) = link_name {
                        assert_eq!(target.to_string_lossy(), "libclang.so.19.1");
                    }
                }
            }

            if path_str == "lib/libclang.so.19.1" {
                assert!(
                    entry_type.is_file(),
                    "libclang.so.19.1 should be detected as regular file"
                );
                found_target = true;
            }
        }

        assert!(found_symlink, "Should find symlink in archive");
        assert!(found_target, "Should find target file in archive");
    }
}
