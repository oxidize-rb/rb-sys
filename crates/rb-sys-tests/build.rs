use std::env;

fn main() {
    export_cargo_cfg();
}

fn export_cargo_cfg() {
    rustc_cfg("version");
    rustc_cfg("major");
    rustc_cfg("minor");
    rustc_cfg("teeny");
    rustc_cfg("patchlevel");
}

fn rustc_cfg(name: &str) {
    let val = env::var(format!("DEP_RUBY_{}", &name.to_uppercase()))
        .unwrap_or_else(|_| panic!("{} not found", name));
    println!("cargo:rustc-cfg=ruby_{}=\"{}\"", name, val);
}
