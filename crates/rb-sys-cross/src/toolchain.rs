use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::platform::Platform;

/// Ensure zig is available on PATH.
pub fn ensure_zig() -> Result<()> {
    which::which("zig").context(
        "zig not found on PATH.\n\
         Install from https://ziglang.org/download/ or run: brew install zig",
    )?;
    Ok(())
}

/// Ensure cargo-zigbuild is available.
pub fn ensure_cargo_zigbuild() -> Result<()> {
    which::which("cargo-zigbuild").context(
        "cargo-zigbuild not found.\n\
         Install with: cargo install cargo-zigbuild",
    )?;
    Ok(())
}

/// Ensure the Rust target is installed via rustup.
pub fn ensure_rust_target(platform: &Platform) -> Result<()> {
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .context("failed to run rustup")?;

    let installed = String::from_utf8_lossy(&output.stdout);
    if installed.lines().any(|l| l.trim() == platform.rust_target) {
        return Ok(());
    }

    eprintln!(
        "Installing Rust target {} via rustup...",
        platform.rust_target
    );
    let status = Command::new("rustup")
        .args(["target", "add", platform.rust_target])
        .status()
        .context("failed to run rustup target add")?;

    if !status.success() {
        bail!(
            "rustup target add {} failed with {}",
            platform.rust_target,
            status
        );
    }

    Ok(())
}

/// Install all prerequisites for cross-compilation.
pub fn setup_all(platforms: &[&Platform]) -> Result<()> {
    ensure_zig()?;
    eprintln!("  zig: ok");

    ensure_cargo_zigbuild()?;
    eprintln!("  cargo-zigbuild: ok");

    for p in platforms {
        ensure_rust_target(p)?;
        eprintln!("  rust target {}: ok", p.rust_target);
    }

    Ok(())
}
