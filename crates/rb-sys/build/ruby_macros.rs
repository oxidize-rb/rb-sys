use rb_sys_build::RbConfig;

#[cfg(feature = "ruby-macros")]
pub fn compile(_rbconfig: &mut RbConfig) {
    use std::path::Path;

    let mut build = rb_sys_build::cc::Build::new();
    let path = Path::new("src").join("macros").join("ruby_macros.c");
    build.file(path);
    build.compile("ruby_macros");
}

#[cfg(not(feature = "ruby-macros"))]
pub fn compile(_rbconfig: &mut RbConfig) {}
