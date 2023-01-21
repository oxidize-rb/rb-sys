mod features;
mod ruby_macros;
mod version;

use features::*;
use rb_sys_build::{bindings, RbConfig};
use std::fs;
use version::Version;

const SUPPORTED_RUBY_VERSIONS: [Version; 8] = [
    Version::new(2, 3),
    Version::new(2, 4),
    Version::new(2, 5),
    Version::new(2, 6),
    Version::new(2, 7),
    Version::new(3, 0),
    Version::new(3, 1),
    Version::new(3, 2),
];

fn main() {
    let mut rbconfig = RbConfig::current();

    println!("cargo:rerun-if-env-changed=RUBY_ROOT");
    println!("cargo:rerun-if-env-changed=RUBY_VERSION");
    println!("cargo:rerun-if-env-changed=RUBY");
    println!("cargo:rerun-if-changed=wrapper.h");

    for file in fs::read_dir("build").unwrap() {
        println!("cargo:rerun-if-changed={}", file.unwrap().path().display());
    }

    bindings::generate(&rbconfig, is_ruby_static_enabled(&rbconfig));
    export_cargo_cfg(&mut rbconfig);

    if is_ruby_macros_enabled() {
        ruby_macros::compile(&mut rbconfig);
    }

    if is_link_ruby_enabled() {
        link_libruby(&mut rbconfig);
    } else {
        add_libruby_to_blocklist(&mut rbconfig);
        enable_dynamic_lookup(&mut rbconfig);
    }

    expose_cargo_features();
    add_unsupported_link_args_to_blocklist(&mut rbconfig);
    rbconfig.print_cargo_args();

    if is_debug_build_enabled() {
        debug_and_exit(&mut rbconfig);
    }
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

fn export_cargo_cfg(rbconfig: &mut RbConfig) {
    rustc_cfg(rbconfig, "ruby_major", "MAJOR");
    rustc_cfg(rbconfig, "ruby_minor", "MINOR");
    rustc_cfg(rbconfig, "ruby_teeny", "TEENY");
    rustc_cfg(rbconfig, "ruby_patchlevel", "PATCHLEVEL");
    rustc_cfg(rbconfig, "ruby_api_version", "RUBY_API_VERSION");

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
            println!(r#"cargo:version_lt_{}_{}=true"#, v.major(), v.minor());
        } else {
            println!(r#"cargo:version_lt_{}_{}=false"#, v.major(), v.minor());
        }

        if &version <= v {
            println!(r#"cargo:rustc-cfg=ruby_lte_{}_{}"#, v.major(), v.minor());
            println!(r#"cargo:version_lte_{}_{}=true"#, v.major(), v.minor());
        } else {
            println!(r#"cargo:version_lte_{}_{}=false"#, v.major(), v.minor());
        }

        if &version == v {
            println!(r#"cargo:rustc-cfg=ruby_eq_{}_{}"#, v.major(), v.minor());
            println!(r#"cargo:version_eq_{}_{}=true"#, v.major(), v.minor());
        } else {
            println!(r#"cargo:version_eq_{}_{}=false"#, v.major(), v.minor());
        }

        if &version >= v {
            println!(r#"cargo:rustc-cfg=ruby_gte_{}_{}"#, v.major(), v.minor());
            println!(r#"cargo:version_gte_{}_{}=true"#, v.major(), v.minor());
        } else {
            println!(r#"cargo:version_gte_{}_{}=false"#, v.major(), v.minor());
        }

        if &version > v {
            println!(r#"cargo:rustc-cfg=ruby_gt_{}_{}"#, v.major(), v.minor());
            println!(r#"cargo:version_gt_{}_{}=true"#, v.major(), v.minor());
        } else {
            println!(r#"cargo:version_gt_{}_{}=false"#, v.major(), v.minor());
        }
    }

    println!("cargo:root={}", rbconfig.get("prefix"));
    println!("cargo:include={}", rbconfig.get("includedir"));
    println!("cargo:archinclude={}", rbconfig.get("archincludedir"));
    println!("cargo:version={}", rbconfig.get("ruby_version"));
    println!("cargo:major={}", rbconfig.get("MAJOR"));
    println!("cargo:minor={}", rbconfig.get("MINOR"));
    println!("cargo:teeny={}", rbconfig.get("TEENY"));
    println!("cargo:patchlevel={}", rbconfig.get("PATCHLEVEL"));

    for key in rbconfig.all_keys() {
        println!("cargo:rbconfig_{}=\"{}\"", key, rbconfig.get(key));
    }

    if is_ruby_static_enabled(rbconfig) {
        println!("cargo:lib={}", rbconfig.libruby_static_name());
        println!("cargo:ruby_static=true");
    } else {
        println!("cargo:lib={}", rbconfig.libruby_so_name());
    }

    println!("cargo:libdir={}", rbconfig.get("libdir"));
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

fn expose_cargo_features() {
    for (key, val) in std::env::vars() {
        if !key.starts_with("CARGO_FEATURE_") {
            continue;
        }

        println!("cargo:{}={}", key.to_lowercase(), val);
    }
}
