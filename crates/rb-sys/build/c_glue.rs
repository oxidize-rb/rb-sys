pub fn compile() {
    use crate::features::is_extra_warnings_enabled;
    use std::path::Path;

    let mut build = rb_sys_build::cc::Build::new();
    if !is_extra_warnings_enabled() {
        build.warnings(false);
    }
    let path = Path::new("src").join("stable_abi").join("compiled.c");
    println!("cargo:rerun-if-changed={}", path.display());
    build.file(path);
    build.try_compile("compiled").unwrap_or_else(|e| {
        panic!(
            "Failed when attempting to compile C glue code for needed for the Ruby stable ABI: {}",
            e
        );
    });
}
