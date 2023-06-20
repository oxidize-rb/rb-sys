use std::path::Path;

pub fn compile() {
    let mut build = rb_sys_build::cc::Build::new();
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = crate_dir.join("src").join("stable_api").join("compiled.c");

    println!("cargo:rerun-if-changed={}", path.display());
    build.file(path);
    build.flag_if_supported("-Wno-unused-parameter");
    build.try_compile("compiled").unwrap_or_else(|e| {
        panic!(
            "Failed when attempting to compile C glue code for needed for the Ruby stable ABI: {}",
            e
        );
    });

    std::process::exit(1);
}
