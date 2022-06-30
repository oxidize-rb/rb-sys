mod rb_config;

pub use rb_config::*;

/// Prints out the default cargo args for the current Ruby version.
pub fn print_cargo_args_for_rb_config() {
    RbConfig::current().print_cargo_args();
}
