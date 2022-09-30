extern crate bindgen;

mod bindings;
mod features;
mod ruby_macros;
mod utils;
mod version;

use features::*;
use rb_sys_build::RbConfig;
use std::fs;
use utils::is_msvc;
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

    println!("cargo:rerun-if-env-changed=RUBY_VERSION");
    println!("cargo:rerun-if-env-changed=RUBY");
    println!("cargo:rerun-if-changed=wrapper.h");

    for file in fs::read_dir("build").unwrap() {
        println!("cargo:rerun-if-changed={}", file.unwrap().path().display());
    }

    bindings::generate(&rbconfig);
    export_cargo_cfg(&mut rbconfig);

    if is_ruby_macros_enabled() {
        ruby_macros::compile(&mut rbconfig);
    }

    if is_link_ruby_enabled() {
        link_libruby(&mut rbconfig);
    } else {
        add_libruby_to_blocklist(&mut rbconfig)
    }

    rbconfig.print_cargo_args();

    if is_debug_build_enabled() {
        debug_and_exit(&mut rbconfig);
    }
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
    eprintln!("The \"debug-build\" feature for rb-sys is enabled, aborting.");
    std::process::exit(1);
}

fn link_libruby(rbconfig: &mut RbConfig) {
    if is_link_ruby_enabled() {
        rbconfig.push_dldflags(&format!("-L{}", &rbconfig.get("libdir")));

        if is_ruby_static_enabled(rbconfig) {
            rbconfig.push_dldflags(&rbconfig.get("LIBRUBYARG_STATIC"));
        } else {
            rbconfig.push_dldflags(&rbconfig.get("LIBRUBYARG_SHARED"));
        }

        // Setup rpath on unix to hardcode the ruby library path
        if cfg!(unix) {
            rbconfig.libs.iter().for_each(|lib| {
                println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib.name);
            });
        } else if is_msvc() {
            rbconfig.push_dldflags("/LINK");
        }
    }
}

fn export_cargo_cfg(rbconfig: &mut RbConfig) {
    rustc_cfg(rbconfig, "ruby_major", "MAJOR");
    rustc_cfg(rbconfig, "ruby_minor", "MINOR");
    rustc_cfg(rbconfig, "ruby_teeny", "TEENY");
    rustc_cfg(rbconfig, "ruby_patchlevel", "PATCHLEVEL");
    rustc_cfg(rbconfig, "ruby_api_version", "RUBY_API_VERSION");

    if has_ruby_dln_check_abi(rbconfig) {
        println!("cargo:rustc-cfg=has_ruby_abi_version");
    }

    if is_global_allocator_enabled(rbconfig) {
        println!("cargo:rustc-cfg=use_global_allocator");
    }

    if is_ruby_abi_version_enabled() {
        println!("cargo:rustc-cfg=use_ruby_abi_version");
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

    if is_ruby_static_enabled(rbconfig) {
        println!("cargo:lib={}", rbconfig.libruby_static_name());
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

fn has_ruby_dln_check_abi(rbconfig: &RbConfig) -> bool {
    let major = rbconfig.get("MAJOR").parse::<i32>().unwrap();
    let minor = rbconfig.get("MINOR").parse::<i32>().unwrap();

    major >= 3 && minor >= 2 && !cfg!(target_family = "windows")
}
