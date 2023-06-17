#[cfg(feature = "stable-abi")]
pub fn compile() {
    use std::path::Path;
    println!("cargo:rerun-if-changed=src/stable_abi/compiled.c");
    let mut build = rb_sys_build::cc::Build::new();
    let path = Path::new("src").join("stable_abi").join("compiled.c");
    build.file(path);
    build.try_compile("compiled").unwrap_or_else(|e| {
        panic!(
            "Failed when attempting to compile C glue code for needed for the Ruby stable ABI: {}",
            e
        );
    });
}

#[cfg(not(feature = "stable-abi"))]
pub fn compile() {}
