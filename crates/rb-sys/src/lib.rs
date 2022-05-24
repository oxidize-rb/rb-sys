#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unknown_lints)]
#![allow(deref_nullptr)]
#![warn(unknown_lints)]
#![allow(unaligned_references)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(feature = "ruby-macros")]
mod ruby_macros;

#[cfg(feature = "ruby-macros")]
pub mod macros {
    pub use crate::ruby_macros::*;
}

pub mod special_consts;

pub use special_consts::*;

pub type Value = VALUE;

pub type RubyValue = VALUE;

#[cfg(ruby_dln_check_abi)]
#[macro_export]
macro_rules! ruby_extension {
    () => {
        #[no_mangle]
        #[allow(unused)]
        pub extern "C" fn ruby_abi_version() -> std::os::raw::c_ulonglong {
            use $crate::RUBY_ABI_VERSION;

            RUBY_ABI_VERSION.into()
        }
    };
}

#[cfg(not(ruby_dln_check_abi))]
#[macro_export]
macro_rules! ruby_extension {
    () => {
        #[no_mangle]
        #[allow(unused)]
        pub extern "C" fn ruby_abi_version() -> std::os::raw::c_ulonglong {
            0
        }
    };
}
