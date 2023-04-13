//! # `rb-sys-env`
//!
//! Helpers to integrate `rb-sys` into your high-level Ruby bindings library.
//!
//! ## Features
//!
//! - Provides the neccesary Cargo configuration to ensure that Rust crates compile properly across all platforms
//! - Sets useful rustc-cfg flags that you can use from your crate
//! - Exposes all `RbConfig::CONFIG` values from rb-sys
//!
//! ## Usage
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [build-dependencies]
//! rb-sys-env = "0.1"
//! ```
//!
//! Then, in your crate's `build.rs`:
//!
//! ```rust
//! pub fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let _rb_env = rb_sys_env::activate()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Available `rustc-cfg`
//!
//! Here is an example of the `rustc-cfg` flags that are set by this crate:
//!
//! - `#[cfg(ruby_have_ruby_re_h)]`
//! - `#[cfg(ruby_use_rgengc)]`
//! - `#[cfg(ruby_use_symbol_as_method_name)]`
//! - `#[cfg(ruby_have_ruby_util_h)]`
//! - `#[cfg(ruby_have_ruby_oniguruma_h)]`
//! - `#[cfg(ruby_have_ruby_defines_h)]`
//! - `#[cfg(ruby_use_flonum)]`
//! - `#[cfg(ruby_have_ruby_onigmo_h)]`
//! - `#[cfg(ruby_use_unaligned_member_access)]`
//! - `#[cfg(ruby_use_transient_heap)]`
//! - `#[cfg(ruby_have_ruby_atomic_h)]`
//! - `#[cfg(ruby_have_rb_scan_args_optional_hash)]`
//! - `#[cfg(ruby_have_rb_data_type_t_parent)]`
//! - `#[cfg(ruby_have_ruby_debug_h)]`
//! - `#[cfg(ruby_have_ruby_encoding_h)]`
//! - `#[cfg(ruby_have_ruby_ruby_h)]`
//! - `#[cfg(ruby_have_ruby_intern_h)]`
//! - `#[cfg(ruby_use_mjit)]`
//! - `#[cfg(ruby_have_rb_data_type_t_function)]`
//! - `#[cfg(ruby_have_rb_fd_init)]`
//! - `#[cfg(ruby_have_rb_reg_new_str)]`
//! - `#[cfg(ruby_have_rb_io_t)]`
//! - `#[cfg(ruby_have_ruby_memory_view_h)]`
//! - `#[cfg(ruby_have_ruby_version_h)]`
//! - `#[cfg(ruby_have_ruby_st_h)]`
//! - `#[cfg(ruby_have_ruby_thread_native_h)]`
//! - `#[cfg(ruby_have_ruby_random_h)]`
//! - `#[cfg(ruby_have_ruby_regex_h)]`
//! - `#[cfg(ruby_have_rb_define_alloc_func)]`
//! - `#[cfg(ruby_have_ruby_fiber_scheduler_h)]`
//! - `#[cfg(ruby_have_ruby_missing_h)]`
//! - `#[cfg(ruby_have_rb_ext_ractor_safe)]`
//! - `#[cfg(ruby_have_ruby_thread_h)]`
//! - `#[cfg(ruby_have_ruby_vm_h)]`
//! - `#[cfg(ruby_use_rincgc)]`
//! - `#[cfg(ruby_have_ruby_ractor_h)]`
//! - `#[cfg(ruby_have_ruby_io_h)]`
//! - `#[cfg(ruby_3)]`
//! - `#[cfg(ruby_3_1)]`
//! - `#[cfg(ruby_3_1_2)]`
//! - `#[cfg(ruby_gte_2_7)]`
//! - `#[cfg(ruby_gt_2_7)]`
//! - `#[cfg(ruby_gte_3_0)]`
//! - `#[cfg(ruby_gt_3_0)]`
//! - `#[cfg(ruby_lte_3_1)]`
//! - `#[cfg(ruby_3_1)]`
//! - `#[cfg(ruby_eq_3_1)]`
//! - `#[cfg(ruby_gte_3_1)]`
//! - `#[cfg(ruby_lt_3_2)]`
//! - `#[cfg(ruby_lte_3_2)]`
//! - `#[cfg(ruby_lt_3_3)]`
//! - `#[cfg(ruby_lte_3_3)]`
//! - `#[cfg(ruby_gte_1)]`
//! - `#[cfg(ruby_gt_1)]`
//! - `#[cfg(ruby_gte_2)]`
//! - `#[cfg(ruby_gt_2)]`
//! - `#[cfg(ruby_lte_3)]`
//! - `#[cfg(ruby_3)]`
//! - `#[cfg(ruby_eq_3)]`
//! - `#[cfg(ruby_gte_3)]`
//! - `#[cfg(ruby_lt_4)]`
//! - `#[cfg(ruby_lte_4)]`

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
