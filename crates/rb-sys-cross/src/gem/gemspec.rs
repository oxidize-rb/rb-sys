use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::cargo_metadata;

/// Metadata extracted from `cargo metadata` to build a gemspec.
#[derive(Debug, Clone)]
pub struct GemMetadata {
    pub name: String,
    pub version: String,
    pub crate_name: String,
    pub authors: Vec<String>,
    pub description: String,
    pub license: Option<String>,
    pub homepage: Option<String>,
}

/// Optional overrides from rb-sys-cross.toml.
#[derive(Debug, Default, Deserialize)]
pub struct CrossConfig {
    #[serde(default)]
    pub gem: GemConfig,
    #[serde(default)]
    pub build: BuildConfig,
}

#[derive(Debug, Default, Deserialize)]
pub struct GemConfig {
    pub name: Option<String>,
    pub require_paths: Option<Vec<String>>,
    pub files: Option<Vec<String>>,
    pub summary: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct BuildConfig {
    pub ext_dir: Option<String>,
}

impl CrossConfig {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?;
        toml::from_str(&contents).with_context(|| format!("parsing {}", path.display()))
    }
}

/// Extract gem metadata from `cargo metadata`.
pub fn from_cargo_metadata(manifest_path: &Path) -> Result<GemMetadata> {
    let metadata = cargo_metadata::query(manifest_path)?;

    // Find the package that owns the manifest
    let manifest_dir = manifest_path.parent().unwrap();
    let pkg = metadata
        .packages
        .iter()
        .find(|p| p.manifest_path.parent() == Some(manifest_dir))
        .or_else(|| metadata.packages.first())
        .context("no package found in cargo metadata")?;

    let crate_name = pkg
        .cdylib_name()
        .unwrap_or_else(|| pkg.name.replace('-', "_"));

    Ok(GemMetadata {
        name: pkg.name.clone(),
        version: pkg.version.clone(),
        crate_name,
        authors: pkg.authors.clone(),
        description: pkg.description.clone().unwrap_or_default(),
        license: pkg.license.clone(),
        homepage: pkg.homepage.clone(),
    })
}

/// Generate a gemspec YAML string for a native platform gem.
pub fn generate_gemspec_yaml(
    meta: &GemMetadata,
    ruby_platform: &str,
    ruby_versions: &[String],
    files: &[String],
    config: &CrossConfig,
) -> String {
    let gem_name = config.gem.name.as_deref().unwrap_or(&meta.name);
    let summary = config
        .gem
        .summary
        .as_deref()
        .unwrap_or(&meta.description);
    let default_require_paths = vec!["lib".to_string()];
    let require_paths = config
        .gem
        .require_paths
        .as_deref()
        .unwrap_or(&default_require_paths);

    let authors_yaml: String = if meta.authors.is_empty() {
        "[]".to_string()
    } else {
        meta.authors
            .iter()
            .map(|a| format!("\n- {a}"))
            .collect::<String>()
    };

    let require_paths_yaml: String = require_paths
        .iter()
        .map(|p| format!("- {p}"))
        .collect::<Vec<_>>()
        .join("\n");

    let files_yaml: String = if files.is_empty() {
        "[]".to_string()
    } else {
        files
            .iter()
            .map(|f| format!("\n- {f}"))
            .collect::<String>()
    };

    // Compute ruby version bounds from the versions we're building for.
    // Min is the lowest version, upper bound is next minor after highest + ".dev"
    let (min_ruby, max_ruby_exclusive) = ruby_version_bounds(ruby_versions);

    let mut yaml = format!(
        r#"--- !ruby/object:Gem::Specification
name: {gem_name}
version: !ruby/object:Gem::Version
  version: '{version}'
platform: {ruby_platform}
authors: {authors_yaml}
autorequire:
bindir: bin
cert_chain: []
dependencies: []
description: {summary}
email:
executables: []
extensions: []
extra_rdoc_files: []
files: {files_yaml}
"#,
        version = meta.version,
    );

    if let Some(ref homepage) = meta.homepage {
        yaml.push_str(&format!("homepage: {homepage}\n"));
    } else {
        yaml.push_str("homepage: \n");
    }

    if let Some(ref license) = meta.license {
        yaml.push_str(&format!("licenses:\n- {license}\n"));
    } else {
        yaml.push_str("licenses: []\n");
    }

    yaml.push_str(&format!(
        r#"metadata: {{}}
post_install_message:
rdoc_options: []
require_paths:
{require_paths_yaml}
required_ruby_version: !ruby/object:Gem::Requirement
  requirements:
  - - ">="
    - !ruby/object:Gem::Version
      version: '{min_ruby}'
  - - "<"
    - !ruby/object:Gem::Version
      version: {max_ruby_exclusive}
required_rubygems_version: !ruby/object:Gem::Requirement
  requirements:
  - - ">="
    - !ruby/object:Gem::Version
      version: '0'
requirements: []
rubygems_version: 3.5.0
signing_key:
specification_version: 4
summary: {summary}
test_files: []
"#
    ));

    yaml
}

/// Compute the min ruby version and exclusive upper bound from a list of versions.
/// e.g. ["3.2", "3.3", "3.4"] -> ("3.2", "3.5.dev")
/// e.g. ["3.3"] -> ("3.3", "3.4.dev")
fn ruby_version_bounds(versions: &[String]) -> (String, String) {
    if versions.is_empty() {
        return ("3.1".to_string(), "3.5.dev".to_string());
    }

    let mut parsed: Vec<(u32, u32)> = versions
        .iter()
        .filter_map(|v| {
            let parts: Vec<&str> = v.split('.').collect();
            if parts.len() >= 2 {
                Some((parts[0].parse().ok()?, parts[1].parse().ok()?))
            } else {
                None
            }
        })
        .collect();

    parsed.sort();

    let min = parsed.first().unwrap_or(&(3, 1));
    let max = parsed.last().unwrap_or(&(3, 4));

    let min_str = format!("{}.{}", min.0, min.1);
    let max_exclusive = format!("{}.{}.dev", max.0, max.1 + 1);

    (min_str, max_exclusive)
}
