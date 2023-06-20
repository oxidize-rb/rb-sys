mod c_glue;
mod features;
mod version;

use features::*;
use rb_sys_build::{bindings, RbConfig};
use std::io::Write;
use std::{
    env,
    fs::{self, File},
    path::PathBuf,
};
use version::Version;

const SUPPORTED_RUBY_VERSIONS: [Version; 9] = [
    Version::new(2, 3),
    Version::new(2, 4),
    Version::new(2, 5),
    Version::new(2, 6),
    Version::new(2, 7),
    Version::new(3, 0),
    Version::new(3, 1),
    Version::new(3, 2),
    Version::new(3, 3),
];

const LATEST_STABLE_VERSION: Version = Version::new(3, 2);
const MIN_SUPPORTED_STABLE_VERSION: Version = Version::new(2, 6);

fn main() {
    warn_deprecated_feature_flags();

    let mut rbconfig = RbConfig::current();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let ruby_version = rbconfig.ruby_program_version();
    let ruby_platform = rbconfig.platform();
    let current_ruby_version = Version::current(&rbconfig);
    let crate_version = env!("CARGO_PKG_VERSION");
    let cfg_capture_path = out_dir.join(format!(
        "cfg-capture-{}-{}-{}",
        crate_version, ruby_platform, ruby_version
    ));
    let mut cfg_capture_file = File::create(cfg_capture_path).unwrap();

    if current_ruby_version < MIN_SUPPORTED_STABLE_VERSION {
        println!(
            "cargo:warning=Support for Ruby {} will be removed in a future release.",
            current_ruby_version
        );
    }

    println!("cargo:rerun-if-env-changed=RUBY_ROOT");
    println!("cargo:rerun-if-env-changed=RUBY_VERSION");
    println!("cargo:rerun-if-env-changed=RUBY");

    for file in fs::read_dir("build").unwrap() {
        println!("cargo:rerun-if-changed={}", file.unwrap().path().display());
    }

    let bindings_path = bindings::generate(
        &rbconfig,
        is_ruby_static_enabled(&rbconfig),
        &mut cfg_capture_file,
    )
    .expect("generate bindings");
    println!(
        "cargo:rustc-env=RB_SYS_BINDINGS_PATH={}",
        bindings_path.display()
    );
    export_cargo_cfg(&mut rbconfig, &mut cfg_capture_file);

    if is_compiled_stable_api_needed(&current_ruby_version) {
        c_glue::compile();
    }

    if is_link_ruby_enabled() {
        link_libruby(&mut rbconfig);
    } else {
        add_libruby_to_blocklist(&mut rbconfig);
        enable_dynamic_lookup(&mut rbconfig);
    }

    expose_cargo_features(&mut cfg_capture_file);
    add_unsupported_link_args_to_blocklist(&mut rbconfig);
    rbconfig.print_cargo_args();

    if is_debug_build_enabled() {
        debug_and_exit(&mut rbconfig);
    }
}

macro_rules! cfg_capture {
    ($file:expr, $fmt:expr, $($arg:tt)*) => {
        println!($fmt, $($arg)*);
        writeln!($file, $fmt, $($arg)*).unwrap();
    };
}

fn add_unsupported_link_args_to_blocklist(rbconfig: &mut RbConfig) {
    rbconfig.blocklist_link_arg("-Wl,--compress-debug-sections=zlib");
    rbconfig.blocklist_link_arg("-s");
}

fn add_libruby_to_blocklist(rbconfig: &mut RbConfig) {
    rbconfig.blocklist_lib(&rbconfig.libruby_so_name());
    rbconfig.blocklist_lib(&rbconfig.libruby_static_name());
}

fn debug_and_exit(rbconfig: &mut RbConfig) {
    eprintln!("========== RbConfig\n");
    dbg!(rbconfig);

    eprintln!("========== Environment Variables\n");
    let env: std::collections::HashMap<_, _> = std::env::vars().collect();
    dbg!(env);

    eprintln!("==========\n");
    eprintln!("The \"RB_SYS_DEBUG_BUILD\" env var was set, aborting.");
    std::process::exit(1);
}

fn link_libruby(rbconfig: &mut RbConfig) {
    if is_link_ruby_enabled() {
        rbconfig.link_ruby(is_ruby_static_enabled(rbconfig));
    }
}

fn export_cargo_cfg(rbconfig: &mut RbConfig, cap: &mut File) {
    rustc_cfg(rbconfig, "ruby_major", "MAJOR");
    rustc_cfg(rbconfig, "ruby_minor", "MINOR");
    rustc_cfg(rbconfig, "ruby_teeny", "TEENY");
    rustc_cfg(rbconfig, "ruby_patchlevel", "PATCHLEVEL");
    rustc_cfg(rbconfig, "ruby_api_version", "RUBY_API_VERSION");

    if Version::current(rbconfig) <= LATEST_STABLE_VERSION {
        println!("cargo:rustc-cfg=ruby_api_stable");
    }

    if is_global_allocator_enabled(rbconfig) {
        println!("cargo:rustc-cfg=use_global_allocator");
    }

    if is_gem_enabled() {
        println!("cargo:rustc-cfg=use_ruby_abi_version");
    }

    if rbconfig.has_ruby_dln_check_abi() {
        println!("cargo:rustc-cfg=has_ruby_abi_version");
    }

    let version = Version::current(rbconfig);

    for v in SUPPORTED_RUBY_VERSIONS.iter() {
        let v = v.to_owned();

        if &version < v {
            println!(r#"cargo:rustc-cfg=ruby_lt_{}_{}"#, v.major(), v.minor());
            cfg_capture!(cap, r#"cargo:version_lt_{}_{}=true"#, v.major(), v.minor());
        } else {
            cfg_capture!(cap, r#"cargo:version_lt_{}_{}=false"#, v.major(), v.minor());
        }

        if &version <= v {
            println!(r#"cargo:rustc-cfg=ruby_lte_{}_{}"#, v.major(), v.minor());
            cfg_capture!(cap, r#"cargo:version_lte_{}_{}=true"#, v.major(), v.minor());
        } else {
            cfg_capture!(
                cap,
                r#"cargo:version_lte_{}_{}=false"#,
                v.major(),
                v.minor()
            );
        }

        if &version == v {
            println!(r#"cargo:rustc-cfg=ruby_eq_{}_{}"#, v.major(), v.minor());
            cfg_capture!(cap, r#"cargo:version_eq_{}_{}=true"#, v.major(), v.minor());
        } else {
            cfg_capture!(cap, r#"cargo:version_eq_{}_{}=false"#, v.major(), v.minor());
        }

        if &version >= v {
            println!(r#"cargo:rustc-cfg=ruby_gte_{}_{}"#, v.major(), v.minor());
            cfg_capture!(cap, r#"cargo:version_gte_{}_{}=true"#, v.major(), v.minor());
        } else {
            cfg_capture!(
                cap,
                r#"cargo:version_gte_{}_{}=false"#,
                v.major(),
                v.minor()
            );
        }

        if &version > v {
            println!(r#"cargo:rustc-cfg=ruby_gt_{}_{}"#, v.major(), v.minor());
            cfg_capture!(cap, r#"cargo:version_gt_{}_{}=true"#, v.major(), v.minor());
        } else {
            cfg_capture!(cap, r#"cargo:version_gt_{}_{}=false"#, v.major(), v.minor());
        }
    }

    cfg_capture!(cap, "cargo:root={}", rbconfig.get("prefix"));
    cfg_capture!(cap, "cargo:include={}", rbconfig.get("includedir"));
    cfg_capture!(cap, "cargo:archinclude={}", rbconfig.get("archincludedir"));
    cfg_capture!(cap, "cargo:version={}", rbconfig.get("ruby_version"));
    cfg_capture!(cap, "cargo:major={}", rbconfig.get("MAJOR"));
    cfg_capture!(cap, "cargo:minor={}", rbconfig.get("MINOR"));
    cfg_capture!(cap, "cargo:teeny={}", rbconfig.get("TEENY"));
    cfg_capture!(cap, "cargo:patchlevel={}", rbconfig.get("PATCHLEVEL"));

    for key in rbconfig.all_keys() {
        cfg_capture!(cap, "cargo:rbconfig_{}={}", key, rbconfig.get(key));
    }

    if is_ruby_static_enabled(rbconfig) {
        cfg_capture!(cap, "cargo:lib={}", rbconfig.libruby_static_name());
        cfg_capture!(cap, "cargo:ruby_static={}", "true");
    } else {
        cfg_capture!(cap, "cargo:lib={}", rbconfig.libruby_so_name());
    }

    cfg_capture!(cap, "cargo:libdir={}", rbconfig.get("libdir"));
}

fn rustc_cfg(rbconfig: &RbConfig, name: &str, key: &str) {
    if let Some(k) = rbconfig.get_optional(key) {
        println!("cargo:rustc-cfg={}=\"{}\"", name, k);
    }
}

fn enable_dynamic_lookup(rbconfig: &mut RbConfig) {
    // See https://github.com/oxidize-rb/rb-sys/issues/88
    if cfg!(target_os = "macos") {
        rbconfig.push_dldflags("-Wl,-undefined,dynamic_lookup");
    }
}

fn expose_cargo_features(cap: &mut File) {
    for (key, val) in std::env::vars() {
        if !key.starts_with("CARGO_FEATURE_") {
            continue;
        }

        cfg_capture!(cap, "cargo:{}={}", key.to_lowercase(), val);
    }
}

fn warn_deprecated_feature_flags() {
    if cfg!(feature = "ruby-macros") {
        println!("cargo:warning=The \"ruby-macros\" feature flag is deprecated and will be removed in a future release.");
    }
}
