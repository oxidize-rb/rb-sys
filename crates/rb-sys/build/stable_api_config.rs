use crate::{c_glue, features::is_env_variable_defined, version::Version};
use std::error::Error;

const LATEST_STABLE_VERSION: Version = Version::new(3, 2);
const MIN_SUPPORTED_STABLE_VERSION: Version = Version::new(2, 6);

#[derive(Debug, Default)]
enum Strategy {
    RustOnly(Version),
    CompiledOnly,
    RustThenCompiled(Version),
    Testing(Version),
    #[default]
    None,
}

impl Strategy {
    fn apply(self) -> Result<(), Box<dyn Error>> {
        match self {
            Strategy::RustOnly(current_ruby_version) => {
                if is_ruby_version_stable(&current_ruby_version) {
                    println!("cargo:rustc-cfg=stable_api_include_rust_impl");
                } else {
                    return Err(format!("A stable Ruby API is needed but could not find a candidate. If you are using a stable version of Ruby, try upgrading rb-sys. Otherwise if you are testing against ruby-head or Ruby < {}, enable the `stable-api-compiled-fallback` feature in rb-sys.", MIN_SUPPORTED_STABLE_VERSION).into());
                }
            }
            Strategy::CompiledOnly => {
                c_glue::compile()?;
                println!("cargo:rustc-cfg=stable_api_enable_compiled_mod");
                println!("cargo:rustc-cfg=stable_api_export_compiled_as_api");
            }
            Strategy::RustThenCompiled(current_ruby_version) => {
                if is_ruby_version_stable(&current_ruby_version) {
                    println!("cargo:rustc-cfg=stable_api_has_rust_impl");
                    println!("cargo:rustc-cfg=stable_api_include_rust_impl");
                } else {
                    c_glue::compile()?;
                    println!("cargo:rustc-cfg=stable_api_enable_compiled_mod");
                    println!("cargo:rustc-cfg=stable_api_export_compiled_as_api");
                }
            }
            Strategy::Testing(current_ruby_version) => {
                c_glue::compile()?;

                println!("cargo:rustc-cfg=stable_api_enable_compiled_mod");

                if is_ruby_version_stable(&current_ruby_version) {
                    println!("cargo:rustc-cfg=stable_api_include_rust_impl");
                } else {
                    println!("cargo:rustc-cfg=stable_api_export_compiled_as_api");
                }
            }
            Strategy::None => {
                return Err("Stable API is needed but could not find a candidate. Try enabling the `stable-api-compiled-fallback` feature in rb-sys.".into());
            }
        };

        Ok(())
    }
}

pub fn configure(current_ruby_version: &Version) -> Result<(), Box<dyn Error>> {
    let mut strategy = Strategy::default();

    if *current_ruby_version < MIN_SUPPORTED_STABLE_VERSION {
        println!(
            "cargo:warning=Support for Ruby {} will be removed in a future release.",
            current_ruby_version
        );
    }

    if is_ruby_version_stable(current_ruby_version) {
        strategy = Strategy::RustOnly(*current_ruby_version);
    }

    if is_fallback_enabled() && !is_ruby_version_stable(current_ruby_version) {
        strategy = Strategy::RustThenCompiled(*current_ruby_version);
    }

    if is_testing() {
        strategy = Strategy::Testing(*current_ruby_version);
    }

    if is_force_enabled() {
        strategy = Strategy::CompiledOnly;
    }

    strategy.apply()?;

    Ok(())
}

fn is_ruby_version_stable(ver: &Version) -> bool {
    *ver >= MIN_SUPPORTED_STABLE_VERSION && *ver <= LATEST_STABLE_VERSION
}

fn is_fallback_enabled() -> bool {
    println!("cargo:rerun-if-env-changed=RB_SYS_USE_STABLE_API_COMPILED_FALLBACK");

    is_env_variable_defined("CARGO_FEATURE_STABLE_API_COMPILED_FALLBACK")
        || cfg!(rb_sys_use_stable_api_compiled_fallback)
        || is_env_variable_defined("RB_SYS_USE_STABLE_API_COMPILED_FALLBACK")
}

fn is_force_enabled() -> bool {
    println!("cargo:rerun-if-env-changed=RB_SYS_USE_STABLE_API_COMPILED_FORCE");

    is_env_variable_defined("CARGO_FEATURE_STABLE_API_COMPILED_FORCE")
        || cfg!(rb_sys_force_stable_api_compiled)
        || is_env_variable_defined("RB_SYS_FORCE_STABLE_API_COMPILED")
}

fn is_testing() -> bool {
    is_env_variable_defined("CARGO_FEATURE_STABLE_API_COMPILED_TESTING")
}
