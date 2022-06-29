//! Helper to define the `ruby_abi_version` function needed for extensions.
//!
//! Since Ruby 3.2, gems are required to define a `ruby_abi_version` function.
//! For C extensions, this is done transparently by including `ruby.h`, but for
//! Rust we have to define it ourselves. This is enabled automatically by the
//! `ruby-abi-versions` Cargo feature flag.

#[doc(hidden)]
#[cfg(not(has_ruby_abi_version))]
pub const __RB_SYS_RUBY_ABI_VERSION: std::os::raw::c_ulonglong = 0;

#[doc(hidden)]
#[cfg(has_ruby_abi_version)]
pub const __RB_SYS_RUBY_ABI_VERSION: std::os::raw::c_ulonglong = crate::RUBY_ABI_VERSION as _;

#[macro_export]
macro_rules! ruby_abi_version {
    () => {
        /// Defines the `ruby_abi_version` function needed for Ruby extensions.
        #[no_mangle]
        #[allow(unused)]
        pub extern "C" fn ruby_abi_version() -> std::os::raw::c_ulonglong {
            $crate::__RB_SYS_RUBY_ABI_VERSION
        }
    };
}
