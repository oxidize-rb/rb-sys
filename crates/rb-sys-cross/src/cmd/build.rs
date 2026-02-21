use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::Args;

use crate::build::cargo;
use crate::build::env::CrossCompileEnv;
use crate::gem::gemspec::{self, CrossConfig};
use crate::gem::pack::{self, GemPackOptions, VersionedArtifact};
use crate::headers::download;
use crate::platform::Platform;
use crate::profile::Profile;
use crate::toolchain;

/// CLI options for the `build` subcommand.
#[derive(Args)]
pub struct BuildOpts {
    /// Target platform (repeatable). e.g. aarch64-linux, x86_64-linux
    #[arg(long, short = 'p', required = true)]
    pub platform: Vec<String>,

    /// Ruby version (repeatable). e.g. 3.2, 3.3, 3.4
    #[arg(long, short = 'r', required = true)]
    pub ruby_version: Vec<String>,

    /// Path to Cargo.toml
    #[arg(long, default_value = "Cargo.toml")]
    pub manifest_path: PathBuf,

    /// Build profile
    #[arg(long, default_value = "release")]
    pub profile: Profile,

    /// Output directory for .gem files
    #[arg(long, default_value = "pkg")]
    pub output_dir: PathBuf,

    /// Cargo features to enable (repeatable)
    #[arg(long)]
    pub features: Vec<String>,

    /// Path to rb-sys-cross.toml config
    #[arg(long)]
    pub config: Option<PathBuf>,
}

impl BuildOpts {
    /// Resolve the config path, defaulting to rb-sys-cross.toml next to Cargo.toml.
    fn config_path(&self) -> PathBuf {
        self.config
            .clone()
            .unwrap_or_else(|| self.manifest_path.parent().unwrap().join("rb-sys-cross.toml"))
    }
}

pub fn run(opts: BuildOpts) -> Result<()> {
    // Load optional config
    let config = CrossConfig::load(&opts.config_path())?;

    // Read gem metadata from Cargo.toml
    let meta = gemspec::from_cargo_metadata(&opts.manifest_path)?;
    eprintln!(
        "Building {} v{} (crate: {})",
        meta.name, meta.version, meta.crate_name
    );

    // Collect extra Ruby files if configured
    let extra_files = collect_extra_files(&opts.manifest_path, &config)?;

    let mut gem_paths = Vec::new();

    for platform_name in &opts.platform {
        let plat = Platform::find(platform_name)?;

        if !plat.zig_supported {
            bail!(
                "platform {} does not support zig cross-compilation (no macOS SDK available via zig)",
                plat.ruby_platform
            );
        }

        // Ensure toolchain prerequisites
        toolchain::ensure_zig()?;
        toolchain::ensure_cargo_zigbuild()?;
        toolchain::ensure_rust_target(plat)?;

        // Collect versioned artifacts for this platform
        let mut artifacts = Vec::new();

        for ruby_version in &opts.ruby_version {
            eprintln!("\n--- {platform_name} / ruby {ruby_version} ---");

            // Download/cache Ruby headers
            let header_dir = download::ensure_headers(plat.ruby_platform, ruby_version)?;

            // Load rbconfig and build cross-compile env
            let rbconfig = download::load_rbconfig(&header_dir)?;
            let build_env = CrossCompileEnv::new(plat, &header_dir, &rbconfig)?;

            // Run cargo zigbuild
            let artifact = cargo::zigbuild(
                &opts.manifest_path,
                plat,
                &opts.profile,
                &opts.features,
                &build_env,
            )?;

            eprintln!("Compiled: {}", artifact.display());

            artifacts.push(VersionedArtifact {
                ruby_version: ruby_version.clone(),
                artifact_path: artifact,
            });
        }

        // Pack all ruby versions into a single native gem for this platform
        let gem_path = pack::pack_native_gem(&GemPackOptions {
            meta: &meta,
            ruby_platform: plat.ruby_platform,
            artifacts: &artifacts,
            lib_ext: plat.shared_lib_ext(),
            extra_files: &extra_files,
            config: &config,
            output_dir: &opts.output_dir,
        })?;

        gem_paths.push(gem_path);
    }

    eprintln!("\nBuilt {} gem(s):", gem_paths.len());
    for p in &gem_paths {
        eprintln!("  {}", p.display());
    }

    Ok(())
}

/// Collect extra Ruby files specified in the config.
fn collect_extra_files(
    manifest_path: &Path,
    config: &CrossConfig,
) -> Result<Vec<(PathBuf, Vec<u8>)>> {
    let mut files = Vec::new();
    let base_dir = manifest_path.parent().unwrap();

    if let Some(ref patterns) = config.gem.files {
        for pattern in patterns {
            let full_pattern = base_dir.join(pattern);
            let pattern_str = full_pattern.to_string_lossy();
            for entry in glob::glob(&pattern_str)
                .with_context(|| format!("invalid glob pattern: {pattern}"))?
            {
                let path = entry.context("glob error")?;
                if path.is_file() {
                    let relative = path.strip_prefix(base_dir).unwrap_or(&path);
                    let contents = std::fs::read(&path)
                        .with_context(|| format!("reading {}", path.display()))?;
                    files.push((relative.to_path_buf(), contents));
                }
            }
        }
    }

    Ok(files)
}
