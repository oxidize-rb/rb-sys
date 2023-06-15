use rb_sys_build::RbConfig;

#[cfg(feature = "compiled-c-impls")]
pub fn compile(_rbconfig: &mut RbConfig) {
    use std::path::Path;

    let mut build = rb_sys_build::cc::Build::new();
    let path = Path::new("src")
        .join("unlinkable")
        .join("compiled_c_impls.c");
    build.file(path);
    build.compile("compiled_c_impls");
    println!("cargo:rustc-cfg=compiled_c_impls_available");
}

#[cfg(not(feature = "compiled-c-impls"))]
pub fn compile(_rbconfig: &mut RbConfig) {}
