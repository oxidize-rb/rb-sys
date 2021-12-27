extern crate bindgen;
extern crate pkg_config;

use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;

#[cfg(target_os = "windows")]
fn delete<'a>(s: &'a str, from: &'a str) -> String {
    let mut result = String::new();
    let mut last_end = 0;
    for (start, part) in s.match_indices(from) {
        result.push_str(unsafe { s.get_unchecked(last_end..start) });
        last_end = start + part.len();
    }
    result.push_str(unsafe { s.get_unchecked(last_end..s.len()) });
    result
}

#[cfg(target_os = "windows")]
fn purge_refptr_text() {
    let buffer = fs::read_to_string("exports.def").expect("Failed to read 'exports.def'");
    fs::write("exports.def", delete(&buffer, ".refptr."))
        .expect("Failed to write update to 'exports.def'");
}

#[cfg(target_os = "windows")]
fn adjust_pkgconfig(config: &mut pkg_config::Config) -> &mut pkg_config::Config {
    // A lot taken from https://github.com/danielpclark/rutie/blob/cba311cbb5873ef42ad627081f2dec04feab9a51/build.rs#L122
    println!("cargo:rustc-link-search={}", rbconfig("bindir"));
    let mingw_libs: OsString = env::var_os("MINGW_LIBS").unwrap_or(OsString::from(format!(
        "{}/ruby_builtin_dlls",
        rbconfig("bindir")
    )));
    println!("cargo:rustc-link-search={}", mingw_libs.to_string_lossy());

    let libruby_so = rbconfig("LIBRUBY_SO");
    let ruby_dll = Path::new(&libruby_so);
    let name = ruby_dll.file_stem().unwrap();

    Command::new("build/windows/vcbuild.cmd")
        .arg("-arch=x64")
        .arg("-host_arch=x64")
        .arg("&&")
        .arg("dumpbin")
        .arg("/exports")
        .arg("/out:exports.txt")
        .arg(Path::new(&rbconfig("bindir")).join(&libruby_so))
        .output()
        .unwrap();

    Command::new("build/windows/exports.bat").output().unwrap();

    purge_refptr_text();
    Command::new("build/windows/vcbuild.cmd")
        .arg("-arch=x64")
        .arg("-host_arch=x64")
        .arg("&&")
        .arg("lib")
        .arg("/def:exports.def")
        .arg(format!("/name:{}", name.to_string_lossy()))
        .arg(format!("/libpath:{}", rbconfig("bindir")))
        .arg("/machine:x64")
        .arg(format!("/out:{}", Path::new(&rbconfig("bindir")).join(name).to_string_lossy()))
        .output()
        .unwrap();

    fs::remove_file("exports.def").expect("couldn't remove exports.def");
    fs::remove_file("exports.txt").expect("couldn't remove exports.txt");

    config
        .statik(false)
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

    let mut config = adjust_pkgconfig(pkg_config::Config::new().cargo_metadata(true)).to_owned();

    let ruby_name = format!("ruby-{}.{}", rbconfig("MAJOR"), rbconfig("MINOR")).to_string();

    config
        .probe(ruby_name.as_str())
        .unwrap_or_else(|_| config.statik(true).probe(ruby_name.as_str()).unwrap())
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
    setup_ruby_pkgconfig();

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
