use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

/// A typed wrapper around Ruby's rbconfig key-value pairs.
#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct RbConfig(HashMap<String, String>);

impl RbConfig {
    /// Load from an rbconfig.json file on disk.
    pub fn from_json_file(path: &Path) -> Result<Self> {
        let contents =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        serde_json::from_str(&contents).with_context(|| format!("parsing {}", path.display()))
    }

    /// Iterate over all key-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Get a specific rbconfig value.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|s| s.as_str())
    }
}
