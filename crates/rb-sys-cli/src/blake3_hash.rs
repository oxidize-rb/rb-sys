use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// A typesafe BLAKE3 hash (32 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Blake3Hash([u8; 32]);

impl Blake3Hash {
    /// Create from 32 bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Create from a hex string (64 lowercase hex chars)
    pub fn from_hex(hex: &str) -> Result<Self, Blake3HashError> {
        if hex.len() != 64 {
            return Err(Blake3HashError::InvalidLength(hex.len()));
        }

        let mut bytes = [0u8; 32];
        for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
            let hex_byte =
                std::str::from_utf8(chunk).map_err(|_| Blake3HashError::InvalidHexChar)?;
            bytes[i] =
                u8::from_str_radix(hex_byte, 16).map_err(|_| Blake3HashError::InvalidHexChar)?;
        }

        Ok(Self(bytes))
    }

    /// Convert to lowercase hex string
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

impl fmt::Display for Blake3Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl FromStr for Blake3Hash {
    type Err = Blake3HashError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}

impl Serialize for Blake3Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Blake3Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Blake3HashError {
    #[error("Invalid BLAKE3 hash length: expected 64 hex chars, got {0}")]
    InvalidLength(usize),
    #[error("Invalid hex character in BLAKE3 hash")]
    InvalidHexChar,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hex() {
        let hex = "0000000000000000000000000000000000000000000000000000000000000000";
        let hash = Blake3Hash::from_hex(hex).unwrap();
        assert_eq!(hash.to_hex(), hex);
    }

    #[test]
    fn test_from_hex_invalid_length() {
        let hex = "00000000";
        assert!(matches!(
            Blake3Hash::from_hex(hex),
            Err(Blake3HashError::InvalidLength(8))
        ));
    }

    #[test]
    fn test_from_hex_invalid_char() {
        let hex = "ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ";
        assert!(matches!(
            Blake3Hash::from_hex(hex),
            Err(Blake3HashError::InvalidHexChar)
        ));
    }

    #[test]
    fn test_serde_roundtrip() {
        let hash = Blake3Hash::from_bytes([42u8; 32]);
        let json = serde_json::to_string(&hash).unwrap();
        let deserialized: Blake3Hash = serde_json::from_str(&json).unwrap();
        assert_eq!(hash, deserialized);
    }
}
