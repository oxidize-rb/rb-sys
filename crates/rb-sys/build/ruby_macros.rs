use rb_sys_build::RbConfig;

#[cfg(feature = "ruby-macros")]
pub fn compile(rbconfig: &mut RbConfig) {
    use std::path::Path;

    use rb_sys_build::utils::shellsplit;

    println!("cargo:rerun-if-changed=src/macros/ruby_macros.h");
    println!("cargo:rerun-if-changed=src/macros/ruby_macros.c");

    let mut build = cc::Build::new();
    let mut cc_args = shellsplit(&rbconfig.get("CC"));
    let libs = shellsplit(&rbconfig.get("LIBS"));

    cc_args.reverse();
    build.compiler(cc_args.pop().expect("CC is empty"));
    cc_args.reverse();

    for arg in cc_args {
        build.flag(&arg);
    }

    for lib in libs {
        build.flag(&lib);
    }

    let path = Path::new("src").join("macros").join("ruby_macros.c");
    build.file(path);
    build.include(format!("{}/include/internal", rbconfig.get("rubyhdrdir")));
    build.include(format!("{}/include/impl", rbconfig.get("rubyhdrdir")));
    build.include(rbconfig.get("rubyhdrdir"));
    build.include(rbconfig.get("rubyarchhdrdir"));
    build.flag("-fms-extensions");
    build.flag("-Wno-error"); // not actionable by user

    for flag in &rbconfig.cflags {
        build.flag(flag);
    }

    build.compile("ruby_macros");
}

#[cfg(not(feature = "ruby-macros"))]
pub fn compile(_rbconfig: &mut RbConfig) {}
