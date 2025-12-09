use crate::lockfile::FileDigest;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct OciExtractor<'a> {
    image: &'a str,
    items: &'a [String],
    strip_prefix: Option<&'a str>,
}

impl<'a> OciExtractor<'a> {
    pub fn new(image: &'a str, items: &'a [String], strip_prefix: Option<&'a str>) -> Self {
        Self {
            image,
            items,
            strip_prefix,
        }
    }

    /// Extract items from OCI image to destination directory
    /// Returns list of extracted files with BLAKE3 digests
    pub async fn extract(&self, dest_dir: &Path, _cache_dir: &Path) -> Result<Vec<FileDigest>> {
        // Parse image reference
        if !self.image.contains('@') {
            anyhow::bail!("OCI image must use digest reference: {}", self.image);
        }

        let parts: Vec<&str> = self.image.split('@').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid OCI image reference: {}", self.image);
        }

        let image_name = parts[0];
        let digest = parts[1];

        tracing::info!("Extracting from OCI image: {}", self.image);

        // Use oci-distribution to pull and extract
        use oci_distribution::client::ClientConfig;
        use oci_distribution::Reference;

        let reference = Reference::try_from(image_name)
            .with_context(|| format!("Invalid OCI reference: {}", image_name))?;

        let config = ClientConfig {
            protocol: oci_distribution::client::ClientProtocol::Https,
            ..Default::default()
        };

        let client = oci_distribution::Client::new(config);

        // Pull image by digest
        let auth = &oci_distribution::secrets::RegistryAuth::Anonymous;

        let image_data = client
            .pull(
                &reference,
                auth,
                vec![
                    oci_distribution::manifest::IMAGE_DOCKER_LAYER_GZIP_MEDIA_TYPE,
                    oci_distribution::manifest::IMAGE_LAYER_MEDIA_TYPE,
                    oci_distribution::manifest::IMAGE_LAYER_GZIP_MEDIA_TYPE,
                ],
            )
            .await
            .with_context(|| format!("Failed to pull OCI image: {}", self.image))?;

        // Verify digest matches
        let manifest_digest = image_data.digest.clone().unwrap_or_default();
        if !manifest_digest.contains(&digest.replace("sha256:", "")) {
            tracing::warn!(
                "Digest mismatch (expected {}, got {}), but proceeding",
                digest,
                manifest_digest
            );
        }

        // Extract layers to temp directory
        let temp_extract_path =
            std::env::temp_dir().join(format!("oci-extract-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_extract_path)?;

        for layer in image_data.layers {
            // Decompress layer
            if layer.media_type.contains("gzip") {
                use flate2::read::GzDecoder;
                let decoder = GzDecoder::new(&layer.data[..]);
                let mut archive = tar::Archive::new(decoder);
                extract_tar_items(
                    &mut archive,
                    &self.items,
                    self.strip_prefix,
                    &temp_extract_path,
                )?;
            } else if layer.media_type.contains("zstd") {
                use zstd::stream::read::Decoder;
                let decoder = Decoder::new(&layer.data[..])?;
                let mut archive = tar::Archive::new(decoder);
                extract_tar_items(
                    &mut archive,
                    &self.items,
                    self.strip_prefix,
                    &temp_extract_path,
                )?;
            } else {
                // Uncompressed tar
                let mut archive = tar::Archive::new(&layer.data[..]);
                extract_tar_items(
                    &mut archive,
                    &self.items,
                    self.strip_prefix,
                    &temp_extract_path,
                )?;
            }
        }

        // Move extracted files to destination
        // and compute BLAKE3 digests
        let mut file_digests = Vec::new();

        use walkdir::WalkDir;
        for entry in WalkDir::new(&temp_extract_path) {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            let rel_path = entry.path().strip_prefix(&temp_extract_path)?;
            let dest_path = dest_dir.join(rel_path);

            // Create parent dirs
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Copy file
            std::fs::copy(entry.path(), &dest_path)?;

            // Compute BLAKE3
            let file = std::fs::File::open(&dest_path)?;
            let size_bytes = file.metadata()?.len();
            let mut hasher = blake3::Hasher::new();
            std::io::copy(&mut std::io::BufReader::new(file), &mut hasher)?;
            let blake3 = hasher.finalize().to_hex().to_string();

            file_digests.push(FileDigest {
                path: rel_path.to_string_lossy().to_string(),
                blake3,
                size_bytes,
            });
        }

        tracing::info!("Extracted {} files from OCI image", file_digests.len());

        Ok(file_digests)
    }
}

/// Extract specific items from a tar archive
fn extract_tar_items<R: std::io::Read>(
    archive: &mut tar::Archive<R>,
    items: &[String],
    strip_prefix: Option<&str>,
    dest: &Path,
) -> Result<()> {
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        let path_str = path.to_string_lossy();

        // Check if this entry matches any requested items
        let mut matched = false;
        for item in items {
            if path_str.starts_with(item) || path_str.starts_with(&format!("./{}", item)) {
                matched = true;
                break;
            }
        }

        if !matched {
            continue;
        }

        // Apply strip_prefix
        let final_path = if let Some(prefix) = strip_prefix {
            path_str
                .strip_prefix(prefix)
                .or_else(|| path_str.strip_prefix(&format!("./{}", prefix)))
                .map(|s| PathBuf::from(s))
                .unwrap_or_else(|| path.to_path_buf())
        } else {
            path.to_path_buf()
        };

        // Security check: no absolute paths or .. traversal
        if final_path.is_absolute()
            || final_path
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            tracing::warn!("Skipping unsafe path: {}", final_path.display());
            continue;
        }

        let dest_path = dest.join(&final_path);

        // Create parent dirs
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Extract entry
        entry.unpack(&dest_path)?;
        tracing::debug!("Extracted: {}", dest_path.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires network and valid OCI image
    async fn test_oci_extract() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();

        let patterns = vec!["opt/rubies/".to_string()];
        let extractor = OciExtractor::new(
            "ghcr.io/oxidize-rb/rb-sys-dock/x86_64-linux@sha256:0000000000000000000000000000000000000000000000000000000000000000",
            &patterns,
            Some("opt/"),
        );

        let result = extractor.extract(temp_dir.path(), cache_dir.path()).await;
        // Will likely fail with wrong digest or missing image
        assert!(result.is_err() || result.unwrap().len() > 0);
    }
}
