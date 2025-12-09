use anyhow::{Context, Result};
use sha2::{Digest as _, Sha256, Sha512};
use std::io::Read;

/// Supported digest algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    Sha256,
    Sha512,
    Blake3,
}

impl Algorithm {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "sha256" => Ok(Algorithm::Sha256),
            "sha512" => Ok(Algorithm::Sha512),
            "blake3" => Ok(Algorithm::Blake3),
            _ => anyhow::bail!("Unsupported digest algorithm: {}", s),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Algorithm::Sha256 => "sha256",
            Algorithm::Sha512 => "sha512",
            Algorithm::Blake3 => "blake3",
        }
    }
}

/// Parse a digest string in format "algorithm:hex"
pub fn parse_digest(digest: &str) -> Result<(Algorithm, String)> {
    let parts: Vec<&str> = digest.split(':').collect();
    if parts.len() != 2 {
        anyhow::bail!(
            "Invalid digest format: expected 'algorithm:hex', got '{}'",
            digest
        );
    }

    let algorithm = Algorithm::from_str(parts[0])
        .with_context(|| format!("Invalid algorithm in digest: {}", parts[0]))?;
    let hex = parts[1].to_lowercase();

    // Validate hex length
    let expected_len = match algorithm {
        Algorithm::Sha256 => 64,
        Algorithm::Sha512 => 128,
        Algorithm::Blake3 => 64,
    };

    if hex.len() != expected_len {
        anyhow::bail!(
            "Invalid hex length for {}: expected {} chars, got {}",
            algorithm.as_str(),
            expected_len,
            hex.len()
        );
    }

    // Validate hex characters
    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        anyhow::bail!("Invalid hex characters in digest");
    }

    Ok((algorithm, hex))
}

/// Compute digest of a reader
pub fn compute_digest<R: Read>(algorithm: Algorithm, mut reader: R) -> Result<String> {
    match algorithm {
        Algorithm::Sha256 => {
            let mut hasher = Sha256::new();
            std::io::copy(&mut reader, &mut hasher)?;
            Ok(format!("{:x}", hasher.finalize()))
        }
        Algorithm::Sha512 => {
            let mut hasher = Sha512::new();
            std::io::copy(&mut reader, &mut hasher)?;
            Ok(format!("{:x}", hasher.finalize()))
        }
        Algorithm::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            std::io::copy(&mut reader, &mut hasher)?;
            Ok(hasher.finalize().to_hex().to_string())
        }
    }
}

/// Verify that a reader matches the expected digest
pub fn verify_digest<R: Read>(digest_str: &str, reader: R) -> Result<()> {
    let (algorithm, expected_hex) = parse_digest(digest_str)?;
    let actual_hex = compute_digest(algorithm, reader).context("Failed to compute digest")?;

    if actual_hex != expected_hex {
        anyhow::bail!(
            "Digest mismatch for {}:\n  Expected: {}\n  Actual:   {}",
            algorithm.as_str(),
            expected_hex,
            actual_hex
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_digest() {
        let (alg, hex) = parse_digest("sha256:abc123").unwrap();
        assert_eq!(alg, Algorithm::Sha256);
        assert!(hex.starts_with("abc123"));
    }

    #[test]
    fn test_compute_sha256() {
        let data = b"hello world";
        let digest = compute_digest(Algorithm::Sha256, &data[..]).unwrap();
        assert_eq!(
            digest,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_compute_blake3() {
        let data = b"hello world";
        let digest = compute_digest(Algorithm::Blake3, &data[..]).unwrap();
        assert_eq!(
            digest,
            "d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24"
        );
    }

    #[test]
    fn test_verify_digest_success() {
        let data = b"hello world";
        let digest = "sha256:b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert!(verify_digest(digest, &data[..]).is_ok());
    }

    #[test]
    fn test_verify_digest_failure() {
        let data = b"hello world";
        let digest = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
        assert!(verify_digest(digest, &data[..]).is_err());
    }
}
