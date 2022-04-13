extern crate bindgen;
extern crate pkg_config;

use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

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

    let ruby_name = Path::new(rbconfig("ruby_pc").as_str())
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    config.probe(ruby_name.as_str()).unwrap_or_else(|_| {
        config
            .statik(true)
            .probe(ruby_name.as_str())
            .expect(format!("{} not found, needed for pkg-config", ruby_name).as_str())
    })
}

fn rbconfig(key: &str) -> String {
    println!("cargo:rerun-if-env-changed=RBCONFIG_{}", key);
    println!("cargo:rerun-if-env-changed=RUBY_VERSION");
    println!("cargo:rerun-if-env-changed=RUBY");

    match env::var(format!("RBCONFIG_{}", key)) {
        Ok(val) => String::from(val),
        Err(_) => {
            let ruby = env::var_os("RUBY").unwrap_or(OsString::from("ruby"));

            let config = Command::new(ruby)
                .arg("--disable-gems")
                .arg("-rrbconfig")
                .arg("-e")
                .arg(format!("print RbConfig::CONFIG['{}']", key))
                .output()
                .unwrap_or_else(|e| panic!("ruby not found: {}", e));

            String::from_utf8(config.stdout).expect("RbConfig value not UTF-8!")
        }
    }
}

fn is_static() -> bool {
    println!("cargo:rerun-if-env-changed=RUBY_STATIC");

    match env::var("RUBY_STATIC") {
        Ok(val) => val == "true" || val == "1",
        _ => cfg!(feature = "ruby-static"),
    }
}

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");

    if cfg!(feature = "link-ruby") {
        let library = setup_ruby_pkgconfig();

        // Setup rpath on unix to hardcode the ruby library path
        if cfg!(unix) {
            library.link_paths.iter().for_each(|path| {
                println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path.display());
            });
        }
    } else if cfg!(unix) {
        println!("cargo:rustc-link-arg=-Wl,-undefined,dynamic_lookup");
    }

    let clang_args = vec![
        format!("-I{}", rbconfig("rubyhdrdir")),
        format!("-I{}", rbconfig("rubyarchhdrdir")),
        "-fms-extensions".to_string(),
    ];

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .use_core()
        .ctypes_prefix("::libc")
        .allowlist_file(".*ruby.*")
        .rustified_enum("*")
        .blocklist_item("ruby_abi_version")
        .blocklist_item("rbimpl_atomic_or")
        .derive_eq(true)
        .derive_debug(true)
        .clang_args(clang_args)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    export_cargo_cfg();
}

fn rustc_cfg(name: &str, key: &str) {
    println!("cargo:rustc-cfg={}=\"{}\"", name, rbconfig(key));
}

fn has_ruby_dln_check_abi() -> bool {
    let major = rbconfig("MAJOR").parse::<i32>().unwrap();
    let minor = rbconfig("MINOR").parse::<i32>().unwrap();

    major >= 3 && minor >= 2 && !cfg!(target_family = "windows")
}
