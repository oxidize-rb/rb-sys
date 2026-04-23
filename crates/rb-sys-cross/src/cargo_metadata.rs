use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use serde::Deserialize;

/// Typed subset of `cargo metadata --format-version=1` output.
#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub packages: Vec<Package>,
    pub target_directory: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub manifest_path: PathBuf,
    pub targets: Vec<Target>,
}

#[derive(Debug, Deserialize)]
pub struct Target {
    pub name: String,
    pub kind: Vec<String>,
}

impl Package {
    /// Find the cdylib target name, normalized (hyphens → underscores).
    pub fn cdylib_name(&self) -> Option<String> {
        self.targets.iter().find_map(|t| {
            if t.kind.iter().any(|k| k == "cdylib") {
                Some(t.name.replace('-', "_"))
            } else {
                None
            }
        })
    }
}

/// Run `cargo metadata --no-deps` and deserialize the result.
pub fn query(manifest_path: &Path) -> Result<Metadata> {
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--format-version=1",
            "--no-deps",
            "--manifest-path",
        ])
        .arg(manifest_path)
        .output()
        .context("running cargo metadata")?;

    serde_json::from_slice(&output.stdout).context("parsing cargo metadata JSON")
}
