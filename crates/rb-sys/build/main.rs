extern crate bindgen;

mod bindings;
mod version;

use rb_sys_build::RbConfig;
use std::env;
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
    println!("cargo:rerun-if-changed=src/ruby_macros/ruby_macros.h");
    println!("cargo:rerun-if-changed=src/ruby_macros/ruby_macros.c");

    for file in std::fs::read_dir("build").unwrap() {
        println!("cargo:rerun-if-changed={}", file.unwrap().path().display());
    }

    if cfg!(feature = "link-ruby") {
        link_libruby(&mut rbconfig);
    }

    bindings::generate(&rbconfig);
    export_cargo_cfg(&mut rbconfig);
    add_platform_link_args(&mut rbconfig);

    if cfg!(feature = "ruby-macros") {
        // Windows does not allow -dynamic_lookup
        if cfg!(windows) {
            link_libruby(&mut rbconfig);
        }
        compile_ruby_macros(&mut rbconfig);
    }

    rbconfig.print_cargo_args();
}

fn link_libruby(rbconfig: &mut RbConfig) {
    rbconfig.push_dldflags(&format!("-L{}", &rbconfig.get("libdir")));

    if is_static(rbconfig) {
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

    if cfg!(feature = "global-allocator") {
        println!("cargo:rustc-cfg=use_global_allocator");
    }

    if cfg!(feature = "ruby-abi-version") {
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
    println!("cargo:version={}", rbconfig.get("ruby_version"));
    println!("cargo:major={}", rbconfig.get("MAJOR"));
    println!("cargo:minor={}", rbconfig.get("MINOR"));
    println!("cargo:teeny={}", rbconfig.get("TEENY"));
    println!("cargo:patchlevel={}", rbconfig.get("PATCHLEVEL"));

    if is_static(rbconfig) {
        println!("cargo:lib={}-static", rbconfig.get("RUBY_SO_NAME"));
    } else {
        println!("cargo:lib={}", rbconfig.get("RUBY_SO_NAME"));
    }

    println!("cargo:libdir={}", rbconfig.get("libdir"));
}

fn is_static(rbconfig: &RbConfig) -> bool {
    println!("cargo:rerun-if-env-changed=RUBY_STATIC");

    match env::var("RUBY_STATIC") {
        Ok(val) => val == "true" || val == "1",
        _ => cfg!(feature = "ruby-static") || rbconfig.get("ENABLE_SHARED") == "no",
    }
}

fn rustc_cfg(rbconfig: &RbConfig, name: &str, key: &str) {
    println!("cargo:rustc-cfg={}=\"{}\"", name, rbconfig.get(key));
}

fn compile_ruby_macros(rbconfig: &mut RbConfig) {
    let mut build = cc::Build::new();
    let mut cc_args =
        shell_words::split(&rbconfig.get("CC")).expect("CC is not a valid shell word");
    let libs = shell_words::split(&rbconfig.get("LIBS")).expect("cannot split LIBS");
    let cc = cc_args.pop().expect("CC is empty");

    cc_args.reverse();
    build.compiler(&cc);
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

    if cfg!(feature = "experimental-lto") {
        if cc != "clang" {
            panic!("experimental-lto feature is only supported with clang");
        } else {
            build.flag("-flto");
            println!("cargo:rustc-link-arg=-flto");
        }
    }

    for flag in &rbconfig.cflags {
        build.flag(flag);
    }

    build.compile("ruby_macros");
}

fn has_ruby_dln_check_abi(rbconfig: &RbConfig) -> bool {
    let major = rbconfig.get("MAJOR").parse::<i32>().unwrap();
    let minor = rbconfig.get("MINOR").parse::<i32>().unwrap();

    major >= 3 && minor >= 2 && !cfg!(target_family = "windows")
}
