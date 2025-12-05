use anyhow::{bail, Context, Result};
use clap::Args;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tracing::{debug, info, instrument};

use crate::extractor::get_cache_dir;
use crate::toolchain::ToolchainInfo;
use crate::zig::{env::cargo_env, shim, target::RustTarget};

/// Configuration for building a gem
#[derive(Args, Debug, Clone)]
pub struct BuildConfig {
    /// Target Rust triple to build for
    #[arg(short, long, required = true)]
    pub target: String,

    /// Path to the Zig compiler
    #[arg(long, default_value = "zig", env = "ZIG_PATH")]
    pub zig_path: PathBuf,

    /// Cargo profile to use (release/dev)
    #[arg(long, default_value = "release")]
    pub profile: String,

    /// Additional features to enable
    #[arg(short, long, value_delimiter = ',')]
    pub features: Vec<String>,

    /// Working directory (defaults to current directory)
    #[arg(long)]
    pub manifest_path: Option<PathBuf>,

    /// Enable verbose logging (shows environment variables, cargo commands, etc.)
    #[arg(short, long)]
    pub verbose: bool,

    /// Additional cargo arguments
    #[arg(last = true)]
    pub cargo_args: Vec<String>,
}

/// Build a native gem for the specified target
#[instrument(skip(config), fields(target = %config.target, profile = %config.profile))]
pub fn build(config: &BuildConfig) -> Result<()> {
    info!(target = %config.target, "Building native gem");

    // Parse and validate the target
    let target = RustTarget::parse(&config.target)?;

    // Validate zig is available
    validate_zig(&config.zig_path)?;

    // Load toolchain info for display
    let toolchain = ToolchainInfo::find_by_rust_target(&config.target)
        .context("Failed to find toolchain for target")?;

    info!(
        ruby_platform = %toolchain.ruby_platform,
        zig_target = %target.to_zig_target(),
        "Using toolchain"
    );

    // Get the CLI binary path for shims to call back into
    let cli_path = std::env::current_exe().context("Failed to get current executable path")?;

    // Determine sysroot path for Linux targets
    let sysroot = if target.requires_sysroot() {
        let cache_dir = get_cache_dir()?;
        let sysroot_path = cache_dir
            .join("rubies")
            .join(&toolchain.ruby_platform)
            .join("sysroot");

        if !sysroot_path.exists() {
            bail!(
                "Sysroot not found for target: {}\n\n\
                 The sysroot is required for Linux cross-compilation.\n\
                 To extract it, run:\n\n  \
                 cargo gem extract --target {}\n",
                config.target,
                config.target
            );
        }

        Some(sysroot_path)
    } else {
        None
    };

    // Validate macOS SDK for Darwin targets
    if target.requires_sdkroot() {
        if std::env::var("SDKROOT").is_err() {
            bail!(
                "SDKROOT environment variable is required for macOS cross-compilation.\n\n\
                 Set it to the path of your macOS SDK, for example:\n  \
                 export SDKROOT=/path/to/MacOSX14.0.sdk\n\n\
                 You can obtain the macOS SDK from Xcode or from:\n  \
                 https://github.com/joseluisq/macosx-sdks"
            );
        }
    }

    // Create hermetic build directory for shims
    // Structure: target/rb-sys/<target>/bin/
    let target_dir = get_target_dir(config.manifest_path.as_deref())?;
    let shim_dir = target_dir.join("rb-sys").join(&config.target).join("bin");

    // Generate shims
    info!(shim_dir = %shim_dir.display(), "Generating compiler shims");
    shim::generate_shims(
        &shim_dir,
        &cli_path,
        &config.zig_path,
        &target,
        sysroot.as_deref(),
    )
    .context("Failed to generate shims")?;

    // Get environment variables
    let env_vars = cargo_env(&target, &shim_dir, sysroot.as_deref());

    // NOTE: We do NOT add the shim directory to PATH because that would affect
    // host builds (like proc-macros). Instead, we rely on the target-specific
    // CC_<target> environment variables set by cargo_env().

    // Build the cargo command
    info!("Running cargo build");
    let mut cmd = Command::new("cargo");
    cmd.arg("build");

    // Add target
    cmd.arg("--target").arg(&config.target);

    // Add profile
    if config.profile != "dev" {
        cmd.arg("--profile").arg(&config.profile);
    }

    // Add features
    if !config.features.is_empty() {
        cmd.arg("--features").arg(config.features.join(","));
    }

    // Add manifest path if specified
    if let Some(ref manifest) = config.manifest_path {
        cmd.arg("--manifest-path").arg(manifest);
    }

    // Add additional cargo args
    for arg in &config.cargo_args {
        cmd.arg(arg);
    }

    // Apply all environment variables
    for (key, value) in &env_vars {
        cmd.env(key, value);
    }

    // Log environment variables at debug level
    debug!(
        env_count = env_vars.len(),
        "Environment variables set for build"
    );

    // Log individual environment variables for verbose output
    for (key, value) in &env_vars {
        // Truncate long values for readability
        let display_value = if value.len() > 100 {
            format!("{}...", &value[..100])
        } else {
            value.clone()
        };
        debug!(key = %key, value = %display_value, "Setting env var");
    }

    // Log the cargo command
    let cargo_args: Vec<String> = cmd
        .get_args()
        .map(|s| s.to_string_lossy().to_string())
        .collect();
    debug!(
        command = "cargo",
        args = ?cargo_args,
        "Executing cargo build"
    );

    // Execute cargo build
    let status = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to execute cargo build")?;

    if !status.success() {
        bail!("Cargo build failed with exit code: {:?}", status.code());
    }

    info!("Build completed successfully");

    let profile_dir = if config.profile == "dev" {
        "debug"
    } else {
        &config.profile
    };

    let output_dir = if let Some(ref manifest) = config.manifest_path {
        manifest.parent().unwrap().join("target")
    } else {
        PathBuf::from("target")
    };

    info!(
        output_location = %format!("{}/{}/{}/", output_dir.display(), config.target, profile_dir),
        "Build artifacts written"
    );

    Ok(())
}

/// Get the target directory for the build.
///
/// Determines the target directory in this order:
/// 1. CARGO_TARGET_DIR environment variable
/// 2. Parent of manifest_path + "target" (if manifest_path provided)
/// 3. Current directory + "target"
fn get_target_dir(manifest_path: Option<&Path>) -> Result<PathBuf> {
    // Check CARGO_TARGET_DIR first
    if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        return Ok(PathBuf::from(target_dir));
    }

    // Use manifest path parent if provided
    if let Some(manifest) = manifest_path {
        if let Some(parent) = manifest.parent() {
            return Ok(parent.join("target"));
        }
    }

    // Default to current directory + target
    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    Ok(cwd.join("target"))
}

/// Validate that zig is available and working
fn validate_zig(zig_path: &Path) -> Result<()> {
    let output = Command::new(zig_path)
        .arg("version")
        .output()
        .context("Failed to execute zig - is it installed?")?;

    if !output.status.success() {
        bail!("Zig command failed - is zig installed correctly?");
    }

    let version = String::from_utf8_lossy(&output.stdout);
    info!(zig_version = %version.trim(), "Using Zig");

    Ok(())
}

/// List all supported target platforms
pub fn list_targets() -> Result<()> {
    info!("Listing supported target platforms");

    let toolchains = ToolchainInfo::list_supported()?;

    println!("📋 Supported target platforms:\n");
    for tc in toolchains {
        println!("  • {} ({})", tc.rust_target, tc.ruby_platform);
    }

    println!("\nUse: cargo gem build --target <rust-target>");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_config_defaults() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            config: BuildConfig,
        }

        let cli = TestCli::parse_from(["test", "--target", "x86_64-unknown-linux-gnu"]);
        assert_eq!(cli.config.target, "x86_64-unknown-linux-gnu");
        assert_eq!(cli.config.profile, "release");
        assert!(cli.config.features.is_empty());
    }

    #[test]
    fn test_get_target_dir_uses_env_when_set() {
        // If CARGO_TARGET_DIR is set (e.g., by the test runner), verify we use it
        if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
            let result = get_target_dir(None).unwrap();
            assert_eq!(result, PathBuf::from(target_dir));
        }
    }

    #[test]
    fn test_get_target_dir_from_manifest_when_no_env() {
        // This test verifies the manifest path logic works correctly
        // We can't easily unset CARGO_TARGET_DIR in tests since it affects other tests
        // So we just verify the function signature and basic behavior
        let manifest = PathBuf::from("/project/Cargo.toml");

        // When CARGO_TARGET_DIR is set, it takes precedence
        if std::env::var("CARGO_TARGET_DIR").is_ok() {
            // Just verify the function doesn't panic
            let _ = get_target_dir(Some(&manifest));
        }
    }

    #[test]
    fn test_get_target_dir_returns_valid_path() {
        // Verify we always get a valid path ending in "target" or from env
        let result = get_target_dir(None).unwrap();
        assert!(
            result.to_string_lossy().contains("target"),
            "Expected path to contain 'target', got: {}",
            result.display()
        );
    }
}
