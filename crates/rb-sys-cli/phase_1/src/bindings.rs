//! Generate pre-generated Ruby bindings for all ruby_version × rust_target combinations.
//!
//! This module generates bindings at build-time (phase_1) so that end users don't need
//! libclang or bindgen dependencies.
//!
//! ## Key Design: Zig libc for Cross-Compilation
//!
//! Instead of extracting sysroot headers from Docker images (which are often wrong
//! architecture), we use Zig's bundled libc headers. Zig ships with complete C library
//! headers for all supported targets (Linux glibc, Linux musl, Windows mingw, macOS).
//!
//! This gives us:
//! - Correct headers for all architectures
//! - No dependency on Docker sysroot extraction
//! - Consistent results across build machines

use anyhow::{Context, Result};
use rb_sys_build::RbConfig;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

use crate::config::Config;

/// Information about a discovered Ruby installation.
#[derive(Debug, Clone)]
pub struct RubyInfo {
    /// Ruby version (e.g., "3.3.9")
    pub version: String,
    /// Path to rbconfig.json
    pub rbconfig_json: PathBuf,
}

/// Discover all extracted Ruby installations in the cache directory.
///
/// Returns a map of ruby_platform -> Vec<RubyInfo>
pub fn discover_rubies(cache_dir: &Path) -> Result<HashMap<String, Vec<RubyInfo>>> {
    let mut rubies: HashMap<String, Vec<RubyInfo>> = HashMap::new();

    // Iterate over platform directories (e.g., "aarch64-linux", "x64-mingw-ucrt")
    let entries = fs::read_dir(cache_dir)
        .with_context(|| format!("Failed to read cache directory: {}", cache_dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let platform_path = entry.path();

        if !platform_path.is_dir() {
            continue;
        }

        let platform_name = platform_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Skip non-platform directories
        if platform_name == "bindings"
            || platform_name == "zig"
            || platform_name == "sysroot"
            || platform_name.starts_with('.')
        {
            continue;
        }

        // Look for rubies subdirectory
        let rubies_dir = platform_path.join("rubies");
        if !rubies_dir.exists() {
            continue;
        }

        // Find all ruby installations with rbconfig.json
        let ruby_infos = discover_ruby_versions(&rubies_dir)?;
        if !ruby_infos.is_empty() {
            rubies.insert(platform_name, ruby_infos);
        }
    }

    Ok(rubies)
}

/// Discover Ruby versions within a platform's rubies directory.
///
/// Returns Vec<RubyInfo> for each Ruby installation that has an rbconfig.json.
fn discover_ruby_versions(rubies_dir: &Path) -> Result<Vec<RubyInfo>> {
    let mut versions = Vec::new();

    // Each platform has a subdirectory with the arch name (e.g., "aarch64-linux-gnu")
    let arch_entries = fs::read_dir(rubies_dir)
        .with_context(|| format!("Failed to read rubies dir: {}", rubies_dir.display()))?;

    for arch_entry in arch_entries {
        let arch_entry = arch_entry?;
        let arch_path = arch_entry.path();

        if !arch_path.is_dir() {
            continue;
        }

        // Look for ruby-X.Y.Z directories
        let ruby_entries = fs::read_dir(&arch_path).with_context(|| {
            format!("Failed to read arch rubies dir: {}", arch_path.display())
        })?;

        for ruby_entry in ruby_entries {
            let ruby_entry = ruby_entry?;
            let ruby_path = ruby_entry.path();

            if !ruby_path.is_dir() {
                continue;
            }

            let ruby_dir_name = ruby_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if ruby_dir_name.starts_with("ruby-") {
                let version = ruby_dir_name.trim_start_matches("ruby-").to_string();

                // Find rbconfig.json in this ruby installation
                if let Some(rbconfig_json) = find_rbconfig_json(&ruby_path)? {
                    versions.push(RubyInfo {
                        version,
                        rbconfig_json,
                    });
                }
            }
        }
    }

    Ok(versions)
}

/// Find rbconfig.json within a Ruby installation directory.
fn find_rbconfig_json(ruby_root: &Path) -> Result<Option<PathBuf>> {
    // Walk the directory tree looking for rbconfig.json
    for entry in walkdir::WalkDir::new(ruby_root)
        .max_depth(6)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.file_name() == Some(std::ffi::OsStr::new("rbconfig.json")) {
            return Ok(Some(path.to_path_buf()));
        }
    }

    Ok(None)
}

/// Find the Zig installation directory.
///
/// Looks for Zig in the cache directory (downloaded by phase_0).
fn find_zig_installation(cache_dir: &Path) -> Result<PathBuf> {
    // Look for zig directory in cache
    let zig_dir = cache_dir.join("zig");

    if !zig_dir.exists() {
        anyhow::bail!(
            "Zig not found at {}. Run `phase_0 download-zig` first.",
            zig_dir.display()
        );
    }

    // Find the actual zig installation (it's versioned, e.g., zig-macos-aarch64-0.13.0/)
    for entry in fs::read_dir(&zig_dir)
        .with_context(|| format!("Failed to read zig directory: {}", zig_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with("zig-") {
                // Verify lib/libc exists
                let libc_path = path.join("lib").join("libc");
                if libc_path.exists() {
                    return Ok(path);
                }
            }
        }
    }

    anyhow::bail!(
        "No valid Zig installation found in {}. Run `phase_0 download-zig` first.",
        zig_dir.display()
    );
}

/// Get Zig libc include paths for a given Rust target.
///
/// Zig ships with complete C library headers for all supported targets.
/// The structure is:
/// - lib/libc/include/generic-glibc/  (common glibc headers like stdio.h)
/// - lib/libc/include/aarch64-linux-gnu/  (arch-specific headers)
/// - lib/libc/include/any-linux-any/  (Linux-common headers)
/// - lib/libc/include/any-windows-any/  (Windows headers)
/// - lib/libc/include/any-macos-any/  (macOS headers)
///
/// IMPORTANT: Include order matters! Arch-specific directories must come BEFORE
/// generic directories so that arch-specific headers (like bits/wordsize.h)
/// override generic ones.
fn get_zig_libc_includes(zig_path: &Path, rust_target: &str) -> Vec<String> {
    let libc_include = zig_path.join("lib").join("libc").join("include");

    let mut includes = Vec::new();

    // Determine which headers to use based on target
    // Order: arch-specific FIRST, then generic (so arch overrides generic)
    if rust_target.contains("linux") {
        if rust_target.contains("musl") {
            // Architecture-specific musl headers FIRST
            let arch_dir = match rust_target {
                t if t.starts_with("aarch64") => Some("aarch64-linux-musl"),
                t if t.starts_with("x86_64") => Some("x86_64-linux-musl"),
                t if t.starts_with("arm") => Some("arm-linux-musl"),
                t if t.starts_with("i686") || t.starts_with("i386") => Some("x86-linux-musl"),
                _ => None,
            };
            if let Some(dir) = arch_dir {
                includes.push(format!("-I{}", libc_include.join(dir).display()));
            }

            // Then generic musl headers
            includes.push(format!("-I{}", libc_include.join("generic-musl").display()));
        } else {
            // glibc (default for linux-gnu targets)
            // Architecture-specific glibc headers FIRST
            let arch_dir = match rust_target {
                t if t.starts_with("aarch64") => Some("aarch64-linux-gnu"),
                t if t.starts_with("x86_64") => Some("x86_64-linux-gnu"),
                t if t.starts_with("arm") && t.contains("gnueabihf") => Some("arm-linux-gnueabihf"),
                t if t.starts_with("arm") => Some("arm-linux-gnueabi"),
                t if t.starts_with("i686") || t.starts_with("i386") => Some("x86-linux-gnu"),
                _ => None,
            };
            if let Some(dir) = arch_dir {
                includes.push(format!("-I{}", libc_include.join(dir).display()));
            }

            // Then generic glibc headers
            includes.push(format!(
                "-I{}",
                libc_include.join("generic-glibc").display()
            ));
        }

        // Architecture-specific Linux-any headers
        let arch_any_dir = match rust_target {
            t if t.starts_with("aarch64") => Some("aarch64-linux-any"),
            t if t.starts_with("x86_64") => Some("x86_64-linux-any"),
            t if t.starts_with("arm") => Some("arm-linux-any"),
            t if t.starts_with("i686") || t.starts_with("i386") => Some("x86-linux-any"),
            _ => None,
        };
        if let Some(dir) = arch_any_dir {
            let path = libc_include.join(dir);
            if path.exists() {
                includes.push(format!("-I{}", path.display()));
            }
        }

        // Common Linux headers (last, so arch-specific overrides)
        includes.push(format!("-I{}", libc_include.join("any-linux-any").display()));
    } else if rust_target.contains("windows") {
        // Windows/mingw headers
        includes.push(format!(
            "-I{}",
            libc_include.join("any-windows-any").display()
        ));
    } else if rust_target.contains("darwin") || rust_target.contains("apple") {
        // macOS headers
        includes.push(format!(
            "-I{}",
            libc_include.join("any-macos-any").display()
        ));
    }

    includes
}

/// Generate bindings for all discovered Ruby installations.
pub fn generate_all_bindings(
    cache_dir: &Path,
    output_dir: &Path,
    config: &Config,
) -> Result<()> {
    let rubies = discover_rubies(cache_dir)?;

    if rubies.is_empty() {
        anyhow::bail!("No Ruby installations found in cache directory");
    }

    info!(
        "Discovered {} platforms with Ruby installations",
        rubies.len()
    );

    // Find Zig installation for libc headers
    let zig_path = find_zig_installation(cache_dir)?;
    info!(zig_path = %zig_path.display(), "Using Zig libc headers");

    // Build a map of ruby_platform -> rust_target from config
    let platform_to_target: HashMap<String, String> = config
        .toolchains
        .iter()
        .map(|tc| (tc.ruby_platform.clone(), tc.rust_target.clone()))
        .collect();

    let mut total_generated = 0;
    let mut errors = Vec::new();

    for (platform, ruby_infos) in &rubies {
        let rust_target = platform_to_target
            .get(platform)
            .cloned()
            .unwrap_or_else(|| {
                // Fallback: try to infer rust target from platform name
                infer_rust_target(platform)
            });

        info!(
            platform = %platform,
            rust_target = %rust_target,
            count = ruby_infos.len(),
            "Generating bindings for platform"
        );

        for ruby_info in ruby_infos {
            match generate_bindings_for_ruby(
                platform,
                &rust_target,
                ruby_info,
                output_dir,
                &zig_path,
            ) {
                Ok(_) => {
                    debug!(
                        platform = %platform,
                        version = %ruby_info.version,
                        "Generated bindings"
                    );
                    total_generated += 1;
                }
                Err(e) => {
                    errors.push(format!("{platform} ruby-{}: {e:#}", ruby_info.version));
                }
            }
        }
    }

    // Abort if any failures occurred
    if !errors.is_empty() {
        anyhow::bail!(
            "Failed to generate {} bindings:\n  - {}",
            errors.len(),
            errors.join("\n  - ")
        );
    }

    info!("Generated bindings for {total_generated} Ruby installations");

    Ok(())
}

/// Infer rust target from ruby platform name.
fn infer_rust_target(platform: &str) -> String {
    match platform {
        "aarch64-linux" => "aarch64-unknown-linux-gnu".to_string(),
        "arm-linux" => "arm-unknown-linux-gnueabihf".to_string(),
        "x86_64-linux" => "x86_64-unknown-linux-gnu".to_string(),
        "x86_64-linux-musl" => "x86_64-unknown-linux-musl".to_string(),
        "aarch64-linux-musl" => "aarch64-unknown-linux-musl".to_string(),
        "arm64-darwin" => "aarch64-apple-darwin".to_string(),
        "x86_64-darwin" => "x86_64-apple-darwin".to_string(),
        "x64-mingw-ucrt" => "x86_64-pc-windows-gnu".to_string(),
        "x64-mingw32" => "x86_64-pc-windows-gnu".to_string(),
        "aarch64-mingw-ucrt" => "aarch64-pc-windows-gnullvm".to_string(),
        _ => platform.to_string(), // Fallback to platform name
    }
}

/// Generate bindings for a single Ruby installation.
fn generate_bindings_for_ruby(
    platform: &str,
    rust_target: &str,
    ruby_info: &RubyInfo,
    output_dir: &Path,
    zig_path: &Path,
) -> Result<()> {
    // Create output directory: {output_dir}/{platform}/{version}/
    let binding_output_dir = output_dir.join(platform).join(&ruby_info.version);
    fs::create_dir_all(&binding_output_dir).with_context(|| {
        format!(
            "Failed to create bindings output directory: {}",
            binding_output_dir.display()
        )
    })?;

    // Load RbConfig from the rbconfig.json
    let rbconfig = RbConfig::from_json(&ruby_info.rbconfig_json).map_err(|e| {
        anyhow::anyhow!(
            "Failed to load RbConfig from {}: {e}",
            ruby_info.rbconfig_json.display()
        )
    })?;

    // Create a temporary directory for bindgen output
    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;

    // Create cfg capture file
    let cfg_path = binding_output_dir.join("bindings.cfg");
    let mut cfg_file = File::create(&cfg_path)
        .with_context(|| format!("Failed to create cfg file: {}", cfg_path.display()))?;

    // Set up environment variables that bindgen/rb-sys-build expects
    // These mimic what cargo sets during a build
    std::env::set_var("OUT_DIR", temp_dir.path());
    std::env::set_var(
        "CARGO_CFG_TARGET_OS",
        target_os_from_rust_target(rust_target),
    );
    std::env::set_var(
        "CARGO_CFG_TARGET_ARCH",
        target_arch_from_rust_target(rust_target),
    );
    std::env::set_var("TARGET", rust_target);

    // Normalize target triple for clang
    // clang doesn't understand "gnullvm" suffix, use "gnu" instead
    let clang_target = normalize_target_for_clang(rust_target);

    // Build extra clang args for cross-compilation
    let mut extra_clang_args = vec![format!("--target={clang_target}")];

    // Add Zig libc include paths
    let zig_includes = get_zig_libc_includes(zig_path, rust_target);
    extra_clang_args.extend(zig_includes);

    // Don't search system include paths (we're cross-compiling)
    // -nostdinc disables both system headers AND clang builtins
    // -nobuiltininc would only disable clang builtins
    // We want to disable system headers but keep clang builtins (stdarg.h, stddef.h, etc.)
    extra_clang_args.push("-nostdinc".to_string());

    // Re-add clang's builtin include path (stdarg.h, stddef.h, stdalign.h, etc.)
    // These headers are provided by clang itself and are target-independent
    if let Some(resource_dir) = get_clang_resource_dir() {
        let builtin_include = resource_dir.join("include");
        if builtin_include.exists() {
            extra_clang_args.push(format!("-I{}", builtin_include.display()));
        }
    }

    debug!(
        platform = %platform,
        version = %ruby_info.version,
        clang_args = ?extra_clang_args,
        "Bindgen clang args"
    );

    // Set BINDGEN_EXTRA_CLANG_ARGS for rb-sys-build to pick up
    std::env::set_var("BINDGEN_EXTRA_CLANG_ARGS", extra_clang_args.join(" "));

    // Generate bindings using rb-sys-build
    let bindings_rs_path = rb_sys_build::bindings::generate(&rbconfig, false, &mut cfg_file)
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to generate bindings for {platform} ruby-{}: {e}",
                ruby_info.version
            )
        })?;

    // Copy generated bindings to our output directory
    let final_bindings_path = binding_output_dir.join("bindings.rs");
    fs::copy(&bindings_rs_path, &final_bindings_path).with_context(|| {
        format!(
            "Failed to copy bindings from {} to {}",
            bindings_rs_path.display(),
            final_bindings_path.display()
        )
    })?;

    debug!(
        bindings = %final_bindings_path.display(),
        cfg = %cfg_path.display(),
        "Generated bindings files"
    );

    Ok(())
}

/// Get clang's resource directory (contains builtin headers like stdarg.h).
///
/// This runs `clang -print-resource-dir` to find where clang stores its
/// target-independent builtin headers.
fn get_clang_resource_dir() -> Option<PathBuf> {
    let output = std::process::Command::new("clang")
        .arg("-print-resource-dir")
        .output()
        .ok()?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout);
        let path = path.trim();
        if !path.is_empty() {
            return Some(PathBuf::from(path));
        }
    }

    None
}

/// Normalize Rust target triple for clang compatibility.
///
/// clang doesn't understand some Rust-specific target suffixes.
fn normalize_target_for_clang(rust_target: &str) -> String {
    // clang doesn't understand "gnullvm", use "gnu" instead
    if rust_target.contains("gnullvm") {
        return rust_target.replace("gnullvm", "gnu");
    }

    rust_target.to_string()
}

/// Extract target OS from rust target triple.
fn target_os_from_rust_target(target: &str) -> &'static str {
    if target.contains("linux") {
        "linux"
    } else if target.contains("darwin") || target.contains("apple") {
        "macos"
    } else if target.contains("windows") {
        "windows"
    } else {
        "unknown"
    }
}

/// Extract target arch from rust target triple.
fn target_arch_from_rust_target(target: &str) -> &'static str {
    if target.starts_with("aarch64") || target.starts_with("arm64") {
        "aarch64"
    } else if target.starts_with("x86_64") {
        "x86_64"
    } else if target.starts_with("arm") {
        "arm"
    } else if target.starts_with("i686") || target.starts_with("i386") || target.starts_with("x86-")
    {
        "x86"
    } else {
        "unknown"
    }
}
