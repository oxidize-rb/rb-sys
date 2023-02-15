/// Raw OS types for the `rb-sys` crate.
pub mod raw {
    pub use libc::{mode_t, off_t, pid_t, size_t, ssize_t, stat, suseconds_t, time_t};
}
