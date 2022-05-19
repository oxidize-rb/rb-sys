use std::env;

fn main() {
    export_cargo_cfg();

    println!(
        "cargo:rustc-link-lib=dylib={}",
        env::var("DEP_RB_LIB").unwrap()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        env::var("DEP_RB_LIBDIR").unwrap()
    );

    if cfg!(unix) {
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,{}",
            env::var("DEP_RB_LIBDIR").unwrap()
        );
    }
}

fn export_cargo_cfg() {
    rustc_cfg("version");
    rustc_cfg("major");
    rustc_cfg("minor");
    rustc_cfg("teeny");
    rustc_cfg("patchlevel");
    rustc_cfg("gte_3_2");
    rustc_cfg("lt_3_1");

    if env::var("DEP_RB_GTE_3_1") == Ok("true".to_string()) && cfg!(windows) {
        println!("cargo:rustc-cfg=br_windows_broken_vm_init_3_1");
    }
}

fn rustc_cfg(name: &str) {
    let val = env::var(format!("DEP_RB_{}", &name.to_uppercase()))
        .unwrap_or_else(|_| panic!("{} not found", name));
    println!("cargo:rustc-cfg=ruby_{}=\"{}\"", name, val);
}
