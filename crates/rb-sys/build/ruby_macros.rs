use rb_sys_build::RbConfig;

#[cfg(feature = "ruby-macros")]
fn shellsplit(s: &str) -> Vec<String> {
    match shell_words::split(s) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("shellsplit failed: {}", e);
            s.split_whitespace().map(|s| s.to_string()).collect()
        }
    }
}

#[cfg(feature = "ruby-macros")]
pub fn compile(rbconfig: &mut RbConfig) {
    println!("cargo:rerun-if-changed=src/ruby_macros/ruby_macros.h");
    println!("cargo:rerun-if-changed=src/ruby_macros/ruby_macros.c");

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

    build.file("src/macros/ruby_macros.c");
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
