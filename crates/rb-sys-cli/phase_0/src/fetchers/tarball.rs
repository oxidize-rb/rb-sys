use crate::digest;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

pub struct TarballFetcher {
    url: String,
    digest: String,
}

impl TarballFetcher {
    pub fn new(url: &str, digest: &str) -> Self {
        Self {
            url: url.to_string(),
            digest: digest.to_string(),
        }
    }

    /// Fetch tarball to cache, verify digest, and return path + size
    pub async fn fetch(&self, cache_dir: &Path) -> Result<(PathBuf, u64)> {
        // Parse digest
        let (_algorithm, expected_hex) = digest::parse_digest(&self.digest)?;

        // Generate cache key from URL and digest
        let cache_key = format!(
            "{}-{}",
            &expected_hex[..16],
            self.url
                .rsplit('/')
                .next()
                .unwrap_or("download")
                .replace(|c: char| !c.is_alphanumeric() && c != '.', "_")
        );

        let cache_path = cache_dir.join("tarballs").join(&cache_key);

        // Check if already cached and valid
        if cache_path.exists() {
            tracing::debug!("Found cached tarball: {}", cache_path.display());
            let file = std::fs::File::open(&cache_path)?;
            let size = file.metadata()?.len();

            match digest::verify_digest(&self.digest, file) {
                Ok(_) => {
                    tracing::info!("Using cached tarball: {}", self.url);
                    return Ok((cache_path, size));
                }
                Err(e) => {
                    tracing::warn!("Cached tarball has invalid digest, re-downloading: {}", e);
                    std::fs::remove_file(&cache_path)?;
                }
            }
        }

        // Download with streaming verification
        tracing::info!("Downloading: {}", self.url);
        std::fs::create_dir_all(cache_path.parent().unwrap())?;

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

        let total_size = response.content_length();

        // Create temp file
        let temp_path = cache_path.with_extension("tmp");
        let mut file = tokio::fs::File::create(&temp_path).await?;

        // Stream download with progress
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;

        use futures::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to read download chunk")?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            if let Some(total) = total_size {
                let percent = (downloaded as f64 / total as f64) * 100.0;
                tracing::debug!("Downloaded {:.1}%", percent);
            }
        }

        file.sync_all().await?;
        drop(file);

        // Verify digest
        tracing::debug!("Verifying digest...");
        let file = std::fs::File::open(&temp_path)?;
        digest::verify_digest(&self.digest, file)
            .with_context(|| format!("Digest verification failed for: {}", self.url))?;

        // Move to final location
        std::fs::rename(&temp_path, &cache_path)?;

        let size = std::fs::metadata(&cache_path)?.len();
        tracing::info!("Downloaded and verified: {} ({} bytes)", self.url, size);

        Ok((cache_path, size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    #[ignore] // Requires network
    async fn test_fetch_tarball() {
        let temp_dir = TempDir::new().unwrap();

        // Use a small, stable file for testing
        let fetcher = TarballFetcher::new(
            "https://httpbin.org/bytes/1024",
            "sha256:0000000000000000000000000000000000000000000000000000000000000000", // This will fail
        );

        let result = fetcher.fetch(temp_dir.path()).await;
        // Should fail due to wrong digest
        assert!(result.is_err());
    }
}
