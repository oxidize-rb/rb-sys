use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::platform::Platform;
use crate::rbconfig::RbConfig;

/// Typed environment for a cross-compilation build.
pub struct CrossCompileEnv {
    pub ruby_hdrdir: PathBuf,
    pub ruby_archhdrdir: PathBuf,
    pub cargo_build_target: String,
    pub bindgen_extra_clang_args: String,
    pub rbconfig_vars: Vec<(String, String)>,
}

impl CrossCompileEnv {
    /// Build the cross-compilation environment from platform info and cached headers.
    pub fn new(platform: &Platform, header_dir: &Path, rbconfig: &RbConfig) -> Result<Self> {
        let mut rbconfig_vars = Vec::new();
        for (key, val) in rbconfig.iter() {
            rbconfig_vars.push((key.to_string(), val.to_string()));
        }

        let include_dir = header_dir.join("include");
        let ruby_hdrdir = find_ruby_version_dir(&include_dir)?;
        let ruby_archhdrdir = find_arch_include(&ruby_hdrdir)?;

        let bindgen_extra_clang_args = format!(
            "-I{} -I{} --target={}",
            ruby_hdrdir.display(),
            ruby_archhdrdir.display(),
            platform.rust_target,
        );

        Ok(Self {
            ruby_hdrdir,
            ruby_archhdrdir,
            cargo_build_target: platform.rust_target.to_string(),
            bindgen_extra_clang_args,
            rbconfig_vars,
        })
    }

    /// Convert into environment variable key-value pairs for passing to a Command.
    pub fn to_env_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::with_capacity(self.rbconfig_vars.len() + 5);

        for (key, val) in &self.rbconfig_vars {
            pairs.push((format!("RBCONFIG_{key}"), val.clone()));
        }

        pairs.push(("RBCONFIG_CROSS_COMPILING".into(), "yes".into()));
        pairs.push((
            "RBCONFIG_rubyhdrdir".into(),
            self.ruby_hdrdir.to_string_lossy().into_owned(),
        ));
        pairs.push((
            "RBCONFIG_rubyarchhdrdir".into(),
            self.ruby_archhdrdir.to_string_lossy().into_owned(),
        ));
        pairs.push((
            "BINDGEN_EXTRA_CLANG_ARGS".into(),
            self.bindgen_extra_clang_args.clone(),
        ));
        pairs.push(("CARGO_BUILD_TARGET".into(), self.cargo_build_target.clone()));

        pairs
    }
}

/// Find the ruby version directory (e.g., include/ruby-3.3.0/).
fn find_ruby_version_dir(include_dir: &Path) -> Result<PathBuf> {
    if include_dir.exists() {
        for entry in std::fs::read_dir(include_dir).context("reading include dir")? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && entry.file_name().to_string_lossy().starts_with("ruby-") {
                return Ok(path);
            }
        }
    }
    // Fall back to include/ itself
    Ok(include_dir.to_path_buf())
}

/// Find the architecture-specific include directory (e.g., ruby-3.3.0/aarch64-linux/).
/// This contains the platform-specific ruby/config.h.
fn find_arch_include(ruby_version_dir: &Path) -> Result<PathBuf> {
    if ruby_version_dir.exists() {
        for entry in std::fs::read_dir(ruby_version_dir).context("reading ruby version dir")? {
            let entry = entry?;
            if entry.path().is_dir() && entry.path().join("ruby").join("config.h").exists() {
                return Ok(entry.path());
            }
        }
    }
    // Fall back to the version dir itself
    Ok(ruby_version_dir.to_path_buf())
}
