extern crate bindgen;
extern crate pkg_config;

mod bindings;
mod rbconfig;
mod version;

use rbconfig::rbconfig;
use std::env;
use std::path::Path;
use version::Version;

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
    let library = setup_ruby_pkgconfig();

    println!("cargo:rustc-link-search=native={}", rbconfig("libdir"));

    // Setup rpath on unix to hardcode the ruby library path
    if cfg!(unix) {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", rbconfig("libdir"));

        library.link_paths.iter().for_each(|path| {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path.display());
        });
    }
}

fn add_platform_link_args() {
    if cfg!(windows) {
        // println!("cargo:rustc-link-arg=-Wl,--dynamicbase");
        // println!("cargo:rustc-link-arg=-Wl,--disable-auto-image-base");
        println!("cargo:rustc-link-arg=-static-libgcc");

        let libruby_arg = if is_static() {
            rbconfig("LIBRUBYARG_STATIC")
        } else {
            rbconfig("LIBRUBYARG")
        };

        for arg in shell_words::split(&libruby_arg).expect("Could not split libruby arg") {
            println!("cargo:rustc-link-arg={}", arg);
        }
    }
}

// Setting up pkgconfig on windows takes a little more work. We need to setup
// the pkgconfig path in a different way and define the prefix variable.
#[cfg(target_os = "windows")]
fn adjust_pkgconfig(config: &mut pkg_config::Config) -> &mut pkg_config::Config {
    config
        .arg("--with-path")
        .arg(format!("{}/pkgconfig", rbconfig("libdir")))
        .arg("--prefix-variable")
        .arg(rbconfig("libdir").replace("/lib", ""))
        .arg("--define-prefix")
}

#[cfg(not(target_os = "windows"))]
fn adjust_pkgconfig(config: &mut pkg_config::Config) -> &mut pkg_config::Config {
    config
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

    println!("cargo:root={}", rbconfig("prefix"));
    println!("cargo:version={}", rbconfig("ruby_version"));
    println!("cargo:major={}", rbconfig("MAJOR"));
    println!("cargo:minor={}", rbconfig("MINOR"));
    println!("cargo:teeny={}", rbconfig("TEENY"));
    println!("cargo:patchlevel={}", rbconfig("PATCHLEVEL"));

    if is_static() {
        println!("cargo:lib={}-static", rbconfig("RUBY_SO_NAME"));
    } else {
        println!("cargo:lib={}", rbconfig("RUBY_SO_NAME"));
    }

    println!("cargo:libdir={}", rbconfig("libdir"));
}

fn setup_ruby_pkgconfig() -> pkg_config::Library {
    match env::var("PKG_CONFIG_PATH") {
        Ok(val) => env::set_var(
            "PKG_CONFIG_PATH",
            &format!("{}/pkgconfig:{}", rbconfig("libdir"), val),
        ),
        Err(_) => env::set_var(
            "PKG_CONFIG_PATH",
            &format!("{}/pkgconfig", rbconfig("libdir")),
        ),
    }

    let ruby_version = rbconfig("ruby_version");

    let mut config = adjust_pkgconfig(pkg_config::Config::new().cargo_metadata(true))
        .exactly_version(ruby_version.as_str())
        .statik(is_static())
        .to_owned();

    config.probe(ruby_lib_name().as_str()).unwrap_or_else(|_| {
        config
            .statik(true)
            .probe(ruby_lib_name().as_str())
            .unwrap_or_else(|_| panic!("{} not found, needed for pkg-config", ruby_lib_name()))
    })
}

fn ruby_lib_name() -> String {
    Path::new(rbconfig("ruby_pc").as_str())
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
}

fn is_static() -> bool {
    println!("cargo:rerun-if-env-changed=RUBY_STATIC");

    match env::var("RUBY_STATIC") {
        Ok(val) => val == "true" || val == "1",
        _ => cfg!(feature = "ruby-static"),
    }
}

fn rustc_cfg(name: &str, key: &str) {
    println!("cargo:rustc-cfg={}=\"{}\"", name, rbconfig(key));
}

fn compile_ruby_macros() {
    let mut build = cc::Build::new();
    let mut cc_args = shell_words::split(&rbconfig("CC")).expect("CC is not a valid shell word");
    let libs = shell_words::split(&rbconfig("LIBS")).expect("cannot split LIBS");

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
    build.include(format!("{}/include/internal", rbconfig("rubyhdrdir")));
    build.include(format!("{}/include/impl", rbconfig("rubyhdrdir")));
    build.include(rbconfig("rubyhdrdir"));
    build.include(rbconfig("rubyarchhdrdir"));
    build.flag("-fms-extensions");
    build.flag("-Wunused-parameter");

    let cflags_str = rbconfig("cflags");
    let rb_cflags = shell_words::split(&cflags_str).expect("failed to parse CFLAGS");

    for flag in rb_cflags {
        build.flag(&flag);
    }

    build.compile("ruby_macros");
}

fn has_ruby_dln_check_abi() -> bool {
    let major = rbconfig("MAJOR").parse::<i32>().unwrap();
    let minor = rbconfig("MINOR").parse::<i32>().unwrap();

    major >= 3 && minor >= 2 && !cfg!(target_family = "windows")
}
