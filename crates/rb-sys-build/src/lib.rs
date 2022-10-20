pub mod bindings;
mod rb_config;
pub mod utils;

pub use rb_config::*;

/// The current RbConfig.
pub fn rb_config() -> RbConfig {
    RbConfig::current()
}
