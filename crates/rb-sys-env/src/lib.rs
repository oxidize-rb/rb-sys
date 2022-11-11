#![doc = include_str!("../readme.md")]

#[macro_use]
mod utils;
mod defines;
mod rb_env;
mod ruby_version;

use std::error::Error;

pub use defines::Defines;
pub use rb_env::RbEnv;
pub use ruby_version::RubyVersion;

/// Configures Cargo linking based on the `DEP_RB_*` environment variables. This
/// is needed to ensure that Cargo properly links to libruby on Windows..
///
/// ```should_panic
/// // In your crate's build.rs
///
/// pub fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let rb_env = rb_sys_env::activate()?;
///
///     if rb_env.ruby_major_minor() < (2, 7) {
///         panic!("Your Ruby version is EOL!");
///     }
///
///     Ok(())
/// }
///
/// // In your crate's lib.rs
/// pub fn is_ruby_flonum_activated() -> bool {
///     cfg!(ruby_use_flonum)
/// }
/// ```
pub fn activate() -> Result<RbEnv, Box<dyn Error>> {
    let env = RbEnv::default();

    env.print_cargo_rerun_if_changed();
    env.print_cargo_rustc_cfg();
    env.print_encoded_cargo_args();

    if std::env::var_os("RB_SYS_ENV_DEBUG").is_some() {
        eprintln!("=======================");
        eprintln!("The \"RB_SYS_ENV_DEBUG\" env var was detecting, aborted build.");
        std::process::exit(1);
    }

    Ok(env)
}

/// Loads the `DEP_RB_*` environment variables, without setting Cargo configuration.
///
/// *Note*: This will not activate the `rb-sys` crate's features to ensure Ruby is properly linked.
///
/// ```
/// // In your crate's build.rs
///
/// pub fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let rb_env = rb_sys_env::load()?;
///
///     rb_env.print_cargo_rerun_if_changed();
///     rb_env.print_cargo_rustc_cfg();
///     rb_env.print_encoded_cargo_args();
///
///     if rb_env.ruby_major_minor() < (2, 7) {
///         panic!("Your Ruby version is EOL!");
///     }
///
///     Ok(())
/// }
/// ```
pub fn load() -> Result<RbEnv, Box<dyn Error>> {
    let env = RbEnv::default();

    Ok(env)
}
