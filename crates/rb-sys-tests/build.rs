use rb_sys_build::rb_config;
use std::env;

fn main() {
    export_cargo_cfg();
    rb_config()
        .link_ruby(!rb_config().enable_shared())
        .print_cargo_args();
}

fn export_cargo_cfg() {
    rustc_cfg("version");
    rustc_cfg("major");
    rustc_cfg("minor");
    rustc_cfg("teeny");
    rustc_cfg("patchlevel");
    rustc_cfg("version_gte_3_2");
    rustc_cfg("version_gte_3_1");

    if env::var("DEP_RB_VERSION_GTE_3_1") == Ok("true".to_string()) && cfg!(windows) {
        println!("cargo:rustc-cfg=windows_broken_vm_init_3_1");
    }
}

fn rustc_cfg(name: &str) {
    let val = env::var(format!("DEP_RB_{}", &name.to_uppercase()))
        .unwrap_or_else(|_| panic!("{} not found", name));
    println!("cargo:rustc-cfg=ruby_{}=\"{}\"", name, val);
}
