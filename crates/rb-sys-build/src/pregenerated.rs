//! Support for pre-generated bindings.
//!
//! This module provides functionality to use pre-generated Ruby bindings
//! instead of running bindgen at build time. This eliminates the need for
//! libclang on end-user systems.
//!
//! # Environment Variables
//!
//! - `RB_SYS_PREGENERATED_BINDINGS_PATH`: Path to the pre-generated bindings.rs file
//! - `RB_SYS_PREGENERATED_CFG_PATH`: Path to the cfg metadata file (contains cargo: directives)
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
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::{env, error::Error};

/// Check if pre-generated bindings should be used.
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

/// Load and process pre-generated bindings.
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

    // Parse into syn::File
    let mut tokens: syn::File = syn::parse_file(&bindings_content)?;

    // Apply feature-aware filtering
    filter_bindings_for_features(&mut tokens);

    // Apply the same sanitizer transforms as the normal bindgen flow
    // Import these from the bindings module
    crate::bindings::sanitizer::ensure_backwards_compatible_encoding_pointers(&mut tokens);

    // Note: We skip MSVC qualifiers since pre-generated bindings are target-specific
    // and the MSVC case should have its own pre-generated set

    // Emit cfg metadata from sidecar file
    emit_cfg_from_sidecar(cfg_out)?;

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

    debug_log!("INFO: Wrote pre-generated bindings to {}", out_path.display());

    Ok(out_path)
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

/// Emit cfg metadata from the sidecar file.
///
/// The sidecar file contains lines like:
/// ```text
/// cargo:rustc-cfg=ruby_have_ruby_encoding_h="true"
/// cargo:defines_have_ruby_encoding_h=true
/// ```
fn emit_cfg_from_sidecar(cfg_out: &mut File) -> Result<(), Box<dyn Error>> {
    let cfg_path = env::var("RB_SYS_PREGENERATED_CFG_PATH").ok();

    if let Some(path) = cfg_path {
        let path = PathBuf::from(path);
        if path.exists() {
            debug_log!("INFO: Reading cfg sidecar from {}", path.display());
            let file = File::open(&path)?;
            let reader = BufReader::new(file);

            for line in reader.lines() {
                let line = line?;
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

    for item in syntax.items.iter() {
        if let syn::Item::Const(item) = item {
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

                println!(
                    r#"cargo:rustc-check-cfg=cfg(ruby_{name}, values("true", "false"))"#
                );
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
