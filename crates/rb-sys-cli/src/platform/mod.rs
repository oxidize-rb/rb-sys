//! Platform-specific configuration for cross-compilation.
//!
//! Each platform (Linux, macOS, Windows) has different requirements for
//! cross-compilation with Zig. This module provides the configuration
//! logic for each platform.

pub mod linux;
pub mod macos;
pub mod windows;

pub use linux::LinuxConfig;
pub use macos::MacOSConfig;
pub use windows::WindowsConfig;
