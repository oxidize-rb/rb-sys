mod sanitizer;
mod stable_api;

use crate::cc::Build;
use crate::utils::is_msvc;
use crate::{debug_log, RbConfig};
use quote::ToTokens;
use stable_api::{categorize_bindings, opaqueify_bindings};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, error::Error};
use syn::{Expr, ExprLit, ItemConst, Lit};

const WRAPPER_H_CONTENT: &str = include_str!("bindings/wrapper.h");

/// Generate bindings for the Ruby using bindgen.
pub fn generate(
    rbconfig: &RbConfig,
    static_ruby: bool,
    cfg_out: &mut File,
) -> Result<PathBuf, Box<dyn Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    let mut clang_args = vec![];
    if let Some(ruby_include_dir) = rbconfig.get("rubyhdrdir") {
        clang_args.push(format!("-I{}", ruby_include_dir));
    }
    if let Some(ruby_arch_include_dir) = rbconfig.get("rubyarchhdrdir") {
        clang_args.push(format!("-I{}", ruby_arch_include_dir));
    }

    clang_args.extend(Build::default_cflags());
    clang_args.extend(rbconfig.cflags.clone());
    clang_args.extend(rbconfig.cppflags());

    debug_log!("INFO: using bindgen with clang args: {:?}", clang_args);

    let mut wrapper_h = WRAPPER_H_CONTENT.to_string();

    if !is_msvc() {
        wrapper_h.push_str("#ifdef HAVE_RUBY_ATOMIC_H\n");
        wrapper_h.push_str("#include \"ruby/atomic.h\"\n");
        wrapper_h.push_str("#endif\n");
    }

    if rbconfig.have_ruby_header("ruby/io/buffer.h") {
        clang_args.push("-DHAVE_RUBY_IO_BUFFER_H".to_string());
    }

    let bindings = default_bindgen(clang_args)
        .allowlist_file(".*ruby.*")
        .blocklist_item("ruby_abi_version")
        .blocklist_function("rb_tr_abi_version")
        .blocklist_function("^__.*")
        .blocklist_item("RData")
        .blocklist_function("rb_tr_rdata")
        .blocklist_function("rb_tr_rtypeddata")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    let bindings = if cfg!(feature = "bindgen-rbimpls") {
        bindings
    } else {
        bindings
            .blocklist_item("^rbimpl_.*")
            .blocklist_item("^RBIMPL_.*")
    };

    let bindings = if cfg!(feature = "bindgen-deprecated-types") {
        bindings
    } else {
        bindings.blocklist_item("^_bindgen_ty_9.*")
    };

    let bindings = opaqueify_bindings(rbconfig, bindings, &mut wrapper_h);

    let mut tokens = {
        write!(std::io::stderr(), "{}", wrapper_h)?;
        let bindings = bindings.header_contents("wrapper.h", &wrapper_h);
        let code_string = bindings.generate()?.to_string();
        syn::parse_file(&code_string)?
    };

    let slug = rbconfig.ruby_version_slug();
    let crate_version = env!("CARGO_PKG_VERSION");
    let out_path = out_dir.join(format!("bindings-{}-{}.rs", crate_version, slug));

    let code = {
        sanitizer::ensure_backwards_compatible_encoding_pointers(&mut tokens);
        clean_docs(rbconfig, &mut tokens);

        if is_msvc() {
            qualify_symbols_for_msvc(&mut tokens, static_ruby, rbconfig);
        }

        push_cargo_cfg_from_bindings(&tokens, cfg_out)?;
        categorize_bindings(&mut tokens);
        tokens.into_token_stream().to_string()
    };

    let mut out_file = File::create(&out_path)?;
    std::io::Write::write_all(&mut out_file, code.as_bytes())?;
    run_rustfmt(&out_path);

    Ok(out_path)
}

fn run_rustfmt(path: &Path) {
    let mut cmd = std::process::Command::new("rustfmt");
    cmd.stderr(std::process::Stdio::inherit());
    cmd.stdout(std::process::Stdio::inherit());

    cmd.arg(path);

    if let Err(e) = cmd.status() {
        debug_log!("WARN: failed to run rustfmt: {}", e);
    }
}

fn clean_docs(rbconfig: &RbConfig, syntax: &mut syn::File) {
    if rbconfig.is_cross_compiling() {
        return;
    }

    let ver = rbconfig.ruby_version_slug();

    sanitizer::cleanup_docs(syntax, &ver).unwrap_or_else(|e| {
        debug_log!("WARN: failed to clean up docs, skipping: {}", e);
    })
}

fn default_bindgen(clang_args: Vec<String>) -> bindgen::Builder {
    let bindings = bindgen::Builder::default()
        .rustified_enum(".*")
        .no_copy("rb_data_type_struct")
        .derive_eq(true)
        .derive_debug(true)
        .clang_args(clang_args)
        .layout_tests(cfg!(feature = "bindgen-layout-tests"))
        .blocklist_item("^__darwin_pthread.*")
        .blocklist_item("^_opaque_pthread.*")
        .blocklist_item("^pthread_.*")
        .blocklist_item("^rb_native.*")
        .opaque_type("^__sFILE$")
        .merge_extern_blocks(true)
        .generate_comments(true)
        .size_t_is_usize(env::var("CARGO_FEATURE_BINDGEN_SIZE_T_IS_USIZE").is_ok())
        .impl_debug(cfg!(feature = "bindgen-impl-debug"))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    if env::var("CARGO_FEATURE_BINDGEN_ENABLE_FUNCTION_ATTRIBUTE_DETECTION").is_ok() {
        bindings.enable_function_attribute_detection()
    } else {
        bindings
    }
}

// This is needed because bindgen doesn't support the `__declspec(dllimport)` on
// global variables. Without it, symbols are not found.
// See https://stackoverflow.com/a/66182704/2057700
fn qualify_symbols_for_msvc(tokens: &mut syn::File, is_static: bool, rbconfig: &RbConfig) {
    let kind = if is_static { "static" } else { "dylib" };

    let name = if is_static {
        rbconfig.libruby_static_name()
    } else {
        rbconfig.libruby_so_name()
    };

    sanitizer::add_link_ruby_directives(tokens, &name, kind).unwrap_or_else(|e| {
        debug_log!("WARN: failed to add link directives: {}", e);
    });
}

// Add things like `#[cfg(ruby_use_transient_heap = "true")]` to the bindings config
fn push_cargo_cfg_from_bindings(
    syntax: &syn::File,
    cfg_out: &mut File,
) -> Result<(), Box<dyn Error>> {
    fn is_defines(line: &str) -> bool {
        line.starts_with("HAVE_RUBY")
            || line.starts_with("HAVE_RB")
            || line.starts_with("USE")
            || line.starts_with("RUBY_DEBUG")
            || line.starts_with("RUBY_NDEBUG")
    }

    for item in syntax.items.iter() {
        if let syn::Item::Const(item) = item {
            let conf = ConfValue::new(item);
            let conf_name = conf.name();

            if is_defines(&conf_name) {
                let name = conf_name.to_lowercase();
                let val = conf.value_bool().to_string();
                println!(
                    r#"cargo:rustc-check-cfg=cfg(ruby_{}, values("true", "false"))"#,
                    name
                );
                println!("cargo:rustc-cfg=ruby_{}=\"{}\"", name, val);
                println!("cargo:defines_{}={}", name, val);
                writeln!(cfg_out, "cargo:defines_{}={}", name, val)?;
            }

            if conf_name.starts_with("RUBY_ABI_VERSION") {
                println!("cargo:ruby_abi_version={}", conf.value_string());
                writeln!(cfg_out, "cargo:ruby_abi_version={}", conf.value_string())?;
            }
        }
    }

    Ok(())
}

/// An autoconf constant in the bindings
struct ConfValue<'a> {
    item: &'a syn::ItemConst,
}

impl<'a> ConfValue<'a> {
    pub fn new(item: &'a ItemConst) -> Self {
        Self { item }
    }

    pub fn name(&self) -> String {
        self.item.ident.to_string()
    }

    pub fn value_string(&self) -> String {
        match &*self.item.expr {
            Expr::Lit(ExprLit { lit, .. }) => lit.to_token_stream().to_string(),
            _ => panic!(
                "Could not convert HAVE_* constant to string: {:#?}",
                self.item
            ),
        }
    }

    pub fn value_bool(&self) -> bool {
        match &*self.item.expr {
            Expr::Lit(ExprLit {
                lit: Lit::Int(ref lit),
                ..
            }) => lit.base10_parse::<u8>().unwrap() != 0,
            Expr::Lit(ExprLit {
                lit: Lit::Bool(ref lit),
                ..
            }) => lit.value,
            _ => panic!(
                "Could not convert HAVE_* constant to bool: {:#?}",
                self.item
            ),
        }
    }
}
