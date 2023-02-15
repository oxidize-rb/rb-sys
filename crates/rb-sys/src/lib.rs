pub mod bindings;
#[cfg(feature = "ruby-macros")]
pub mod macros;
pub mod os;
pub mod special_consts;
pub mod value_type;

#[cfg(use_global_allocator)]
mod allocator;
mod ruby_abi_version;

#[cfg(use_global_allocator)]
pub use allocator::*;
pub use bindings::*;
pub use ruby_abi_version::*;
pub use special_consts::*;
pub use value_type::*;

pub type Value = VALUE;
pub type RubyValue = VALUE;

#[cfg(use_global_allocator)]
ruby_global_allocator!();

#[cfg(use_ruby_abi_version)]
ruby_abi_version!();
