extern crate bindgen;
extern crate pkg_config;

mod rbconfig;
mod version;

use rbconfig::rbconfig;
use std::env;
use std::path::{Path, PathBuf};
use version::Version;

const SUPPORTED_RUBY_VERSIONS: [Version; 4] = [
    Version::new(2, 7),
    Version::new(3, 0),
    Version::new(3, 1),
    Version::new(3, 2),
];

fn main() {
    println!("cargo:rerun-if-env-changed=RUBY_VERSION");
    println!("cargo:rerun-if-env-changed=RUBY");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/ruby_macros/ruby_macros.h");
    println!("cargo:rerun-if-changed=src/ruby_macros/ruby_macros.c");

    if cfg!(feature = "link-ruby") {
        link_libruby();
    } else if cfg!(unix) {
        println!("cargo:rustc-link-arg=-Wl,-undefined,dynamic_lookup");
    }

    generate_bindings();
    export_cargo_cfg();
    add_platform_link_args();

    if cfg!(feature = "ruby-macros") {
        // Windows does not allow -dynamic_lookup
        if cfg!(windows) {
            link_libruby();
        }
        compile_ruby_macros();
    }
}

fn link_libruby() {
    let library = setup_ruby_pkgconfig();

    println!("cargo:rustc-link-search=native={}", rbconfig("libdir"));

    // Setup rpath on unix to hardcode the ruby library path
    if cfg!(unix) {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", rbconfig("libdir"));

        library.link_paths.iter().for_each(|path| {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path.display());
        });
    }
}

fn add_platform_link_args() {
    if cfg!(windows) {
        // println!("cargo:rustc-link-arg=-Wl,--dynamicbase");
        // println!("cargo:rustc-link-arg=-Wl,--disable-auto-image-base");
        println!("cargo:rustc-link-arg=-static-libgcc");

        let libruby_arg = if is_static() {
            rbconfig("LIBRUBYARG_STATIC")
        } else {
            rbconfig("LIBRUBYARG")
        };

        if rbconfig("MAJOR") == "3" && rbconfig("MINOR") == "0" {
            println!("cargo:rustc-link-lib=static=ssp");
        }

        for arg in shell_words::split(&libruby_arg).expect("Could not split libruby arg") {
            println!("cargo:rustc-link-arg={}", arg);
        }
    }
}

fn generate_bindings() {
    let clang_args = vec![
        format!("-I{}", rbconfig("rubyhdrdir")),
        format!("-I{}", rbconfig("rubyarchhdrdir")),
        "-fms-extensions".to_string(),
    ];

    let bindings = default_bindgen(clang_args)
        .header("wrapper.h")
        .allowlist_function("^(onig(enc)?|rb|ruby)_.*")
        .allowlist_function("eaccess")
        .allowlist_function("explicit_bzero")
        .allowlist_function("setproctitle")
        .allowlist_type("VALUE")
        .allowlist_type("Regexp")
        .allowlist_type("^(Onig|R[A-Z]|re_|rb_|rbimpl_|ruby_|st_).*")
        .allowlist_var("^(Onig|rb_|ruby_).*")
        .allowlist_var("^(FMODE_|INTEGER_|HAVE_|ONIG|Onig|RBIMPL_|RB_|RGENGC_|RUBY_|SIGNEDNESS_|SIZEOF_|USE_).*")
        .allowlist_var("^PRI(.PTRDIFF|.SIZE|.VALUE|.*_PREFIX)$")
        .allowlist_var("ATAN2_INF_C99")
        .allowlist_var("BROKEN_BACKTRACE")
        .allowlist_var("BROKEN_CRYPT")
        .allowlist_var("CASEFOLD_FILESYSTEM")
        .allowlist_var("COROUTINE_H")
        .allowlist_var("DLEXT")
        .allowlist_var("DLEXT_MAXLEN")
        .allowlist_var("ENUM_OVER_INT")
        .allowlist_var("FALSE")
        .allowlist_var("INCLUDE_RUBY_CONFIG_H")
        .allowlist_var("INTERNAL_ONIGENC_CASE_FOLD_MULTI_CHAR")
        .allowlist_var("LIBDIR_BASENAME")
        .allowlist_var("NEGATIVE_TIME_T")
        .allowlist_var("PATH_ENV")
        .allowlist_var("PATH_SEP")
        .allowlist_var("POSIX_SIGNAL")
        .allowlist_var("STACK_GROW_DIRECTION")
        .allowlist_var("STDC_HEADERS")
        .allowlist_var("ST_INDEX_BITS")
        .allowlist_var("THREAD_IMPL_H")
        .allowlist_var("THREAD_IMPL_SRC")
        .allowlist_var("TRUE")
        .allowlist_var("UNALIGNED_WORD_ACCESS")
        .allowlist_var("UNLIMITED_ARGUMENTS")
        .allowlist_var("_ALL_SOURCE")
        .allowlist_var("_GNU_SOURCE")
        .allowlist_var("_POSIX_PTHREAD_SEMANTICS")
        .allowlist_var("_REENTRANT")
        .allowlist_var("_TANDEM_SOURCE")
        .allowlist_var("_THREAD_SAFE")
        .allowlist_var("__EXTENSIONS__")
        .allowlist_var("__STDC_WANT_LIB_EXT1__")
        .blocklist_item("ruby_abi_version")
        .blocklist_item("^rbimpl_.*")
        .blocklist_item("^RBIMPL_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks));

    write_bindings(bindings, "bindings.rs");
}

// Setting up pkgconfig on windows takes a little more work. We need to setup
// the pkgconfig path in a different way and define the prefix variable.
#[cfg(target_os = "windows")]
fn adjust_pkgconfig(config: &mut pkg_config::Config) -> &mut pkg_config::Config {
    config
        .arg("--with-path")
        .arg(format!("{}/pkgconfig", rbconfig("libdir")))
        .arg("--prefix-variable")
        .arg(rbconfig("libdir").replace("/lib", ""))
        .arg("--define-prefix")
}

#[cfg(not(target_os = "windows"))]
fn adjust_pkgconfig(config: &mut pkg_config::Config) -> &mut pkg_config::Config {
    config
}

fn export_cargo_cfg() {
    rustc_cfg("ruby_major", "MAJOR");
    rustc_cfg("ruby_minor", "MINOR");
    rustc_cfg("ruby_teeny", "TEENY");
    rustc_cfg("ruby_patchlevel", "PATCHLEVEL");
    rustc_cfg("ruby_api_version", "RUBY_API_VERSION");

    if has_ruby_dln_check_abi() {
        println!("cargo:rustc-cfg=ruby_dln_check_abi");
    }

    let version = Version::current();

    for v in SUPPORTED_RUBY_VERSIONS {
        if version < v {
            println!(r#"cargo:lt_{}_{}=true"#, v.major(), v.minor());
        } else {
            println!(r#"cargo:lt_{}_{}=false"#, v.major(), v.minor());
        }

        if version <= v {
            println!(r#"cargo:lte_{}_{}=true"#, v.major(), v.minor());
        } else {
            println!(r#"cargo:lte_{}_{}=false"#, v.major(), v.minor());
        }

        if version == v {
            println!(r#"cargo:eq_{}_{}=true"#, v.major(), v.minor());
        } else {
            println!(r#"cargo:eq_{}_{}=false"#, v.major(), v.minor());
        }

        if version >= v {
            println!(r#"cargo:gte_{}_{}=true"#, v.major(), v.minor());
        } else {
            println!(r#"cargo:gte_{}_{}=false"#, v.major(), v.minor());
        }

        if version > v {
            println!(r#"cargo:gt_{}_{}=true"#, v.major(), v.minor());
        } else {
            println!(r#"cargo:gt_{}_{}=false"#, v.major(), v.minor());
        }
    }

    println!("cargo:root={}", rbconfig("prefix"));
    println!("cargo:version={}", rbconfig("ruby_version"));
    println!("cargo:major={}", rbconfig("MAJOR"));
    println!("cargo:minor={}", rbconfig("MINOR"));
    println!("cargo:teeny={}", rbconfig("TEENY"));
    println!("cargo:patchlevel={}", rbconfig("PATCHLEVEL"));

    if is_static() {
        println!("cargo:lib={}-static", rbconfig("RUBY_SO_NAME"));
    } else {
        println!("cargo:lib={}", rbconfig("RUBY_SO_NAME"));
    }

    println!("cargo:libdir={}", rbconfig("libdir"));
}

fn setup_ruby_pkgconfig() -> pkg_config::Library {
    match env::var("PKG_CONFIG_PATH") {
        Ok(val) => env::set_var(
            "PKG_CONFIG_PATH",
            &format!("{}/pkgconfig:{}", rbconfig("libdir"), val),
        ),
        Err(_) => env::set_var(
            "PKG_CONFIG_PATH",
            &format!("{}/pkgconfig", rbconfig("libdir")),
        ),
    }

    let ruby_version = rbconfig("ruby_version");

    let mut config = adjust_pkgconfig(pkg_config::Config::new().cargo_metadata(true))
        .exactly_version(ruby_version.as_str())
        .statik(is_static())
        .to_owned();

    config.probe(ruby_lib_name().as_str()).unwrap_or_else(|_| {
        config
            .statik(true)
            .probe(ruby_lib_name().as_str())
            .unwrap_or_else(|_| panic!("{} not found, needed for pkg-config", ruby_lib_name()))
    })
}

fn ruby_lib_name() -> String {
    Path::new(rbconfig("ruby_pc").as_str())
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
}

fn is_static() -> bool {
    println!("cargo:rerun-if-env-changed=RUBY_STATIC");

    match env::var("RUBY_STATIC") {
        Ok(val) => val == "true" || val == "1",
        _ => cfg!(feature = "ruby-static"),
    }
}

fn rustc_cfg(name: &str, key: &str) {
    println!("cargo:rustc-cfg={}=\"{}\"", name, rbconfig(key));
}

fn compile_ruby_macros() {
    let mut build = cc::Build::new();
    let mut cc_args = shell_words::split(&rbconfig("CC")).expect("CC is not a valid shell word");
    let libs = shell_words::split(&rbconfig("LIBS")).expect("cannot split LIBS");

    cc_args.reverse();
    build.compiler(cc_args.pop().expect("CC is empty"));
    cc_args.reverse();

    for arg in cc_args {
        build.flag(&arg);
    }

    for lib in libs {
        build.flag(&lib);
    }

    build.file("src/ruby_macros/ruby_macros.c");
    build.include(format!("{}/include/internal", rbconfig("rubyhdrdir")));
    build.include(format!("{}/include/impl", rbconfig("rubyhdrdir")));
    build.include(rbconfig("rubyhdrdir"));
    build.include(rbconfig("rubyarchhdrdir"));
    build.flag("-fms-extensions");
    build.flag("-Wunused-parameter");

    let cflags_str = rbconfig("CFLAGS");
    let rb_cflags = shell_words::split(&cflags_str).expect("failed to parse CFLAGS");

    for flag in rb_cflags {
        build.flag(&flag);
    }

    build.compile("ruby_macros");
}

fn default_bindgen(clang_args: Vec<String>) -> bindgen::Builder {
    bindgen::Builder::default()
        .use_core()
        .ctypes_prefix("::libc")
        .rustified_enum("*")
        .derive_eq(true)
        .derive_debug(true)
        .clang_args(clang_args)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
}

fn write_bindings(builder: bindgen::Builder, path: &str) {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    builder
        .generate()
        .unwrap_or_else(|_| panic!("Unable to generate bindings for {}", path))
        .write_to_file(out_path.join(path))
        .unwrap_or_else(|_| panic!("Couldn't write bindings for {}", path))
}

fn has_ruby_dln_check_abi() -> bool {
    let major = rbconfig("MAJOR").parse::<i32>().unwrap();
    let minor = rbconfig("MINOR").parse::<i32>().unwrap();

    major >= 3 && minor >= 2 && !cfg!(target_family = "windows")
}
