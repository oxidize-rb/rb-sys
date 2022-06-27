use std::{collections::HashMap, env, process::Command};

use regex::Regex;
mod flags;
mod library;
mod search_path;

use library::*;
use search_path::*;
use serde_json::{Map, Value};
use std::ffi::OsString;

use self::flags::Flags;

/// Extracts structured information from raw compiler/linker flags to make
/// compiling Ruby gems easier.
#[derive(Debug, PartialEq, Eq)]
pub struct RbConfig {
    pub search_paths: Vec<SearchPath>,
    pub libs: Vec<Library>,
    pub link_args: Vec<String>,
    pub cflags: Vec<String>,
    pub blocklist_lib: Vec<String>,
    value_map: HashMap<String, Value>,
}

impl Default for RbConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl RbConfig {
    /// Creates a new, blank `RbConfig`. You likely want to use `RbConfig::current()` instead.
    pub fn new() -> RbConfig {
        RbConfig {
            blocklist_lib: vec![],
            search_paths: Vec::new(),
            libs: Vec::new(),
            link_args: Vec::new(),
            cflags: Vec::new(),
            value_map: HashMap::new(),
        }
    }

    /// Sets a value for a key
    pub fn set_value_for_key(&mut self, key: &str, value: Value) {
        self.value_map.insert(key.to_owned(), value);
    }

    /// Get the name for libruby-static (i.e. `ruby.3.1-static`)
    pub fn libruby_static_name(&self) -> String {
        self.get("LIBRUBY_A")
            .strip_prefix("lib")
            .unwrap()
            .strip_suffix(".a")
            .unwrap()
            .to_string()
    }

    /// Get the name for libruby (i.e. `ruby.3.1`)
    pub fn libruby_so_name(&self) -> String {
        self.get("RUBY_SO_NAME")
    }

    /// Instantiates a new `RbConfig` for the current Ruby.
    pub fn current() -> RbConfig {
        println!("cargo:rerun-if-env-changed=RUBY");

        let mut rbconfig = RbConfig::new();
        let ruby = env::var_os("RUBY").unwrap_or_else(|| OsString::from("ruby"));

        let config = Command::new(ruby)
            .arg("--disable-gems")
            .arg("-rrbconfig")
            .arg("-rjson")
            .arg("-e")
            .arg("print RbConfig::CONFIG.to_json")
            .output()
            .unwrap_or_else(|e| panic!("ruby not found: {}", e));

        let output = String::from_utf8(config.stdout).expect("RbConfig value not UTF-8!");
        let parsed: serde_json::Value =
            serde_json::from_str(&output).expect("Could not parse RbConfig output");
        let value_map: Map<String, Value> = parsed.as_object().unwrap().clone();
        let cflags = value_map.get("cflags").unwrap().as_str().unwrap();
        let ldflags = value_map.get("DLDFLAGS").unwrap().as_str().unwrap();

        rbconfig.push_cflags(cflags);
        rbconfig.push_dldflags(ldflags);
        let mut hash_map = HashMap::new();

        for (key, value) in value_map {
            hash_map.insert(key, value);
        }

        rbconfig.value_map = hash_map;

        rbconfig
    }

    /// Filter the libs, removing the ones that are not needed.
    pub fn blocklist_lib(&mut self, name: &str) -> &mut RbConfig {
        self.blocklist_lib.push(name.to_string());
        self
    }

    /// Returns the current ruby version.
    pub fn ruby_version(&self) -> String {
        self.get("ruby_version")
    }

    /// Returns the value of the given key from the either the matching
    /// `RBCONFIG_{key}` environment variable or `RbConfig::CONFIG[{key}]` hash.
    pub fn get(&self, key: &str) -> String {
        println!("cargo:rerun-if-env-changed=RBCONFIG_{}", key);

        match env::var(format!("RBCONFIG_{}", key)) {
            Ok(val) => val,
            _ => self
                .value_map
                .get(key)
                .expect("key not found")
                .as_str()
                .unwrap()
                .to_owned(),
        }
    }

    /// Returns the value of the given key from the either the matching
    /// `RBCONFIG_{key}` environment variable or `RbConfig::CONFIG[{key}]` hash.
    pub fn get_optional(&self, key: &str) -> Option<String> {
        println!("cargo:rerun-if-env-changed=RBCONFIG_{}", key);

        match env::var(format!("RBCONFIG_{}", key)) {
            Ok(val) => Some(val),
            _ => self
                .value_map
                .get(key)
                .map(|val| val.as_str())
                .map(|val| val.unwrap().to_owned()),
        }
    }

    /// Push cflags string
    pub fn push_cflags(&mut self, cflags: &str) -> &mut Self {
        shell_words::split(cflags)
            .expect("cannot split cflags")
            .iter()
            .for_each(|cflag| {
                self.cflags.push(cflag.to_owned());
            });
        self
    }

    /// Get the rb_config output for cargo
    pub fn cargo_args(&self) -> Vec<String> {
        let mut result = vec![];

        for search_path in &self.search_paths {
            result.push(format!("cargo:rustc-link-search={}", search_path));
        }

        for lib in &self.libs {
            if !self.blocklist_lib.iter().any(|b| lib.name.contains(b)) {
                result.push(format!("cargo:rustc-link-lib={}", lib));
            }
        }

        for link_arg in &self.link_args {
            result.push(format!("cargo:rustc-link-arg={}", link_arg));
        }

        result
    }

    /// Print to rb_config output for cargo
    pub fn print_cargo_args(&self) {
        for arg in self.cargo_args() {
            println!("{}", arg);
        }
    }

    /// Adds items to the rb_config based on a string from LDFLAGS/DLDFLAGS
    pub fn push_dldflags(&mut self, input: &str) -> &mut Self {
        let split_args = Flags::new(input);

        let search_path_regex = Regex::new(r"^-L\s*(?P<name>.*)$").unwrap();
        let libruby_regex = Regex::new(r"^-l\s*ruby(?P<name>\S+)$").unwrap();
        let lib_regex_short = Regex::new(r"^-l\s*(?P<name>\w+\S+)$").unwrap();
        let lib_regex_long = Regex::new(r"^--library=(?P<name>\w+\S+)$").unwrap();
        let static_lib_regex = Regex::new(r"^-l\s*:lib(?P<name>\S+).a$").unwrap();
        let dynamic_lib_regex = Regex::new(r"^-l\s*:lib(?P<name>\S+).(so|dylib|dll)$").unwrap();
        let framework_regex_short = Regex::new(r"^-F\s*(?P<name>.*)$").unwrap();
        let framework_regex_long = Regex::new(r"^-framework\s*(?P<name>.*)$").unwrap();

        for arg in split_args {
            let arg = self.subst_shell_variables(arg);

            if let Some(name) = capture_name(&search_path_regex, &arg) {
                self.search_paths.push(SearchPath {
                    kind: SearchPathKind::Native,
                    name,
                });
            } else if let Some(name) = capture_name(&libruby_regex, &arg) {
                let (kind, modifiers) = if name.contains("static") {
                    (LibraryKind::Static, vec!["+whole-archive".to_string()])
                } else {
                    (LibraryKind::Dylib, vec![])
                };

                self.libs.push(Library {
                    kind,
                    name: format!("ruby{}", name.to_owned()),
                    rename: Some("rb".to_string()),
                    modifiers,
                });
            } else if let Some(name) = capture_name(&lib_regex_long, &arg) {
                self.libs.push(Library {
                    kind: LibraryKind::Native,
                    name: name.to_owned(),
                    rename: None,
                    modifiers: vec![],
                });
            } else if let Some(name) = capture_name(&lib_regex_short, &arg) {
                self.libs.push(Library {
                    kind: LibraryKind::Native,
                    name: name.to_owned(),
                    rename: None,
                    modifiers: vec![],
                });
            } else if let Some(name) = capture_name(&static_lib_regex, &arg) {
                self.libs.push(Library {
                    kind: LibraryKind::Static,
                    name: name.to_owned(),
                    rename: None,
                    modifiers: vec![],
                });
            } else if let Some(name) = capture_name(&dynamic_lib_regex, &arg) {
                self.libs.push(Library {
                    kind: LibraryKind::Dylib,
                    name: name.to_owned(),
                    rename: None,
                    modifiers: vec![],
                });
            } else if let Some(name) = capture_name(&static_lib_regex, &arg) {
                self.libs.push(Library {
                    kind: LibraryKind::Static,
                    name: name.to_owned(),
                    rename: None,
                    modifiers: vec![],
                });
            } else if let Some(name) = capture_name(&dynamic_lib_regex, &arg) {
                self.libs.push(Library {
                    kind: LibraryKind::Dylib,
                    name: name.to_owned(),
                    rename: None,
                    modifiers: vec![],
                });
            } else if let Some(name) = capture_name(&framework_regex_short, &arg) {
                self.search_paths.push(SearchPath {
                    kind: SearchPathKind::Framework,
                    name: name.to_owned(),
                });
            } else if let Some(name) = capture_name(&framework_regex_long, &arg) {
                self.libs.push(Library {
                    kind: LibraryKind::Framework,
                    name: name.to_owned(),
                    rename: None,
                    modifiers: vec![],
                });
            } else {
                self.link_args.push(arg.to_owned());
            }
        }

        self
    }

    // Examines the string from shell variables and expands them with values in the value_map
    fn subst_shell_variables(&self, input: &str) -> String {
        let mut result = String::new();
        let mut chars = input.chars();

        while let Some(c) = chars.next() {
            if c == '$' {
                if let Some(c) = chars.next() {
                    if c == '(' {
                        let mut key = String::new();

                        for c in chars.by_ref() {
                            if c == ')' {
                                break;
                            } else {
                                key.push(c);
                            }
                        }

                        if let Some(val) = self.get_optional(&key) {
                            result.push_str(&val);
                        } else {
                            // Consume whitespace
                            chars.next();
                        }
                    }
                }
            } else {
                result.push(c);
            }
        }

        result
    }
}

fn capture_name(regex: &Regex, arg: &str) -> Option<String> {
    regex
        .captures(arg)
        .map(|cap| cap.name("name").unwrap().as_str().trim().to_owned())
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    #[test]
    fn test_extract_lib_search_paths() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("-L/usr/local/lib -L/usr/lib");
        assert_eq!(
            rb_config.search_paths,
            vec!["/usr/local/lib".into(), "/usr/lib".into()]
        );
    }

    #[test]
    fn test_search_path_basic() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("-L/usr/local/lib");

        assert_eq!(rb_config.search_paths, vec!["native=/usr/local/lib".into()]);
    }

    #[test]
    fn test_search_path_space() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("-L /usr/local/lib");

        assert_eq!(rb_config.search_paths, vec!["/usr/local/lib".into()]);
    }

    #[test]
    fn test_search_path_space_in_path() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("-L/usr/local/my lib");

        assert_eq!(
            rb_config.search_paths,
            vec!["native=/usr/local/my lib".into()]
        );
    }

    #[test]
    fn test_simple_lib() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("-lfoo");

        assert_eq!(rb_config.libs, ["foo".into()]);
    }

    #[test]
    fn test_lib_with_nonascii() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("-lws2_32");

        assert_eq!(rb_config.libs, ["ws2_32".into()]);
    }

    #[test]
    fn test_simple_lib_space() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("-l foo");

        assert_eq!(rb_config.libs, ["foo".into()]);
    }

    #[test]
    fn test_verbose_lib_space() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("--library=foo");

        assert_eq!(rb_config.libs, ["foo".into()]);
    }

    #[test]
    fn test_libstatic_with_colon() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("-l:libssp.a");

        assert_eq!(rb_config.libs, ["static=ssp".into()]);
    }

    #[test]
    fn test_libstatic_with_colon_space() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("-l :libssp.a");

        assert_eq!(rb_config.libs, ["static=ssp".into()]);
    }

    #[test]
    fn test_unconventional_lib_with_colon() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("-l:ssp.a");

        assert_eq!(rb_config.link_args, vec!["-l:ssp.a"]);
    }

    #[test]
    fn test_dylib_with_colon_space() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("-l :libssp.dylib");

        assert_eq!(rb_config.libs, ["dylib=ssp".into()]);
    }

    #[test]
    fn test_so_with_colon_space() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("-l :libssp.so");

        assert_eq!(rb_config.libs, ["dylib=ssp".into()]);
    }

    #[test]
    fn test_dll_with_colon_space() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("-l :libssp.dll");

        assert_eq!(rb_config.libs, ["dylib=ssp".into()]);
    }

    #[test]
    fn test_framework() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("-F/some/path");

        assert_eq!(rb_config.search_paths, ["framework=/some/path".into()]);
    }

    #[test]
    fn test_framework_space() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("-F /some/path");

        assert_eq!(
            rb_config.search_paths,
            [SearchPath {
                kind: SearchPathKind::Framework,
                name: "/some/path".into(),
            }]
        );
    }

    #[test]
    fn test_framework_arg_real() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("-framework CoreFoundation");

        assert_eq!(
            rb_config.libs,
            [Library {
                kind: LibraryKind::Framework,
                name: "CoreFoundation".into(),
                rename: None,
                modifiers: vec![],
            }]
        );
    }

    #[test]
    fn test_libruby_static() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("-lruby.3.1-static");

        assert_eq!(
            rb_config.cargo_args(),
            ["cargo:rustc-link-lib=static:+whole-archive=rb:ruby.3.1-static"]
        );
    }

    #[test]
    fn test_libruby_dynamic() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("-lruby.3.1");

        assert_eq!(
            rb_config.cargo_args(),
            ["cargo:rustc-link-lib=dylib=rb:ruby.3.1"]
        );
    }

    #[test]
    fn test_non_lib_dash_l() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("test_rubygems_20220413-976-lemgf9/prefix");

        assert_eq!(
            rb_config.link_args,
            vec!["test_rubygems_20220413-976-lemgf9/prefix"]
        );
    }

    #[test]
    fn test_real_dldflags() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("-L/Users/ianks/.asdf/installs/ruby/3.1.1/lib -L/opt/homebrew/opt/openssl@1.1/lib -Wl,-undefined,dynamic_lookup -Wl,-multiply_defined,suppress");

        assert_eq!(
            rb_config.link_args,
            vec![
                "-Wl,-undefined,dynamic_lookup",
                "-Wl,-multiply_defined,suppress"
            ]
        );
        assert_eq!(
            rb_config.search_paths,
            vec![
                SearchPath {
                    kind: SearchPathKind::Native,
                    name: "/Users/ianks/.asdf/installs/ruby/3.1.1/lib".to_string()
                },
                SearchPath {
                    kind: SearchPathKind::Native,
                    name: "/opt/homebrew/opt/openssl@1.1/lib".to_string()
                },
            ]
        );
    }

    #[test]
    fn test_crazy_cases() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("-F   /something -l:libssp.a -static-libgcc ");

        assert_eq!(rb_config.link_args, vec!["-static-libgcc"]);
        assert_eq!(
            rb_config.libs,
            vec![Library {
                kind: LibraryKind::Static,
                name: "ssp".to_string(),
                rename: None,
                modifiers: vec![],
            }]
        );
        assert_eq!(
            rb_config.search_paths,
            vec![SearchPath {
                kind: SearchPathKind::Framework,
                name: "/something".to_string()
            },]
        );
    }

    #[test]
    fn test_printing_cargo_args() {
        let mut rb_config = RbConfig::new();
        rb_config.push_dldflags("-L/Users/ianks/.asdf/installs/ruby/3.1.1/lib");
        rb_config.push_dldflags("-lfoo");
        rb_config.push_dldflags("-static-libgcc");
        let result = rb_config.cargo_args();

        assert_eq!(
            vec![
                "cargo:rustc-link-search=native=/Users/ianks/.asdf/installs/ruby/3.1.1/lib",
                "cargo:rustc-link-lib=foo",
                "cargo:rustc-link-arg=-static-libgcc"
            ],
            result
        );
    }

    #[test]
    fn test_variable_subst() {
        let mut rb_config = RbConfig::new();
        rb_config.set_value_for_key("DEFFILE", "some.def".into());

        rb_config.push_dldflags("--enable-auto-import $(DEFFILE) foo");
        let result = rb_config.cargo_args();

        assert_eq!(
            vec!["cargo:rustc-link-arg=--enable-auto-import some.def foo"],
            result
        );
    }

    #[test]
    fn test_variable_subst_unknown_var() {
        let mut rb_config = RbConfig::new();

        rb_config.push_dldflags("--enable-auto-import $(DEFFILE) foo");
        let result = rb_config.cargo_args();

        assert_eq!(
            vec!["cargo:rustc-link-arg=--enable-auto-import foo"],
            result
        );
    }
}
