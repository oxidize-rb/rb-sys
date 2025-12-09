#![allow(rustdoc::bare_urls)]
#![doc = include_str!("../readme.md")]

pub mod bindings;
#[cfg(feature = "stable-api")]
pub mod macros;
pub mod memory;
pub mod special_consts;
#[cfg(feature = "stable-api")]
pub mod stable_api;
pub mod symbol;
pub mod tracking_allocator;
pub mod value_type;

mod hidden;
mod ruby_abi_version;
mod utils;

pub use bindings::*;
#[cfg(feature = "stable-api")]
pub use macros::*;
pub use ruby_abi_version::*;
pub use special_consts::*;
#[cfg(feature = "stable-api")]
pub use stable_api::StableApiDefinition;
pub use value_type::*;

#[deprecated(since = "0.9.79", note = "Use `VALUE` instead")]
pub type Value = VALUE;
#[deprecated(since = "0.9.79", note = "Use `VALUE` instead")]
pub type RubyValue = VALUE;

// Compatibility module for magnus and other crates that expect rbimpl_typeddata_flags as a module
// instead of an enum. The embedded bindings don't include this enum, so we provide fallback constants.
#[cfg(ruby_gte_3_0)]
pub mod rbimpl_typeddata_flags {
    // These constants match the values defined in Ruby's include/ruby/internal/core/rtypeddata.h
    pub const RUBY_TYPED_FREE_IMMEDIATELY: u32 = 1;
    pub const RUBY_TYPED_EMBEDDABLE: u32 = 2;
    pub const RUBY_TYPED_WB_PROTECTED: u32 = 32;
    pub const RUBY_TYPED_UNUSED: u32 = 64;
    pub const RUBY_TYPED_FROZEN_SHAREABLE: u32 = 256;
    pub const RUBY_TYPED_DECL_MARKING: u32 = 16384;
}

#[cfg(use_global_allocator)]
set_global_tracking_allocator!();
