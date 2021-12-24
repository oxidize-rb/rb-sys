extern crate bindgen;
extern crate pkg_config;

use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

fn setup_ruby_pkgconfig() -> pkg_config::Library {
    match env::var("PKG_CONFIG_PATH") {
        Ok(val) => env::set_var(
            "PKG_CONFIG_PATH",
            &format!("{}:{}/pkgconfig", val, rbconfig("libdir")),
        ),
        Err(_) => env::set_var(
            "PKG_CONFIG_PATH",
            &format!("{}/pkgconfig", rbconfig("libdir")),
        ),
    }

    pkg_config::Config::new()
        .probe(format!("ruby-{}.{}", rbconfig("MAJOR"), rbconfig("MINOR")).as_str())
        .unwrap()
}

fn rbconfig(key: &str) -> String {
    let ruby = env::var_os("RUBY").unwrap_or(OsString::from("ruby"));

    let config = Command::new(ruby)
        .arg("-e")
        .arg(format!("print RbConfig::CONFIG['{}']", key))
        .output()
        .unwrap_or_else(|e| panic!("ruby not found: {}", e));

    String::from_utf8(config.stdout).expect("RbConfig value not UTF-8!")
}

fn main() {
    let library = setup_ruby_pkgconfig();

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");

    // Make sure we have the rpath set so libruby can be foudn when the program runs
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", rbconfig("libdir"));

    let mut clang_args = library
        .include_paths
        .iter()
        .map(|path| format!("-I{}", path.to_str().unwrap()).to_string())
        .collect::<Vec<_>>();

    clang_args.push("-fdeclspec".to_string());

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .use_core()
        .ctypes_prefix("::libc")
        .allowlist_file(".*ruby.*")
        .rustified_enum("*")
        .new_type_alias_deref("VALUE")
        .default_alias_style(bindgen::AliasVariation::NewType)
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
}