extern crate bindgen;

use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

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
    let ruby_libs = rbconfig("LIBS");

    if !ruby_libs.trim().is_empty() {
        println!("cargo:rustc-link-arg={}", ruby_libs);
    }

    let libruby_arg = rbconfig("LIBRUBYARG");

    if !libruby_arg.trim().is_empty() {
        println!("cargo:rustc-link-arg={}", libruby_arg);
    }

    rbconfig("DLDFLAGS").split(' ').for_each(|arg| {
        println!("cargo:rustc-link-arg={}", arg);
    });

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .use_core()
        .ctypes_prefix("::libc")
        .blocklist_item("mjit.*")
        .clang_args(&[
            format!("-I{}", rbconfig("rubyhdrdir")),
            format!("-I{}", rbconfig("rubyarchhdrdir")),
            "-fdeclspec".to_string(),
        ])
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
