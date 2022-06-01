extern crate bindgen;

mod bindings;
mod version;

use lazy_static::lazy_static;
use rb_sys_build::RbConfig;
use std::env;
use version::Version;

lazy_static! {
    /// This is an example for using doc comment attributes
    static ref RBCONFIG: RbConfig = RbConfig::current();
}

const SUPPORTED_RUBY_VERSIONS: [Version; 4] = [
    Version::new(2, 7),
    Version::new(3, 0),
    Version::new(3, 1),
    Version::new(3, 2),
];

fn main() {
    println!("cargo:rerun-if-env-changed=RUBY_VERSION");
    println!("cargo:rerun-if-env-changed=RUBY");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=src/ruby_macros/ruby_macros.h");
    println!("cargo:rerun-if-changed=src/ruby_macros/ruby_macros.c");

    for file in std::fs::read_dir("build").unwrap() {
        println!("cargo:rerun-if-changed={}", file.unwrap().path().display());
    }

    if cfg!(feature = "link-ruby") {
        link_libruby();
    } else if cfg!(unix) {
        println!("cargo:rustc-link-arg=-Wl,-undefined,dynamic_lookup");
    }

    bindings::generate();
    export_cargo_cfg();
    add_platform_link_args();

    if cfg!(feature = "ruby-macros") {
        // Windows does not allow -dynamic_lookup
        if cfg!(windows) {
            link_libruby();
        }
        compile_ruby_macros();
    }
}

fn link_libruby() {
    RBCONFIG.print_cargo_args();

    println!("cargo:rustc-link-search=native={}", RBCONFIG.get("libdir"));

    // Setup rpath on unix to hardcode the ruby library path
    if cfg!(unix) {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", RBCONFIG.get("libdir"));

        RBCONFIG.libs.iter().for_each(|path| {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path.name);
        });
    }
}

fn add_platform_link_args() {
    if cfg!(windows) {
        // println!("cargo:rustc-link-arg=-Wl,--dynamicbase");
        // println!("cargo:rustc-link-arg=-Wl,--disable-auto-image-base");
        println!("cargo:rustc-link-arg=-static-libgcc");

        let libruby_arg = if is_static() {
            RBCONFIG.get("LIBRUBYARG_STATIC")
        } else {
            RBCONFIG.get("LIBRUBYARG")
        };

        for arg in shell_words::split(&libruby_arg).expect("Could not split libruby arg") {
            println!("cargo:rustc-link-arg={}", arg);
        }
    }
}

fn export_cargo_cfg() {
    rustc_cfg("ruby_major", "MAJOR");
    rustc_cfg("ruby_minor", "MINOR");
    rustc_cfg("ruby_teeny", "TEENY");
    rustc_cfg("ruby_patchlevel", "PATCHLEVEL");
    rustc_cfg("ruby_api_version", "RUBY_API_VERSION");

    if has_ruby_dln_check_abi() {
        println!("cargo:rustc-cfg=ruby_dln_check_abi");
    }

    let version = Version::current();

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

    println!("cargo:root={}", RBCONFIG.get("prefix"));
    println!("cargo:version={}", RBCONFIG.get("ruby_version"));
    println!("cargo:major={}", RBCONFIG.get("MAJOR"));
    println!("cargo:minor={}", RBCONFIG.get("MINOR"));
    println!("cargo:teeny={}", RBCONFIG.get("TEENY"));
    println!("cargo:patchlevel={}", RBCONFIG.get("PATCHLEVEL"));

    if is_static() {
        println!("cargo:lib={}-static", RBCONFIG.get("RUBY_SO_NAME"));
    } else {
        println!("cargo:lib={}", RBCONFIG.get("RUBY_SO_NAME"));
    }

    println!("cargo:libdir={}", RBCONFIG.get("libdir"));
}

fn is_static() -> bool {
    println!("cargo:rerun-if-env-changed=RUBY_STATIC");

    match env::var("RUBY_STATIC") {
        Ok(val) => val == "true" || val == "1",
        _ => cfg!(feature = "ruby-static"),
    }
}

fn rustc_cfg(name: &str, key: &str) {
    println!("cargo:rustc-cfg={}=\"{}\"", name, RBCONFIG.get(key));
}

fn compile_ruby_macros() {
    let mut build = cc::Build::new();
    let mut cc_args =
        shell_words::split(&RBCONFIG.get("CC")).expect("CC is not a valid shell word");
    let libs = shell_words::split(&RBCONFIG.get("LIBS")).expect("cannot split LIBS");

    cc_args.reverse();
    build.compiler(cc_args.pop().expect("CC is empty"));
    cc_args.reverse();

    for arg in cc_args {
        build.flag(&arg);
    }

    for lib in libs {
        build.flag(&lib);
    }

    build.file("src/ruby_macros/ruby_macros.c");
    build.include(format!("{}/include/internal", RBCONFIG.get("rubyhdrdir")));
    build.include(format!("{}/include/impl", RBCONFIG.get("rubyhdrdir")));
    build.include(RBCONFIG.get("rubyhdrdir"));
    build.include(RBCONFIG.get("rubyarchhdrdir"));
    build.flag("-fms-extensions");
    build.flag("-Wunused-parameter");

    for flag in &RBCONFIG.cflags {
        build.flag(&flag);
    }

    build.compile("ruby_macros");
}

fn has_ruby_dln_check_abi() -> bool {
    let major = RBCONFIG.get("MAJOR").parse::<i32>().unwrap();
    let minor = RBCONFIG.get("MINOR").parse::<i32>().unwrap();

    major >= 3 && minor >= 2 && !cfg!(target_family = "windows")
}
