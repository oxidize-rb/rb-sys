use crate::{Defines, RubyVersion};
use std::{collections::HashMap, rc::Rc};

const ENV_PREFIX: &str = "DEP_RB_";
const RBCONFIG_PREFIX: &str = "RBCONFIG_";
const CARGO_FEATURE_PREFIX: &str = "CARGO_FEATURE_";

/// Information about the rb-sys environment.
#[derive(Debug, Clone)]
pub struct RbEnv {
    defines: Defines,
    vars: Rc<HashMap<String, String>>,
}

impl RbEnv {
    /// The current Ruby version.
    pub fn ruby_version(&self) -> RubyVersion {
        RubyVersion::from_raw_environment(&self.vars)
    }

    /// The (major, minor) tuple of the current Ruby version.
    pub fn ruby_major_minor(&self) -> (u8, u8) {
        self.ruby_version().major_minor()
    }

    /// Get a value from the current Ruby's `RbConfig::CONFIG`.
    pub fn get_rbconfig_value(&self, key: &str) -> Option<&str> {
        self.vars
            .get(&format!("{}{}", RBCONFIG_PREFIX, key))
            .map(|v| v.as_str())
    }

    /// List the Cargo features of rb-sys
    pub fn cargo_features(&self) -> Vec<String> {
        let keys = self.vars.keys();
        let keys = keys.filter(|k| k.starts_with(CARGO_FEATURE_PREFIX));
        let keys = keys.map(|k| k.trim_start_matches(CARGO_FEATURE_PREFIX));

        keys.map(|k| k.replace('_', "-").to_lowercase()).collect()
    }

    /// Tell Cargo to link to libruby, even if `rb-sys` decided not to.
    pub fn force_link_ruby(self) -> Self {
        let libdir = self.vars.get("LIBDIR").expect("DEP_RB_LIBDIR is not set");
        let lib = self.vars.get("LIB").expect("DEP_RB_LIB is not set");

        println!("cargo:rustc-link-search=native={}", libdir);
        println!("cargo:rustc-link-lib={}", lib);

        self
    }

    // Decodes the cargo args from `DEP_RB_ENCODED_CARGO_ARGS` environment variable.
    pub fn encoded_cargo_args(&self) -> Vec<String> {
        if let Some(raw_args) = self.vars.get("ENCODED_CARGO_ARGS") {
            let lines = raw_args.split('\x1E');
            let unescaped = lines.map(|line| line.replace('\x1F', "\n"));
            unescaped.collect()
        } else {
            vec![]
        }
    }

    /// Indicates if we are using libruby-static.
    pub fn is_ruby_static(&self) -> bool {
        self.vars
            .get("RUBY_STATIC")
            .map(|v| v == "true")
            .unwrap_or(false)
    }

    /// Prints args for rustc (i.e. `cargo:rustc-cfg=...`).
    pub fn print_cargo_rustc_cfg(&self) {
        self.defines.print_cargo_rustc_cfg();
        self.ruby_version().print_cargo_rustc_cfg();
    }

    /// Prints directives for rustc (i.e. `cargo:rustc-link-lib=...`).
    pub fn print_encoded_cargo_args(&self) {
        for line in self.encoded_cargo_args() {
            println!("{}", line);
        }
    }

    /// Prints directives for re-runs (i.e. `cargo:rerun-if-env-changed=...`)
    pub fn print_cargo_rerun_if_changed(&self) {
        for key in self.vars.keys() {
            println!("cargo:rerun-if-env-changed={}{}", ENV_PREFIX, key);
        }

        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-env-changed=RB_SYS_ENV_DEBUG");
        println!("cargo:rerun-if-env-changed=RUBY");
        println!("cargo:rerun-if-env-changed=RUBY_ROOT");
        println!("cargo:rerun-if-env-changed=RUBY_VERSION");
    }
}

impl Default for RbEnv {
    fn default() -> Self {
        let vars = std::env::vars();
        let vars = vars.filter(|(key, _)| key.starts_with(ENV_PREFIX));
        let vars = vars.map(|(key, value)| (key.trim_start_matches(ENV_PREFIX).to_string(), value));
        let vars: HashMap<String, String> = vars.collect();
        let vars = Rc::new(vars);
        let defines = Defines::from_raw_environment(vars.clone());

        Self { defines, vars }
    }
}
