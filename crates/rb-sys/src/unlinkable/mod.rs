mod embeddable;

#[cfg(all(compiled_c_impls_available))]
pub mod compiled_c_impls;

#[cfg(any(ruby_abi_stable, feature = "bypass-stable-abi-version-checks"))]
pub mod rust_impls;

pub(crate) mod impls {
    #[cfg(all(compiled_c_impls_available, not(ruby_abi_stable),))]
    pub(crate) use super::compiled_c_impls::*;

    #[cfg(any(ruby_abi_stable))]
    pub(crate) use super::rust_impls::*;
}

pub(crate) use impls::{rarray_const_ptr, rarray_len, rstring_len, rstring_ptr};
