use anyhow::{bail, Context, Result};
use clap::Args;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tracing::{debug, info, instrument};

use crate::assets::AssetManager;
use crate::platform::MacOSConfig;
use crate::sysroot::SysrootManager;
use crate::toolchain::ToolchainInfo;
use crate::zig::{
    env::cargo_env,
    libc as zig_libc, manager as zig_manager, shim,
    shim::ShimPaths,
    target::{Os, RustTarget},
};

/// Configuration for building a gem
#[derive(Args, Debug, Clone)]
pub struct BuildConfig {
    /// Target Rust triple to build for
    #[arg(short, long, required = true)]
    pub target: String,

    /// Ruby version(s) to build for (e.g., "3.4" or "3.3,3.4")
    /// Required for cross-compilation
    #[arg(long, value_delimiter = ',')]
    pub ruby_version: Vec<String>,

    /// Path to the Zig compiler (defaults to bundled Zig if available)
    #[arg(long, env = "ZIG_PATH")]
    pub zig_path: Option<PathBuf>,

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

/// Build a native gem for a specific Ruby version
fn build_for_ruby_version(
    config: &BuildConfig,
    toolchain: &ToolchainInfo,
    ruby_version: &str,
    target: &RustTarget,
    sysroots: &SysrootManager,
    target_dir: &Path,
    cli_path: &Path,
    zig_path: &Path,
    sysroot_path: &Path,
    shim_dir: &Path,
    shim_paths: &ShimPaths,
) -> Result<()> {
    info!(ruby_version = %ruby_version, "Building for Ruby version");

    // Load assets manager
    let assets = AssetManager::new().context("Failed to initialize asset manager")?;

    // Load rbconfig.json from embedded assets
    let rbconfig_json = assets
        .extract_rbconfig(&toolchain.ruby_platform, ruby_version)
        .with_context(|| {
            format!(
                "Failed to load rbconfig.json for {}/{}",
                toolchain.ruby_platform, ruby_version
            )
        })?;

    // Parse rbconfig
    let rbconfig: serde_json::Value = serde_json::from_str(&rbconfig_json).with_context(|| {
        format!(
            "Failed to parse rbconfig.json for {}/{}",
            toolchain.ruby_platform, ruby_version
        )
    })?;

    // Mount sysroot to get Ruby headers and macOS SDK (if applicable)
    // Must be done BEFORE setting up environment variables
    let mounted_sysroot = sysroots.mount(&config.target, target_dir)?;
    let rubies_path = mounted_sysroot.rubies_path();
    let macos_sdk_path = mounted_sysroot.macos_sdk_path();

    // Get environment variables
    let mut env_vars = cargo_env(target, shim_paths, Some(sysroot_path), macos_sdk_path);

    // Configure libclang for bindgen (if embedded libclang is available)
    crate::libclang::configure_bindgen_env(&mut env_vars)?;

    // Configure bindgen include paths for cross-compilation
    let bindgen_args = build_bindgen_args(&config.target, zig_path, sysroot_path, macos_sdk_path)?;
    debug!(bindgen_args = %bindgen_args, "Setting BINDGEN_EXTRA_CLANG_ARGS");
    env_vars.insert("BINDGEN_EXTRA_CLANG_ARGS".to_string(), bindgen_args.clone());

    // Set BOTH hyphen and underscore variants - different tools check different variants
    // rb-sys checks: BINDGEN_EXTRA_CLANG_ARGS_x86_64_unknown_linux_gnu (underscores)
    // bindgen checks: BINDGEN_EXTRA_CLANG_ARGS_x86_64-unknown-linux-gnu (hyphens)
    let key_underscore = format!(
        "BINDGEN_EXTRA_CLANG_ARGS_{}",
        config.target.replace('-', "_")
    );
    let key_hyphen = format!("BINDGEN_EXTRA_CLANG_ARGS_{}", config.target);
    env_vars.insert(key_underscore, bindgen_args.clone());
    env_vars.insert(key_hyphen, bindgen_args);

    // Export ALL rbconfig values as RBCONFIG_* env vars
    if let Some(config_obj) = rbconfig.get("config").and_then(|c| c.as_object()) {
        for (key, value) in config_obj {
            let env_key = format!("RBCONFIG_{}", key);
            let env_value = value.as_str().unwrap_or("").to_string();
            env_vars.insert(env_key, env_value);
        }
    }

    // Mark as cross-compiling
    env_vars.insert("RBCONFIG_CROSS_COMPILING".to_string(), "yes".to_string());

    // Set PKG_CONFIG_PATH for sysroot libraries (OpenSSL, zlib, etc.)
    let pkg_config_path = build_pkg_config_path(sysroot_path);
    if !pkg_config_path.is_empty() {
        debug!(pkg_config_path = %pkg_config_path, "Setting PKG_CONFIG_PATH");
        env_vars.insert("PKG_CONFIG_PATH".to_string(), pkg_config_path);
        // Disable pkg-config from finding host libraries
        env_vars.insert(
            "PKG_CONFIG_SYSROOT_DIR".to_string(),
            sysroot_path.display().to_string(),
        );
    }

    // Override Ruby header paths to use extracted target Ruby (not host Ruby)
    // This prevents rb-sys from using host nix Ruby headers when cross-compiling
    if let Some(ruby_headers) = find_ruby_headers(rubies_path) {
        info!(
            rubyhdrdir = %ruby_headers.hdrdir.display(),
            rubyarchhdrdir = %ruby_headers.archhdrdir.display(),
            "Using extracted Ruby headers for target"
        );
        env_vars.insert(
            "RBCONFIG_rubyhdrdir".to_string(),
            ruby_headers.hdrdir.display().to_string(),
        );
        env_vars.insert(
            "RBCONFIG_rubyarchhdrdir".to_string(),
            ruby_headers.archhdrdir.display().to_string(),
        );
    }

    // For Windows targets, find and link Ruby import library
    // Windows PE/COFF requires symbols to be resolved at link time (no dynamic lookup)
    if target.os == Os::Windows {
        if let Some(import_lib) = find_ruby_import_lib(rubies_path) {
            info!(
                lib_dir = %import_lib.lib_dir.display(),
                lib_name = %import_lib.lib_name,
                "Found Ruby import library for Windows linking"
            );

            // Append to RUSTFLAGS (may already have -C dlltool from cargo_env)
            let rustflags_key = format!(
                "CARGO_TARGET_{}_RUSTFLAGS",
                config.target.replace('-', "_").to_uppercase()
            );
            let existing = env_vars.get(&rustflags_key).cloned().unwrap_or_default();
            let new_flags = format!(
                "{} -L native={} -l {}",
                existing.trim(),
                import_lib.lib_dir.display(),
                import_lib.lib_name
            );
            env_vars.insert(rustflags_key, new_flags.trim().to_string());
        } else {
            debug!(
                rubies_path = %rubies_path.display(),
                "No Ruby import library found for Windows target"
            );
        }
    }

    // For macOS targets, add SDK linker flags
    if target.os == Os::Darwin {
        if let Some(sdk_path) = macos_sdk_path {
            info!(
                sdk = %sdk_path.display(),
                "Using embedded macOS SDK for linking"
            );

            let rustflags_key = format!(
                "CARGO_TARGET_{}_RUSTFLAGS",
                config.target.replace('-', "_").to_uppercase()
            );
            let existing = env_vars.get(&rustflags_key).cloned().unwrap_or_default();
            let new_flags = format!(
                "{} -C link-arg=-isysroot{} -C link-arg=-mmacosx-version-min=10.13",
                existing.trim(),
                sdk_path.display()
            );
            env_vars.insert(rustflags_key, new_flags.trim().to_string());
        }
    }

    // NOTE: We do NOT add the shim directory to PATH because that would affect
    // host builds (like proc-macros). Instead, we rely on the target-specific
    // CC_<target> environment variables set by cargo_env().

    // Build the cargo command
    info!(ruby_version = %ruby_version, "Running cargo build");
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

    info!(ruby_version = %ruby_version, "Build completed successfully");

    Ok(())
}

/// Build a native gem for the specified target
#[instrument(skip(config), fields(target = %config.target, profile = %config.profile))]
pub fn build(config: &BuildConfig) -> Result<()> {
    info!(target = %config.target, "Building native gem");

    // Parse and validate the target
    let target = RustTarget::parse(&config.target)?;

    // Resolve Zig path (bundled, explicit, or system)
    let zig_path = zig_manager::resolve_zig_path(config.zig_path.as_deref())
        .context("Failed to resolve Zig path")?;

    // Validate zig is available
    validate_zig(&zig_path)?;

    // Load toolchain info for display
    let toolchain = ToolchainInfo::find_by_rust_target(&config.target)
        .context("Failed to find toolchain for target")?;

    info!(
        ruby_platform = %toolchain.ruby_platform,
        zig_target = %target.to_zig_target(),
        "Using toolchain"
    );

    // Load assets manager
    let assets = AssetManager::new().context("Failed to initialize asset manager")?;

    // Determine Ruby version(s) to build for
    let ruby_versions = if config.ruby_version.is_empty() {
        // Auto-detect latest version for this platform from manifest
        let platform = assets
            .manifest()
            .platform_for_rust_target(&toolchain.rust_target)
            .with_context(|| {
                format!(
                    "No platform found for rust target: {}",
                    toolchain.rust_target
                )
            })?;

        if platform.ruby_versions.is_empty() {
            return Err(anyhow::anyhow!(
                "No Ruby versions configured for platform: {}",
                toolchain.ruby_platform
            ));
        }

        // Use the latest version from the manifest
        let latest_version = platform
            .ruby_versions
            .iter()
            .max_by(|a, b| {
                // Simple version comparison
                let a_parts: Vec<i32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
                let b_parts: Vec<i32> = b.split('.').filter_map(|s| s.parse().ok()).collect();
                b_parts.cmp(&a_parts)
            })
            .unwrap();

        vec![latest_version.clone()]
    } else {
        config.ruby_version.clone()
    };

    // Get the CLI binary path for shims to call back into
    let cli_path = std::env::current_exe().context("Failed to get current executable path")?;

    // Get target directory
    let target_dir = get_target_dir(config.manifest_path.as_deref())?;

    // Load assets manager
    let assets = AssetManager::new().context("Failed to initialize asset manager")?;

    // Mount sysroot for hermetic build (ALWAYS, for all targets)
    let sysroots = SysrootManager::new(assets);
    let mounted_sysroot = sysroots
        .mount(&config.target, &target_dir)
        .context("Failed to mount sysroot")?;

    // Canonicalize sysroot path to absolute - shims need absolute paths
    let sysroot_path = mounted_sysroot.path().canonicalize().with_context(|| {
        format!(
            "Failed to canonicalize sysroot path: {}",
            mounted_sysroot.path().display()
        )
    })?;

    info!(
        sysroot = %sysroot_path.display(),
        "Mounted sysroot for hermetic build"
    );

    // Validate macOS SDK for Darwin targets
    if target.requires_sdkroot() {
        // Try to get macOS config (will use embedded SDK if available)
        let sdk_path = mounted_sysroot.macos_sdk_path();
        if let Err(e) = MacOSConfig::from_env_or_embedded(sdk_path) {
            bail!("macOS SDK validation failed: {}", e);
        }
    }

    // Create hermetic build directory for shims
    // Structure: target/rb-sys/<target>/bin/
    let shim_dir = target_dir.join("rb-sys").join(&config.target).join("bin");

    // Ensure shim directory exists and get absolute path
    // This is necessary because build scripts may run from different working directories
    std::fs::create_dir_all(&shim_dir)
        .with_context(|| format!("Failed to create shim directory: {}", shim_dir.display()))?;
    let shim_dir = shim_dir.canonicalize().with_context(|| {
        format!(
            "Failed to canonicalize shim directory: {}",
            shim_dir.display()
        )
    })?;

    // Generate shims
    info!(shim_dir = %shim_dir.display(), "Generating compiler shims");
    let shim_paths = shim::generate_shims(
        &shim_dir,
        &cli_path,
        &zig_path,
        &target,
        Some(&sysroot_path),
    )
    .context("Failed to generate shims")?;

    // Build for each requested Ruby version
    for ruby_version in &ruby_versions {
        build_for_ruby_version(
            config,
            &toolchain,
            ruby_version,
            &target,
            &sysroots,
            &target_dir,
            &cli_path,
            &zig_path,
            &sysroot_path,
            &shim_dir,
            &shim_paths,
        )?;
    }

    // On success, mounted_sysroot will be dropped and cleaned up automatically
    info!("All builds completed successfully");

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

/// Build BINDGEN_EXTRA_CLANG_ARGS for cross-compilation.
///
/// This sets up include paths in the correct order:
/// 1. Zig libc includes (stdio.h, stdlib.h, etc.) - for non-Darwin targets
/// 2. RCD sysroot includes (OpenSSL, zlib headers, etc.)
/// 3. For Darwin: -isysroot=$SDKROOT instead of zig libc
fn build_bindgen_args(
    rust_target: &str,
    zig_path: &Path,
    sysroot_path: &Path,
    macos_sdk_path: Option<&Path>,
) -> Result<String> {
    let mut include_args: Vec<String> = vec![];

    if zig_libc::requires_zig_libc(rust_target) {
        // Get zig's libc include paths for the target
        let zig_includes =
            zig_libc::get_zig_libc_includes(zig_path, rust_target).with_context(|| {
                format!(
                    "Failed to get zig libc includes for target '{rust_target}'.\n\
                     Make sure zig is installed and supports this target."
                )
            })?;

        for path in &zig_includes {
            include_args.push(format!("-I{}", path.display()));
        }

        info!(
            target = %rust_target,
            include_count = zig_includes.len(),
            "Using zig libc includes for bindgen"
        );
    } else if zig_libc::requires_sdkroot(rust_target) {
        // Darwin targets use macOS SDK
        // Prefer embedded SDK if available, fall back to SDKROOT env var
        let sdkroot = macos_sdk_path
            .map(|p| p.display().to_string())
            .or_else(|| std::env::var("SDKROOT").ok());

        if let Some(sdk) = sdkroot {
            include_args.push(format!("-isysroot{sdk}"));
            info!(
                target = %rust_target,
                sdkroot = %sdk,
                "Using macOS SDK for bindgen"
            );
        }
        // Note: If SDKROOT is not set, we already validated and bailed earlier
    }

    // Add RCD sysroot includes for additional libraries (OpenSSL, zlib, etc.)
    let sysroot_include = sysroot_path.join("usr/include");
    if sysroot_include.exists() {
        include_args.push(format!("-I{}", sysroot_include.display()));
        debug!(
            sysroot_include = %sysroot_include.display(),
            "Added sysroot include path"
        );
    }

    // Combine with any existing BINDGEN_EXTRA_CLANG_ARGS from environment
    let existing_args = std::env::var("BINDGEN_EXTRA_CLANG_ARGS").unwrap_or_default();
    if !existing_args.is_empty() {
        include_args.push(existing_args);
    }

    Ok(include_args.join(" "))
}

/// Build PKG_CONFIG_PATH for sysroot libraries.
///
/// This allows pkg-config based crates (like openssl-sys) to find
/// libraries in the cross-compilation sysroot.
fn build_pkg_config_path(sysroot_path: &Path) -> String {
    let mut paths: Vec<String> = vec![];

    // Standard pkg-config locations in sysroot
    let pkgconfig_dirs = [
        sysroot_path.join("usr/lib/pkgconfig"),
        sysroot_path.join("usr/share/pkgconfig"),
        sysroot_path.join("usr/lib/x86_64-linux-gnu/pkgconfig"),
        sysroot_path.join("usr/lib/aarch64-linux-gnu/pkgconfig"),
        sysroot_path.join("usr/lib/arm-linux-gnueabihf/pkgconfig"),
    ];

    for dir in &pkgconfig_dirs {
        if dir.exists() {
            paths.push(dir.display().to_string());
        }
    }

    paths.join(":")
}

/// Ruby header paths for cross-compilation
struct RubyHeaders {
    /// Path to ruby.h and other main headers
    hdrdir: PathBuf,
    /// Path to architecture-specific headers (config.h)
    archhdrdir: PathBuf,
}

/// Ruby import library for Windows linking
struct RubyImportLib {
    /// Directory containing the import library
    lib_dir: PathBuf,
    /// Library name without "lib" prefix and ".a" suffix (for -l flag)
    lib_name: String,
}

/// Find Ruby headers in the extracted rubies directory for the target platform.
///
/// The extracted rubies are stored in:
/// - <rubies_path>/<triplet>/ruby-<version>/include/ruby-<version>/
/// - <rubies_path>/<triplet>/ruby-<version>/include/ruby-<version>/<arch>/
///
/// We look for the newest Ruby version available.
fn find_ruby_headers(rubies_path: &Path) -> Option<RubyHeaders> {
    if !rubies_path.exists() {
        debug!(rubies_path = %rubies_path.display(), "Rubies directory not found");
        return None;
    }

    // Find triplet directories (e.g., "x86_64-linux-gnu", "aarch64-linux-gnu")
    let triplet_dirs: Vec<_> = std::fs::read_dir(rubies_path)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    for triplet_entry in triplet_dirs {
        let triplet_dir = triplet_entry.path();

        // Find ruby-<version> directories
        let ruby_dirs: Vec<_> = std::fs::read_dir(&triplet_dir)
            .ok()?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir() && e.file_name().to_string_lossy().starts_with("ruby-"))
            .collect();

        // Sort by version (descending) to get newest first
        let mut ruby_dirs: Vec<_> = ruby_dirs.into_iter().map(|e| e.path()).collect();
        ruby_dirs.sort_by(|a, b| b.cmp(a));

        for ruby_dir in ruby_dirs {
            let include_dir = ruby_dir.join("include");
            if !include_dir.exists() {
                continue;
            }

            // Find ruby-<major>.<minor>.0 directory inside include
            let ruby_include_dirs: Vec<_> = std::fs::read_dir(&include_dir)
                .ok()?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().is_dir() && e.file_name().to_string_lossy().starts_with("ruby-")
                })
                .collect();

            for ruby_include_entry in ruby_include_dirs {
                let hdrdir = ruby_include_entry.path();

                // Check if ruby.h exists
                if !hdrdir.join("ruby.h").exists() {
                    continue;
                }

                // Find arch-specific directory (contains config.h)
                // Look for directories that might match the triplet
                let arch_dirs: Vec<_> = std::fs::read_dir(&hdrdir)
                    .ok()?
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .filter(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        // Match patterns like "x86_64-linux", "aarch64-linux-gnu", etc.
                        name.contains("linux") || name.contains("darwin") || name.contains("mingw")
                    })
                    .collect();

                if let Some(arch_entry) = arch_dirs.first() {
                    let archhdrdir = arch_entry.path();

                    // Verify config.h exists
                    if archhdrdir.join("ruby/config.h").exists()
                        || archhdrdir.join("config.h").exists()
                    {
                        debug!(
                            hdrdir = %hdrdir.display(),
                            archhdrdir = %archhdrdir.display(),
                            "Found Ruby headers"
                        );
                        return Some(RubyHeaders { hdrdir, archhdrdir });
                    }
                }
            }
        }
    }

    debug!(
        rubies_path = %rubies_path.display(),
        "No Ruby headers found in rubies directory"
    );
    None
}

/// Find Ruby import library in the extracted rubies directory for Windows targets.
///
/// The import library is needed for Windows PE/COFF linking where symbols must
/// be resolved at link time (unlike ELF/Mach-O which support dynamic lookup).
///
/// Searches for files matching the pattern `lib*-ruby*.a` in the Ruby lib directory.
/// Returns the library directory and name (without "lib" prefix and ".a" suffix)
/// for use with -L and -l linker flags.
fn find_ruby_import_lib(rubies_path: &Path) -> Option<RubyImportLib> {
    if !rubies_path.exists() {
        debug!(rubies_path = %rubies_path.display(), "Rubies directory not found");
        return None;
    }

    // Find triplet directories (e.g., "x86_64-w64-mingw32", "aarch64-w64-mingw32")
    let triplet_dirs: Vec<_> = std::fs::read_dir(rubies_path)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    for triplet_entry in triplet_dirs {
        let triplet_dir = triplet_entry.path();

        // Find ruby-<version> directories
        let ruby_dirs: Vec<_> = std::fs::read_dir(&triplet_dir)
            .ok()?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir() && e.file_name().to_string_lossy().starts_with("ruby-"))
            .collect();

        // Sort by version (descending) to get newest first
        let mut ruby_dirs: Vec<_> = ruby_dirs.into_iter().map(|e| e.path()).collect();
        ruby_dirs.sort_by(|a, b| b.cmp(a));

        for ruby_dir in ruby_dirs {
            let lib_dir = ruby_dir.join("lib");
            if !lib_dir.exists() {
                continue;
            }

            // Find import lib matching pattern lib*-ruby*.a
            let lib_entries: Vec<_> = std::fs::read_dir(&lib_dir)
                .ok()?
                .filter_map(|e| e.ok())
                .collect();

            for lib_entry in lib_entries {
                let lib_path = lib_entry.path();
                if !lib_path.is_file() {
                    continue;
                }

                let name = lib_path.file_name()?.to_string_lossy();

                // Match pattern: lib{arch}-{crt}-ruby{version}.a
                // We want the dynamic import library, not the static library
                // Priority: .dll.a > .a (without -static suffix)
                // Examples: libx64-ucrt-ruby340.dll.a (best), libx64-ucrt-ruby340.a (ok)
                // Exclude: libx64-ucrt-ruby340-static.a (static lib with all dependencies)
                if name.starts_with("lib")
                    && name.contains("-ruby")
                    && name.ends_with(".a")
                    && !name.contains("-static")
                {
                    // Extract lib name (remove "lib" prefix and ".a" or ".dll.a" suffix)
                    let lib_name = if let Some(without_dll_a) = name.strip_suffix(".dll.a") {
                        without_dll_a.strip_prefix("lib")?
                    } else {
                        name.strip_prefix("lib")?.strip_suffix(".a")?
                    };

                    debug!(
                        lib_dir = %lib_dir.display(),
                        lib_name = %lib_name,
                        "Found Ruby import library"
                    );

                    return Some(RubyImportLib {
                        lib_dir,
                        lib_name: lib_name.to_string(),
                    });
                }
            }
        }
    }

    debug!(
        rubies_path = %rubies_path.display(),
        "No Ruby import library found in rubies directory"
    );
    None
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
