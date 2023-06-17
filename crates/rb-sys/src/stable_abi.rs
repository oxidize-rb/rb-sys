//! Stable ABI functions which provide access to Ruby internals that
//! is compatible across Ruby versions, and are guaranteed to be not break due
//! to Ruby binary changes.
//!
//! ### Goals
//!
//! 1. To provide access to Ruby internals that are not exposed by the libruby
//!    (i.e. C macros and inline functions).
//! 2. Provide support for Ruby development versions, which can make breaking
//!    changes without semantic versioning. We want to support these versions
//!    to ensure Rust extensions don't prevent the Ruby core team from testing
//!    changes in production.

use crate::VALUE;
use std::ffi::{c_char, c_long};

pub trait StableAbiDefinition {
    /// Get the length of a Ruby string (akin to `RSTRING_LEN`).
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying Ruby data. The caller must ensure that the pointer
    /// is valid.
    unsafe fn rstring_len(obj: VALUE) -> c_long;

    /// Get a pointer to the bytes of a Ruby string (akin to `RSTRING_PTR`).
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying Ruby data. The caller must ensure that the pointer
    /// is valid.
    unsafe fn rstring_ptr(obj: VALUE) -> *const c_char;

    /// Get the length of a Ruby array (akin to `RARRAY_LEN`).
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying Ruby data. The caller must ensure that the pointer
    /// is valid.
    unsafe fn rarray_len(obj: VALUE) -> c_long;

    /// Get a pointer to the elements of a Ruby array (akin to `RARRAY_CONST_PTR`).
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying Ruby data. The caller must ensure that the pointer
    /// is valid.
    unsafe fn rarray_const_ptr(obj: VALUE) -> *const VALUE;
}

#[cfg(any(not(ruby_abi_stable), feature = "stable-abi-compiled"))]
mod compiled;

#[cfg(ruby_eq_2_6)]
#[path = "stable_abi/ruby_2_6.rs"]
mod abi;

#[cfg(ruby_eq_2_7)]
#[path = "stable_abi/ruby_2_7.rs"]
mod abi;

#[cfg(ruby_eq_3_0)]
#[path = "stable_abi/ruby_3_0.rs"]
mod abi;

#[cfg(ruby_eq_3_1)]
#[path = "stable_abi/ruby_3_1.rs"]
mod abi;

#[cfg(ruby_eq_3_2)]
#[path = "stable_abi/ruby_3_2.rs"]
mod abi;

#[cfg(any(not(ruby_abi_stable), ruby_lt_2_6))]
use compiled as abi;

pub use abi::Definition as StableAbi;

#[cfg(feature = "stable-abi-compiled")]
pub use compiled::Definition as Compiled;

#[cfg(test)]
mod tests;
