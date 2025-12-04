use anyhow::{Context, Result};
use clap::Args;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::TempDir;
use tracing::{debug, info, instrument};

use crate::shim_generator::ShimGenerator;
use crate::toolchain::ToolchainInfo;

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

    /// Glibc version for Linux targets (e.g., "2.17")
    #[arg(long, env = "GEM_FORGE_GLIBC")]
    pub glibc_version: Option<String>,

    /// Working directory (defaults to current directory)
    #[arg(long)]
    pub manifest_path: Option<PathBuf>,

    /// Additional cargo arguments
    #[arg(last = true)]
    pub cargo_args: Vec<String>,
}

/// Build a native gem for the specified target
#[instrument(skip(config), fields(target = %config.target, profile = %config.profile))]
pub fn build(config: &BuildConfig) -> Result<()> {
    info!(target = %config.target, "Building native gem");

    // Validate zig is available
    validate_zig(&config.zig_path)?;

    // Load toolchain info
    let toolchain = ToolchainInfo::find_by_rust_target(&config.target)
        .context("Failed to find toolchain for target")?;

    info!(ruby_platform = %toolchain.ruby_platform, zig_target = %toolchain.zig_target(), "Using toolchain");
    debug!(?toolchain, "Full toolchain info");

    // Create temporary directory for shims
    let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
    let shim_dir = temp_dir.path().join("shims");

    // Generate shims
    info!("Generating compiler shims");
    let shim_gen = ShimGenerator::new(shim_dir.clone(), config.zig_path.clone());
    shim_gen.generate().context("Failed to generate shims")?;

    // Get environment variables
    let mut env_vars = shim_gen.get_shim_env(&config.target, None);

    // Add glibc version if specified
    if let Some(ref glibc) = config.glibc_version {
        env_vars.insert("GEM_FORGE_GLIBC".to_string(), glibc.clone());
        info!(glibc = %glibc, "Targeting glibc version");
    }

    // NOTE: We do NOT add the shim directory to PATH because that would affect
    // host builds (like proc-macros). Instead, we rely on the target-specific
    // CC_<target> environment variables set by get_shim_env().

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
    debug!(?env_vars, "Environment variables set for build");
    debug!(?cmd, "Cargo command to execute");

    // Execute cargo build
    let status = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to execute cargo build")?;

    if !status.success() {
        anyhow::bail!("Cargo build failed with exit code: {:?}", status.code());
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

/// Validate that zig is available and working
fn validate_zig(zig_path: &Path) -> Result<()> {
    let output = Command::new(zig_path)
        .arg("version")
        .output()
        .context("Failed to execute zig - is it installed?")?;

    if !output.status.success() {
        anyhow::bail!("Zig command failed - is zig installed correctly?");
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
        println!(
            "  • {} ({})",
            tc.rust_target,
            tc.ruby_platform
        );
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
        assert!(cli.config.glibc_version.is_none());
    }
}
