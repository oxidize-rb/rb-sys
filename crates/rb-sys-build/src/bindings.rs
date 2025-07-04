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

    // On Windows, use a different approach to handle intrinsics issues
    if cfg!(target_os = "windows") {
        debug_log!("INFO: Configuring clang for Windows to handle intrinsics issues");

        // Add MinGW include path for mm_malloc.h and other system headers
        if let Some(mingw_prefix) = rbconfig.get("prefix") {
            // Try common MinGW include paths
            let possible_paths = vec![
                format!("{}/include", mingw_prefix),
                format!("{}/mingw64/include", mingw_prefix),
                format!("{}/ucrt64/include", mingw_prefix),
                format!("{}/msys64/ucrt64/include", mingw_prefix),
            ];

            for path in possible_paths {
                if std::path::Path::new(&path).exists() {
                    clang_args.push(format!("-I{}", path));
                    debug_log!("INFO: Added MinGW include path: {}", path);
                    break;
                }
            }
        }

        // Step 1: Set explicit target triple for Windows GNU toolchain
        clang_args.push("--target=x86_64-pc-windows-gnu".to_string());

        // Step 2: Force basic x86-64 architecture without extensions
        clang_args.push("-march=x86-64".to_string());

        // Step 3: Explicitly disable all AVX512 and AVX10 features
        // Note: We use both -mno- flags and -U macros for maximum compatibility
        let avx_disable_flags = vec![
            "-mno-avx512f",
            "-mno-avx512cd",
            "-mno-avx512er",
            "-mno-avx512pf",
            "-mno-avx512dq",
            "-mno-avx512bw",
            "-mno-avx512vl",
            "-mno-avx512ifma",
            "-mno-avx512vbmi",
            "-mno-avx512vbmi2",
            "-mno-avx512vnni",
            "-mno-avx512bitalg",
            "-mno-avx512vpopcntdq",
            "-mno-avx512fp16",
            "-mno-avx512bf16",
            "-mno-avx512vp2intersect",
            "-mno-amx-tile",
            "-mno-amx-int8",
            "-mno-amx-bf16",
        ];

        for flag in avx_disable_flags {
            clang_args.push(flag.to_string());
        }

        // Step 4: Undefine all feature detection macros
        let undef_macros = vec![
            "-U__AVX512F__",
            "-U__AVX512CD__",
            "-U__AVX512ER__",
            "-U__AVX512PF__",
            "-U__AVX512DQ__",
            "-U__AVX512BW__",
            "-U__AVX512VL__",
            "-U__AVX512IFMA__",
            "-U__AVX512VBMI__",
            "-U__AVX512VBMI2__",
            "-U__AVX512VNNI__",
            "-U__AVX512BITALG__",
            "-U__AVX512VPOPCNTDQ__",
            "-U__AVX512FP16__",
            "-U__AVX512BF16__",
            "-U__AVX512VP2INTERSECT__",
            "-U__AMX_TILE__",
            "-U__AMX_INT8__",
            "-U__AMX_BF16__",
            "-U__AMX_AVX512__",
            "-U__AVX10_1__",
            "-U__AVX10_1_256__",
            "-U__AVX10_1_512__",
            "-U__AVX10_2__",
            "-U__AVX10_2_256__",
            "-U__AVX10_2_512__",
        ];

        for macro_undef in undef_macros {
            clang_args.push(macro_undef.to_string());
        }

        // Step 5: Add compatibility flags
        clang_args.push("-fno-builtin".to_string());
        clang_args.push("-fms-extensions".to_string());
    }

    debug_log!("INFO: using bindgen with clang args: {:?}", clang_args);

    let mut wrapper_h = WRAPPER_H_CONTENT.to_string();

    // Add Windows-specific wrapper to suppress intrinsics
    if cfg!(target_os = "windows") {
        // Include our custom Windows wrapper that defines all header guards
        let windows_wrapper = include_str!("bindings/wrapper_windows.h");
        wrapper_h = windows_wrapper.to_string() + "\n" + &wrapper_h;
    }

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
    let mut bindings = bindgen::Builder::default()
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

    // Comprehensive blocklist for Windows Clang 20 AVX512 intrinsics issues
    if cfg!(target_os = "windows") {
        debug_log!(
            "INFO: Adding Windows-specific blocklist and header guards for AVX512 intrinsics"
        );

        // Add raw lines to define header guards before any includes
        bindings = bindings
            // Define header guards for AVX512 intrinsics headers
            .raw_line("#define _AMXAVX512INTRIN_H")
            .raw_line("#define _AVX10_2CONVERTINTRIN_H")
            .raw_line("#define _AVX512FP16INTRIN_H")
            .raw_line("#define _AVX512VLFP16INTRIN_H")
            .raw_line("#define _IMMINTRIN_H")
            .raw_line("#define _AVX512FINTRIN_H")
            .raw_line("#define _AVX512PFINTRIN_H")
            .raw_line("#define _AVX512VLINTRIN_H")
            .raw_line("#define _AVX512BWINTRIN_H")
            .raw_line("#define _AVX512DQINTRIN_H")
            .raw_line("#define _AVX512CDINTRIN_H")
            .raw_line("#define _AVX512ERINTRIN_H")
            .raw_line("#define _AVX512IFMAINTRIN_H")
            .raw_line("#define _AVX512IFMAVLINTRIN_H")
            .raw_line("#define _AVX512VBMIINTRIN_H")
            .raw_line("#define _AVX512VBMIVLINTRIN_H")
            .raw_line("#define _AVX512VBMI2INTRIN_H")
            .raw_line("#define _AVX512VBMI2VLINTRIN_H")
            .raw_line("#define _AVX512VNNIINTRIN_H")
            .raw_line("#define _AVX512VNNIVLINTRIN_H")
            .raw_line("#define _AVX512VPOPCNTDQINTRIN_H")
            .raw_line("#define _AVX512VPOPCNTDQVLINTRIN_H")
            .raw_line("#define _AVX512BITALGINTRIN_H")
            .raw_line("#define _AVX512BITALG_H")
            .raw_line("#define _AVX512BF16INTRIN_H")
            .raw_line("#define _AVX512BF16VLINTRIN_H")
            .raw_line("#define _AVX512VP2INTERSECTINTRIN_H")
            .raw_line("#define _AVX512VP2INTERSECTVLINTRIN_H")
            // Also prevent AVX10 headers
            .raw_line("#define _AVX10_1_256INTRIN_H")
            .raw_line("#define _AVX10_1_512INTRIN_H")
            .raw_line("#define _AVX10_1INTRIN_H")
            .raw_line("#define _AVX10_2_256INTRIN_H")
            .raw_line("#define _AVX10_2_512INTRIN_H")
            .raw_line("#define _AVX10_2INTRIN_H")
            .raw_line("#define _AVX10_2CONVERTINTRIN_H")
            .raw_line("#define _AVX10_2SATCVTINTRIN_H")
            .raw_line("#define _AVX10_2COPYINTRIN_H")
            .raw_line("#define _AVX10_2MEDIAINTRIN_H")
            .raw_line("#define _AVX10_2MINMAXINTRIN_H");

        // Block problematic AVX512 FP16 types
        bindings = bindings
            .blocklist_item("__m512h")
            .blocklist_item("__m256h")
            .blocklist_item("__m128h")
            .blocklist_item("__v8hf")
            .blocklist_item("__v16hf")
            .blocklist_item("__v32hf")
            // Block problematic type aliases
            .blocklist_item("_Float16")
            .blocklist_type("_Float16")
            // Block the specific intrinsics headers
            .blocklist_file(".*amxavx512intrin\\.h")
            .blocklist_file(".*avx10_2convertintrin\\.h")
            .blocklist_file(".*avx512fp16intrin\\.h")
            .blocklist_file(".*avx512vlfp16intrin\\.h")
            // Block functions that use these types
            .blocklist_function("_tile_cmmimfp16ps")
            .blocklist_function("_tile_cmmrlfp16ps");
    }

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
            }) => lit.base10_parse::<u8>().unwrap_or(1) != 0,
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
