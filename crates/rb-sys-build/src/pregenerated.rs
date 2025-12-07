//! Support for pre-generated bindings.
//!
//! This module provides functionality to use pre-generated Ruby bindings
//! instead of running bindgen at build time. This eliminates the need for
//! libclang on end-user systems.
//!
//! # Binding Sources (in priority order)
//!
//! 1. **Environment variable**: `RB_SYS_PREGENERATED_BINDINGS_PATH` - explicit path
//! 2. **Embedded bindings**: Built into the crate for common cross-compilation targets
//!
//! # Environment Variables
//!
//! - `RB_SYS_PREGENERATED_BINDINGS_PATH`: Path to the pre-generated bindings.rs file
//! - `RB_SYS_PREGENERATED_CFG_PATH`: Path to the cfg metadata file (contains cargo: directives)
//! - `RB_SYS_FORCE_BINDGEN`: If set, always use bindgen (skip pre-generated bindings)
//!
//! # Embedded Bindings
//!
//! The crate ships with pre-generated bindings for common cross-compilation targets.
//! These are stored in a compressed tarball and extracted on-demand during build.
//!
//! Supported platforms:
//! - aarch64-linux (Ruby 2.7-3.4)
//! - arm-linux (Ruby 2.7-3.4)
//! - x64-mingw-ucrt (Ruby 3.1-3.4)
//! - x64-mingw32 (Ruby 2.7-3.0)
//! - aarch64-mingw-ucrt (Ruby 3.4)
//!
//! # Feature-aware Filtering
//!
//! Pre-generated bindings are created with all items included (rbimpls, deprecated-types).
//! At consume time, items are filtered out based on the crate's enabled features:
//!
//! - If `bindgen-rbimpls` is NOT enabled: items matching `^rbimpl_.*` and `^RBIMPL_.*` are removed
//! - If `bindgen-deprecated-types` is NOT enabled: items matching `^_bindgen_ty_9.*` are removed

use crate::{debug_log, RbConfig};
use quote::ToTokens;
use regex::Regex;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::{env, error::Error};

/// Embedded bindings tarball (zstd-compressed tar archive).
///
/// This contains pre-generated bindings for cross-compilation targets.
/// Structure: bindings/{platform}/{version}/bindings.rs
///           bindings/{platform}/{version}/bindings.cfg
static EMBEDDED_BINDINGS: &[u8] = include_bytes!("../data/assets.tar.zst");

/// Cache for extracted embedded bindings index.
static EMBEDDED_INDEX: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();

/// Check if pre-generated bindings should be used via environment variable.
///
/// Returns `Some(path)` if RB_SYS_PREGENERATED_BINDINGS_PATH is set,
/// `None` otherwise.
pub fn pregenerated_bindings_path() -> Option<PathBuf> {
    env::var("RB_SYS_PREGENERATED_BINDINGS_PATH")
        .ok()
        .map(PathBuf::from)
        .filter(|p| p.exists())
}

/// Check if we should use pre-generated bindings.
pub fn use_pregenerated() -> bool {
    // Check if force-bindgen is set (developer escape hatch)
    if env::var("RB_SYS_FORCE_BINDGEN").is_ok() {
        debug_log!("INFO: RB_SYS_FORCE_BINDGEN is set, ignoring pre-generated bindings");
        return false;
    }

    pregenerated_bindings_path().is_some()
}

/// Check if embedded bindings are available for the given rbconfig.
///
/// This checks if we have pre-generated bindings for the target platform and Ruby version.
pub fn has_embedded_bindings(rbconfig: &RbConfig) -> bool {
    // Don't use embedded bindings if force-bindgen is set
    if env::var("RB_SYS_FORCE_BINDGEN").is_ok() {
        return false;
    }

    let platform = normalize_platform(&rbconfig.platform());
    let version = match ruby_version_string(rbconfig) {
        Some(v) => v,
        None => return false,
    };

    let index = get_embedded_index();
    if let Some(versions) = index.get(&platform) {
        versions.iter().any(|v| v == &version)
    } else {
        false
    }
}

/// Load and process pre-generated bindings from environment variable path.
///
/// This function:
/// 1. Reads the pre-generated bindings file
/// 2. Parses it into a syn::File
/// 3. Applies feature-aware filtering
/// 4. Applies sanitizer transforms (same as normal bindgen flow)
/// 5. Emits cfg metadata from the sidecar file
/// 6. Writes the processed bindings to OUT_DIR
pub fn load_pregenerated(
    rbconfig: &RbConfig,
    cfg_out: &mut File,
) -> Result<PathBuf, Box<dyn Error>> {
    let bindings_path = pregenerated_bindings_path()
        .ok_or("RB_SYS_PREGENERATED_BINDINGS_PATH not set or file not found")?;

    debug_log!(
        "INFO: Using pre-generated bindings from {}",
        bindings_path.display()
    );

    // Read the pre-generated bindings
    let bindings_content = fs::read_to_string(&bindings_path)?;

    // Process and write bindings
    process_bindings_content(rbconfig, &bindings_content, cfg_out, None)
}

/// Load embedded bindings for the target platform and Ruby version.
///
/// This extracts bindings from the embedded tarball and processes them.
pub fn load_embedded(rbconfig: &RbConfig, cfg_out: &mut File) -> Result<PathBuf, Box<dyn Error>> {
    let platform = normalize_platform(&rbconfig.platform());
    let version = ruby_version_string(rbconfig)
        .ok_or_else(|| "Could not determine Ruby version from rbconfig".to_string())?;

    debug_log!(
        "INFO: Loading embedded bindings for platform={}, version={}",
        platform,
        version
    );

    // Extract the bindings and cfg content from the tarball
    let (bindings_content, cfg_content) = extract_embedded_bindings(&platform, &version)?;

    // Process and write bindings
    process_bindings_content(rbconfig, &bindings_content, cfg_out, Some(&cfg_content))
}

/// Process bindings content and write to OUT_DIR.
fn process_bindings_content(
    rbconfig: &RbConfig,
    bindings_content: &str,
    cfg_out: &mut File,
    embedded_cfg: Option<&str>,
) -> Result<PathBuf, Box<dyn Error>> {
    // Parse into syn::File
    let mut tokens: syn::File = syn::parse_file(bindings_content)?;

    // Apply feature-aware filtering
    filter_bindings_for_features(&mut tokens);

    // Apply the same sanitizer transforms as the normal bindgen flow
    crate::bindings::sanitizer::ensure_backwards_compatible_encoding_pointers(&mut tokens);

    // Emit cfg metadata
    if let Some(cfg) = embedded_cfg {
        emit_cfg_from_content(cfg, cfg_out)?;
    } else {
        emit_cfg_from_sidecar(cfg_out)?;
    }

    // Also extract cfg from the bindings themselves (like the normal flow)
    push_cargo_cfg_from_bindings(&tokens, cfg_out)?;

    // Apply stable_api categorization
    crate::bindings::stable_api::categorize_bindings(&mut tokens);

    // Write to OUT_DIR
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let slug = rbconfig.ruby_version_slug();
    let crate_version = env!("CARGO_PKG_VERSION");
    let out_path = out_dir.join(format!("bindings-{crate_version}-{slug}.rs"));

    let code = tokens.into_token_stream().to_string();
    let mut out_file = File::create(&out_path)?;
    out_file.write_all(code.as_bytes())?;

    // Try to run rustfmt
    run_rustfmt(&out_path);

    debug_log!(
        "INFO: Wrote pre-generated bindings to {}",
        out_path.display()
    );

    Ok(out_path)
}

/// Get the index of embedded bindings.
///
/// Returns a map of platform -> [versions].
fn get_embedded_index() -> &'static HashMap<String, Vec<String>> {
    EMBEDDED_INDEX.get_or_init(build_embedded_index)
}

/// Build an index of what's in the embedded tarball.
fn build_embedded_index() -> HashMap<String, Vec<String>> {
    let mut index: HashMap<String, Vec<String>> = HashMap::new();

    // Decompress and read the tarball to index its contents
    let decoder = match zstd::Decoder::new(Cursor::new(EMBEDDED_BINDINGS)) {
        Ok(d) => d,
        Err(e) => {
            debug_log!("WARN: Failed to decompress embedded bindings: {}", e);
            return index;
        }
    };

    let mut archive = tar::Archive::new(decoder);

    if let Ok(entries) = archive.entries() {
        for entry in entries.flatten() {
            if let Ok(path) = entry.path() {
                let path_str = path.to_string_lossy();
                // Path format: bindings/{platform}/{version}/bindings.rs
                let parts: Vec<&str> = path_str.split('/').collect();
                if parts.len() >= 4 && parts[0] == "bindings" && parts[3] == "bindings.rs" {
                    let platform = parts[1].to_string();
                    let version = parts[2].to_string();
                    index.entry(platform).or_default().push(version);
                }
            }
        }
    }

    index
}

/// Extract bindings for a specific platform and version from the embedded tarball.
fn extract_embedded_bindings(
    platform: &str,
    version: &str,
) -> Result<(String, String), Box<dyn Error>> {
    let bindings_path = format!("bindings/{platform}/{version}/bindings.rs");
    let cfg_path = format!("bindings/{platform}/{version}/bindings.cfg");

    let mut bindings_content = None;
    let mut cfg_content = None;

    // Decompress and search the tarball
    let decoder = zstd::Decoder::new(Cursor::new(EMBEDDED_BINDINGS))?;
    let mut archive = tar::Archive::new(decoder);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let entry_path = entry.path()?.to_string_lossy().to_string();

        if entry_path == bindings_path {
            let mut content = String::new();
            entry.read_to_string(&mut content)?;
            bindings_content = Some(content);
        } else if entry_path == cfg_path {
            let mut content = String::new();
            entry.read_to_string(&mut content)?;
            cfg_content = Some(content);
        }

        // Early exit if we found both
        if bindings_content.is_some() && cfg_content.is_some() {
            break;
        }
    }

    let bindings = bindings_content.ok_or_else(|| {
        format!(
            "Embedded bindings not found for platform={platform}, version={version}"
        )
    })?;

    let cfg = cfg_content.unwrap_or_default();

    Ok((bindings, cfg))
}

/// Normalize Ruby platform to match our embedded bindings directory names.
///
/// Ruby's platform() returns values like "aarch64-linux-gnu" or "x64-mingw-ucrt",
/// but our embedded bindings use simplified names like "aarch64-linux".
fn normalize_platform(platform: &str) -> String {
    // Map Ruby platform names to our directory names
    match platform {
        // Linux variants
        p if p.starts_with("aarch64-linux") => "aarch64-linux".to_string(),
        p if p.starts_with("arm-linux") || p.starts_with("arm-linux-gnueabihf") => {
            "arm-linux".to_string()
        }
        p if p.starts_with("x86_64-linux") => "x86_64-linux".to_string(),
        p if p.starts_with("i686-linux") || p.starts_with("i386-linux") => "x86-linux".to_string(),

        // musl variants
        p if p.contains("aarch64") && p.contains("musl") => "aarch64-linux-musl".to_string(),
        p if p.contains("x86_64") && p.contains("musl") => "x86_64-linux-musl".to_string(),

        // Windows variants (keep as-is, they already match)
        "x64-mingw-ucrt" => "x64-mingw-ucrt".to_string(),
        "x64-mingw32" => "x64-mingw32".to_string(),
        "aarch64-mingw-ucrt" => "aarch64-mingw-ucrt".to_string(),

        // Darwin variants
        p if p.contains("arm64-darwin") || p.contains("aarch64-darwin") => {
            "arm64-darwin".to_string()
        }
        p if p.contains("x86_64-darwin") => "x86_64-darwin".to_string(),

        // Fallback to original
        _ => platform.to_string(),
    }
}

/// Get Ruby version string in X.Y.Z format.
fn ruby_version_string(rbconfig: &RbConfig) -> Option<String> {
    // Try RUBY_PROGRAM_VERSION first
    if let Some(ver) = rbconfig.get("RUBY_PROGRAM_VERSION") {
        return Some(ver);
    }

    // Fall back to MAJOR.MINOR.TEENY
    let major = rbconfig.get("MAJOR")?;
    let minor = rbconfig.get("MINOR")?;
    let teeny = rbconfig.get("TEENY").unwrap_or_else(|| "0".to_string());

    Some(format!("{major}.{minor}.{teeny}"))
}

/// Filter out items based on enabled features.
///
/// Pre-generated bindings include all items. We remove items based on features:
/// - Without `bindgen-rbimpls`: remove `^rbimpl_.*` and `^RBIMPL_.*`
/// - Without `bindgen-deprecated-types`: remove `^_bindgen_ty_9.*`
fn filter_bindings_for_features(tokens: &mut syn::File) {
    let filter_rbimpls = !cfg!(feature = "bindgen-rbimpls");
    let filter_deprecated = !cfg!(feature = "bindgen-deprecated-types");

    if !filter_rbimpls && !filter_deprecated {
        return; // Nothing to filter
    }

    let rbimpl_re = Regex::new(r"^(rbimpl_|RBIMPL_)").unwrap();
    let deprecated_re = Regex::new(r"^_bindgen_ty_9").unwrap();

    tokens.items.retain(|item| {
        let ident = match item {
            syn::Item::Fn(f) => Some(f.sig.ident.to_string()),
            syn::Item::Const(c) => Some(c.ident.to_string()),
            syn::Item::Static(s) => Some(s.ident.to_string()),
            syn::Item::Type(t) => Some(t.ident.to_string()),
            syn::Item::Struct(s) => Some(s.ident.to_string()),
            syn::Item::Enum(e) => Some(e.ident.to_string()),
            syn::Item::Union(u) => Some(u.ident.to_string()),
            syn::Item::Mod(m) => m.ident.to_string().into(),
            _ => None,
        };

        if let Some(name) = ident {
            if filter_rbimpls && rbimpl_re.is_match(&name) {
                debug_log!("INFO: Filtering out rbimpl item: {}", name);
                return false;
            }
            if filter_deprecated && deprecated_re.is_match(&name) {
                debug_log!("INFO: Filtering out deprecated type: {}", name);
                return false;
            }
        }

        true
    });
}

/// Emit cfg metadata from the sidecar file (environment variable path).
fn emit_cfg_from_sidecar(cfg_out: &mut File) -> Result<(), Box<dyn Error>> {
    let cfg_path = env::var("RB_SYS_PREGENERATED_CFG_PATH").ok();

    if let Some(path) = cfg_path {
        let path = PathBuf::from(path);
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            emit_cfg_from_content(&content, cfg_out)?;
        }
    }

    Ok(())
}

/// Emit cfg metadata from content string.
fn emit_cfg_from_content(content: &str, cfg_out: &mut File) -> Result<(), Box<dyn Error>> {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Echo to stdout for cargo
        println!("{line}");

        // Also write to cfg_out file
        if line.starts_with("cargo:defines_") || line.starts_with("cargo:ruby_") {
            writeln!(cfg_out, "{line}")?;
        }
    }

    Ok(())
}

/// Extract and emit cfg metadata from bindings constants.
/// This mirrors the logic in bindings.rs.
fn push_cargo_cfg_from_bindings(
    syntax: &syn::File,
    cfg_out: &mut File,
) -> Result<(), Box<dyn Error>> {
    use quote::ToTokens;
    use syn::{Expr, ExprLit, Lit};

    fn is_defines(line: &str) -> bool {
        line.starts_with("HAVE_RUBY")
            || line.starts_with("HAVE_RB")
            || line.starts_with("USE")
            || line.starts_with("RUBY_DEBUG")
            || line.starts_with("RUBY_NDEBUG")
    }

    // Helper to find constants in the uncategorized module
    fn find_consts_in_module(items: &[syn::Item]) -> Vec<&syn::ItemConst> {
        let mut consts = Vec::new();

        for item in items {
            match item {
                syn::Item::Const(c) => consts.push(c),
                syn::Item::Mod(m) => {
                    // Look inside modules (especially "uncategorized")
                    if let Some((_, ref items)) = m.content {
                        consts.extend(find_consts_in_module(items));
                    }
                }
                _ => {}
            }
        }

        consts
    }

    let consts = find_consts_in_module(&syntax.items);

    for item in consts {
        let conf_name = item.ident.to_string();

        if is_defines(&conf_name) {
            let name = conf_name.to_lowercase();
            let val = match &*item.expr {
                Expr::Lit(ExprLit {
                    lit: Lit::Int(ref lit),
                    ..
                }) => (lit.base10_parse::<u8>().unwrap_or(1) != 0).to_string(),
                Expr::Lit(ExprLit {
                    lit: Lit::Bool(ref lit),
                    ..
                }) => lit.value.to_string(),
                _ => "true".to_string(),
            };

            println!(r#"cargo:rustc-check-cfg=cfg(ruby_{name}, values("true", "false"))"#);
            println!("cargo:rustc-cfg=ruby_{name}=\"{val}\"");
            println!("cargo:defines_{name}={val}");
            writeln!(cfg_out, "cargo:defines_{name}={val}")?;
        }

        if conf_name.starts_with("RUBY_ABI_VERSION") {
            let val = item.expr.to_token_stream().to_string();
            println!("cargo:ruby_abi_version={val}");
            writeln!(cfg_out, "cargo:ruby_abi_version={val}")?;
        }
    }

    Ok(())
}

fn run_rustfmt(path: &Path) {
    let mut cmd = std::process::Command::new("rustfmt");
    cmd.stderr(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::null());
    cmd.arg(path);
    let _ = cmd.status();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_platform() {
        assert_eq!(normalize_platform("aarch64-linux-gnu"), "aarch64-linux");
        assert_eq!(normalize_platform("arm-linux-gnueabihf"), "arm-linux");
        assert_eq!(normalize_platform("x64-mingw-ucrt"), "x64-mingw-ucrt");
        assert_eq!(normalize_platform("x64-mingw32"), "x64-mingw32");
    }

    #[test]
    fn test_embedded_index_is_populated() {
        let index = get_embedded_index();
        // We should have at least some platforms
        assert!(!index.is_empty(), "Embedded index should not be empty");

        // Check for expected platforms
        assert!(
            index.contains_key("aarch64-linux"),
            "Should have aarch64-linux"
        );
        assert!(index.contains_key("arm-linux"), "Should have arm-linux");
    }

    #[test]
    fn test_extract_embedded_bindings() {
        // Try to extract a known binding
        let index = get_embedded_index();

        if let Some(versions) = index.get("aarch64-linux") {
            if let Some(version) = versions.first() {
                let result = extract_embedded_bindings("aarch64-linux", version);
                assert!(result.is_ok(), "Should extract bindings successfully");

                let (bindings, cfg) = result.unwrap();
                assert!(!bindings.is_empty(), "Bindings should not be empty");
                assert!(!cfg.is_empty(), "Cfg should not be empty");
            }
        }
    }
}
