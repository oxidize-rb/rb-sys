//! Zig cross-compilation support for rb-sys.
//!
//! This module provides the core functionality for using Zig as a cross-compiler
//! for building Ruby native extensions. It handles:
//!
//! - Target triple translation (Rust → Zig format)
//! - CPU feature flag selection
//! - Compiler/linker argument filtering and rewriting
//! - Bash shim generation for CC/CXX/AR/LD
//! - Environment variable setup for Cargo

pub mod ar;
pub mod args;
pub mod cc;
pub mod cpu;
pub mod env;
pub mod ld;
pub mod shim;
pub mod target;

// Re-exports for external use (used by build.rs and main.rs)
#[allow(unused_imports)]
pub use shim::generate_shims;
#[allow(unused_imports)]
pub use target::RustTarget;
