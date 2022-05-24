use super::rbconfig;
use std::env;
use std::path::PathBuf;

pub fn generate() {
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
