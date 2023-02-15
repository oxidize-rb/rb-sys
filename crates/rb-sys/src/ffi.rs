//! Raw OS types for the `rb-sys` crate. This module is not intended to be used
//! directly, but to make bindings more uniform across platforms.

pub use libc::{off_t, size_t, ssize_t, stat, time_t};

#[cfg(unix)]
pub use libc::{mode_t, pid_t, suseconds_t};
