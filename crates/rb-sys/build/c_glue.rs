use std::error::Error;
use std::path::Path;

pub fn compile() -> Result<(), Box<dyn Error>> {
    let mut build = rb_sys_build::cc::Build::new();
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = crate_dir.join("src").join("stable_api").join("compiled.c");

    build.file(path);
    build.try_compile("compiled")
}
