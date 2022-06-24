extern crate bindgen;

mod bindings;
mod features;
mod version;

use features::*;
use rb_sys_build::RbConfig;
use std::fs;
use version::Version;

const SUPPORTED_RUBY_VERSIONS: [Version; 4] = [
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
    add_platform_link_args(&mut rbconfig);

    if is_ruby_macros_enabled() {
        compile_ruby_macros(&mut rbconfig);
    }

    if is_link_ruby_enabled() {
        link_libruby(&mut rbconfig);
    } else {
        add_libruby_to_blocklist(&mut rbconfig)
    }

    if is_debug_build_enabled() {
        debug_and_exit(&mut rbconfig);
    }

    rbconfig.print_cargo_args();
}

fn add_libruby_to_blocklist(rbconfig: &mut RbConfig) {
    rbconfig.blocklist_lib(&rbconfig.libruby_so_name());
    rbconfig.blocklist_lib(&rbconfig.libruby_static_name());
}

fn debug_and_exit(rbconfig: &mut RbConfig) {
    dbg!(rbconfig);
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
        }
    }
}

fn add_platform_link_args(rbconfig: &mut RbConfig) {
    if cfg!(windows) {
        println!("cargo:rustc-link-arg=-Wl,--dynamicbase");
        println!("cargo:rustc-link-arg=-Wl,--disable-auto-image-base");
        rbconfig.push_dldflags("-static-libgcc");
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

    if is_global_allocator_enabled() {
        println!("cargo:rustc-cfg=use_global_allocator");
    }

    if is_ruby_abi_version_enabled() {
        println!("cargo:rustc-cfg=use_ruby_abi_version");
    }

    let version = Version::current(rbconfig);

    for v in SUPPORTED_RUBY_VERSIONS {
        if version < v {
            println!(r#"cargo:lt_{}_{}=true"#, v.major(), v.minor());
        } else {
            println!(r#"cargo:lt_{}_{}=false"#, v.major(), v.minor());
        }

        if version <= v {
            println!(r#"cargo:lte_{}_{}=true"#, v.major(), v.minor());
        } else {
            println!(r#"cargo:lte_{}_{}=false"#, v.major(), v.minor());
        }

        if version == v {
            println!(r#"cargo:eq_{}_{}=true"#, v.major(), v.minor());
        } else {
            println!(r#"cargo:eq_{}_{}=false"#, v.major(), v.minor());
        }

        if version >= v {
            println!(r#"cargo:gte_{}_{}=true"#, v.major(), v.minor());
        } else {
            println!(r#"cargo:gte_{}_{}=false"#, v.major(), v.minor());
        }

        if version > v {
            println!(r#"cargo:gt_{}_{}=true"#, v.major(), v.minor());
        } else {
            println!(r#"cargo:gt_{}_{}=false"#, v.major(), v.minor());
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
    println!("cargo:rustc-cfg={}=\"{}\"", name, rbconfig.get(key));
}

#[cfg(feature = "ruby-macros")]
fn compile_ruby_macros(rbconfig: &mut RbConfig) {
    println!("cargo:rerun-if-changed=src/ruby_macros/ruby_macros.h");
    println!("cargo:rerun-if-changed=src/ruby_macros/ruby_macros.c");

    let mut build = cc::Build::new();
    let mut cc_args =
        shell_words::split(&rbconfig.get("CC")).expect("CC is not a valid shell word");
    let libs = shell_words::split(&rbconfig.get("LIBS")).expect("cannot split LIBS");

    cc_args.reverse();
    build.compiler(cc_args.pop().expect("CC is empty"));
    cc_args.reverse();

    for arg in cc_args {
        build.flag(&arg);
    }

    for lib in libs {
        build.flag(&lib);
    }

    build.file("src/macros/ruby_macros.c");
    build.include(format!("{}/include/internal", rbconfig.get("rubyhdrdir")));
    build.include(format!("{}/include/impl", rbconfig.get("rubyhdrdir")));
    build.include(rbconfig.get("rubyhdrdir"));
    build.include(rbconfig.get("rubyarchhdrdir"));
    build.flag("-fms-extensions");
    build.flag("-Wunused-parameter");

    for flag in &rbconfig.cflags {
        build.flag(flag);
    }

    build.compile("ruby_macros");
}

#[cfg(not(feature = "ruby-macros"))]
fn compile_ruby_macros(_rbconfig: &mut RbConfig) {}

fn has_ruby_dln_check_abi(rbconfig: &RbConfig) -> bool {
    let major = rbconfig.get("MAJOR").parse::<i32>().unwrap();
    let minor = rbconfig.get("MINOR").parse::<i32>().unwrap();

    major >= 3 && minor >= 2 && !cfg!(target_family = "windows")
}
