use std::env;
use std::ffi::OsString;
use std::process::Command;

fn main() {
    export_cargo_cfg();
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

fn rustc_cfg(name: &str, key: &str) {
    println!("cargo:rustc-cfg={}=\"{}\"", name, rbconfig(key));
}

fn has_ruby_dln_check_abi() -> bool {
    let major = rbconfig("MAJOR").parse::<i32>().unwrap();
    let minor = rbconfig("MINOR").parse::<i32>().unwrap();

    major >= 3 && minor >= 2 && !cfg!(target_family = "windows")
}

fn rbconfig(key: &str) -> String {
    println!("cargo:rerun-if-env-changed=RBCONFIG_{}", key);
    println!("cargo:rerun-if-env-changed=RUBY_VERSION");
    println!("cargo:rerun-if-env-changed=RUBY");

    match env::var(format!("RBCONFIG_{}", key)) {
        Ok(val) => val,
        Err(_) => {
            let ruby = env::var_os("RUBY").unwrap_or_else(|| OsString::from("ruby"));

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
