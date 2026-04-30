use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::cargo_metadata;
use crate::platform::Platform;
use crate::profile::Profile;

use super::env::CrossCompileEnv;

/// Run `cargo zigbuild` to produce a shared library for the target platform.
///
/// Returns the path to the compiled artifact (.so/.dll).
pub fn zigbuild(
    manifest_path: &Path,
    platform: &Platform,
    profile: &Profile,
    features: &[String],
    env: &CrossCompileEnv,
) -> Result<PathBuf> {
    let mut cmd = Command::new("cargo");
    cmd.arg("zigbuild");
    cmd.arg("--manifest-path").arg(manifest_path);
    cmd.arg("--target").arg(platform.zigbuild_target());
    cmd.arg("--lib");

    for arg in profile.cargo_args() {
        cmd.arg(arg);
    }

    for f in features {
        cmd.arg("--features").arg(f);
    }

    // Inject all cross-compilation env vars
    for (k, v) in env.to_env_pairs() {
        cmd.env(k, v);
    }

    eprintln!(
        "Running: cargo zigbuild --target {} --lib",
        platform.zigbuild_target()
    );

    let status = cmd.status().context("failed to run cargo zigbuild")?;
    if !status.success() {
        bail!("cargo zigbuild failed with {status}");
    }

    find_artifact(manifest_path, platform, profile)
}

/// Locate the compiled shared library artifact in the target directory.
fn find_artifact(manifest_path: &Path, platform: &Platform, profile: &Profile) -> Result<PathBuf> {
    let metadata = cargo_metadata::query(manifest_path)?;

    let crate_name = metadata
        .packages
        .iter()
        .find_map(|pkg| pkg.cdylib_name())
        .context(
            "no cdylib target found in Cargo.toml. Add `crate-type = [\"cdylib\"]` to [lib].",
        )?;

    let ext = platform.shared_lib_ext();
    let lib_prefix = if platform.rust_target.contains("windows") {
        ""
    } else {
        "lib"
    };

    let artifact = metadata
        .target_directory
        .join(platform.rust_target)
        .join(profile.dir_name())
        .join(format!("{lib_prefix}{crate_name}.{ext}"));

    if !artifact.exists() {
        // Try without lib prefix as fallback
        let alt = metadata
            .target_directory
            .join(platform.rust_target)
            .join(profile.dir_name())
            .join(format!("{crate_name}.{ext}"));
        if alt.exists() {
            return Ok(alt);
        }
        bail!(
            "compiled artifact not found at {}\nExpected a cdylib output.",
            artifact.display()
        );
    }

    Ok(artifact)
}
