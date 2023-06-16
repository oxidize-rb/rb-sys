use rb_sys_build::RbConfig;

#[cfg(feature = "compiled-stable-abi")]
pub fn compile(_rbconfig: &mut RbConfig) {
    println!("cargo:rerun-if-changed=src/stable_abi/compiled.c");
    use std::path::Path;

    let mut build = rb_sys_build::cc::Build::new();
    let path = Path::new("src").join("stable_abi").join("compiled.c");
    build.file(path);
    build.compile("compiled");
    println!("cargo:rustc-cfg=compiled_stable_abi_available");
}

#[cfg(not(feature = "compiled-stable-abi"))]
pub fn compile(_rbconfig: &mut RbConfig) {}
