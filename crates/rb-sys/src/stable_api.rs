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
use std::{
    os::raw::{c_char, c_long},
    ptr::NonNull,
    time::Duration,
};

pub trait StableApiDefinition {
    const VERSION_MAJOR: u32;
    const VERSION_MINOR: u32;

    fn version(&self) -> (u32, u32) {
        (Self::VERSION_MAJOR, Self::VERSION_MINOR)
    }

    /// Get the length of a Ruby string (akin to `RSTRING_LEN`).
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying Ruby data. The caller must ensure that the pointer
    /// is valid.
    unsafe fn rstring_len(&self, obj: VALUE) -> c_long;

    /// Get a pointer to the bytes of a Ruby string (akin to `RSTRING_PTR`).
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying Ruby data. The caller must ensure that the pointer
    /// is valid.
    unsafe fn rstring_ptr(&self, obj: VALUE) -> *const c_char;

    /// Get the length of a Ruby array (akin to `RARRAY_LEN`).
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying Ruby data. The caller must ensure that the pointer
    /// is valid.
    unsafe fn rarray_len(&self, obj: VALUE) -> c_long;

    /// Get a pointer to the elements of a Ruby array (akin to `RARRAY_CONST_PTR`).
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying Ruby data. The caller must ensure that the pointer
    /// is valid.
    unsafe fn rarray_const_ptr(&self, obj: VALUE) -> *const VALUE;

    /// Get element from array by index (akin to `RARRAY_AREF`).
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying Ruby data. The caller must ensure that the pointer
    /// is valid and that the index is within bounds.
    unsafe fn rarray_aref(&self, obj: VALUE, idx: isize) -> VALUE;

    /// Set element in array by index (akin to `RARRAY_ASET`).
    ///
    /// Includes GC write barrier for safety.
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying Ruby data. The caller must ensure that the pointer
    /// is valid and that the index is within bounds.
    unsafe fn rarray_aset(&self, obj: VALUE, idx: isize, val: VALUE);

    /// Get the class from a VALUE which contains an RBasic struct.
    ///
    /// `VALUE` is a valid pointer to a non-immediate object.
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying RBasic struct. The caller must ensure that the
    /// `VALUE` is a valid pointer to an RBasic struct.
    unsafe fn rbasic_class(&self, obj: VALUE) -> Option<NonNull<VALUE>>;

    /// Checks if the given object is frozen.
    ///
    /// `VALUE` is a valid pointer to a non-immediate object.
    ///
    /// # Safety
    /// This function is unsafe because it may dereference a raw pointer to get
    /// access to underlying RBasic struct. The caller must ensure that the
    /// `VALUE` is a valid pointer to an RBasic struct.
    unsafe fn frozen_p(&self, obj: VALUE) -> bool;

    /// Tests if a bignum is positive.
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying RBasic struct. The caller must ensure that the
    /// `VALUE` is a valid pointer to a bignum.
    unsafe fn bignum_positive_p(&self, obj: VALUE) -> bool;

    /// Tests if a bignum is negative.
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying RBasic struct. The caller must ensure that the
    /// `VALUE` is a valid pointer to a bignum.
    #[inline]
    unsafe fn bignum_negative_p(&self, obj: VALUE) -> bool {
        !self.bignum_positive_p(obj)
    }

    /// Tests if the given value is a special constant.
    fn special_const_p(&self, value: VALUE) -> bool;

    /// Queries the type of the object.
    ///
    /// # Note
    /// The input `obj` must not be a special constant.
    ///
    /// # Safety
    /// This function is unsafe because it could dereference a raw pointer when
    /// attemping to access the underlying [`RBasic`] struct.
    unsafe fn builtin_type(&self, obj: VALUE) -> crate::ruby_value_type;

    /// Tests if the object's type is the given type.
    ///
    /// # Safety
    /// This function is unsafe because it could dereference a raw pointer when
    /// attemping to access the underlying [`RBasic`] struct.
    unsafe fn type_p(&self, obj: VALUE, ty: crate::ruby_value_type) -> bool;

    /// Checks if the given object is nil.
    fn nil_p(&self, obj: VALUE) -> bool;

    /// Checks if the given object is a so-called Fixnum.
    fn fixnum_p(&self, obj: VALUE) -> bool;

    /// Checks if the given object is a dynamic symbol.
    ///
    /// # Safety
    /// This function is unsafe because it could dereference a raw pointer when
    /// attemping to access the underlying [`RBasic`] struct.
    unsafe fn dynamic_sym_p(&self, obj: VALUE) -> bool;

    /// Checks if the given object is a static symbol.
    fn static_sym_p(&self, obj: VALUE) -> bool;

    /// Checks if the given object is a symbol.
    ///
    /// # Safety
    /// This function is unsafe because it could dereference a raw pointer when
    /// attemping to access the underlying [`RBasic`] struct.
    unsafe fn symbol_p(&self, obj: VALUE) -> bool;

    /// Checks if the given object is a so-called Flonum.
    ///
    /// # Safety
    /// This function is unsafe because it could dereference a raw pointer when
    /// attemping to access the underlying [`RBasic`] struct.
    unsafe fn float_type_p(&self, obj: VALUE) -> bool;

    /// Checks if the given object is an integer type
    ///
    /// # Safety
    /// This function is unsafe because it could dereference a raw pointer when
    /// attemping to access the underlying [`RBasic`] struct.
    unsafe fn integer_type_p(&self, obj: VALUE) -> bool;

    /// Checks if the given object is a so-called Flonum.
    fn flonum_p(&self, obj: VALUE) -> bool;

    /// Checks if the given  object is  an immediate  i.e. an  object which  has
    /// no corresponding storage inside of the object space.
    fn immediate_p(&self, obj: VALUE) -> bool;

    /// Emulates Ruby's "if" statement by testing if the given `obj` is neither `Qnil` or `Qfalse`.
    ///
    /// # Safety
    /// This function is unsafe because it could dereference a raw pointer when
    /// attemping to access the underlying [`RBasic`] struct.
    fn rb_test(&self, ob: VALUE) -> bool;

    /// Queries the type of the object. Identical to `StableApi.builtin_type`,
    /// except it can also accept special constants.
    ///
    /// # Safety
    /// This function is unsafe because it could dereference a raw pointer when
    /// attemping to access the underlying [`RBasic`] struct.
    unsafe fn rb_type(&self, obj: VALUE) -> crate::ruby_value_type;

    /// Check if a Ruby string is interned (akin to `RSTRING_FSTR`).
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying flags of the RString. The caller must ensure that
    /// the `VALUE` is a valid pointer to an RString.
    unsafe fn rstring_interned_p(&self, obj: VALUE) -> bool;

    /// Blocks the current thread until the given duration has passed.
    fn thread_sleep(&self, duration: Duration);

    /// Checks if the given object is an RTypedData.
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying Ruby data. The caller must ensure that the pointer
    /// is valid and points to a T_DATA object.
    unsafe fn rtypeddata_p(&self, obj: VALUE) -> bool;

    /// Checks if the given RTypedData is embedded.
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying Ruby data. The caller must ensure that the pointer
    /// is valid and points to an RTypedData object.
    unsafe fn rtypeddata_embedded_p(&self, obj: VALUE) -> bool;

    /// Gets the data type from an RTypedData object.
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying Ruby data. The caller must ensure that the pointer
    /// is valid and points to an RTypedData object.
    unsafe fn rtypeddata_type(&self, obj: VALUE) -> *const crate::rb_data_type_t;

    /// Gets the data pointer from an RTypedData object.
    ///
    /// # Safety
    /// This function is unsafe because it dereferences a raw pointer to get
    /// access to underlying Ruby data. The caller must ensure that the pointer
    /// is valid and points to an RTypedData object.
    unsafe fn rtypeddata_get_data(&self, obj: VALUE) -> *mut std::ffi::c_void;

    /// Execute GC write barrier when storing a reference (akin to `RB_OBJ_WRITE`).
    ///
    /// Must be called when storing a VALUE reference from one heap object to another.
    /// This is critical for GC correctness - without it, the GC may collect objects
    /// that are still referenced.
    ///
    /// # Safety
    /// - `old` must be a valid heap-allocated Ruby object
    /// - `slot` must be a valid pointer to a VALUE within `old`
    /// - `young` must be a valid VALUE
    unsafe fn rb_obj_write(&self, old: VALUE, slot: *mut VALUE, young: VALUE) -> VALUE;

    /// Declare a write barrier without actually writing (akin to `RB_OBJ_WRITTEN`).
    ///
    /// Use this when you've already written a reference but need to inform the GC.
    ///
    /// # Safety
    /// - `old` must be a valid heap-allocated Ruby object
    /// - `oldv` is the previous value (can be any VALUE)
    /// - `young` must be a valid VALUE that was written
    unsafe fn rb_obj_written(&self, old: VALUE, oldv: VALUE, young: VALUE) -> VALUE;
}

#[cfg(stable_api_enable_compiled_mod)]
mod compiled;
#[cfg(stable_api_export_compiled_as_api)]
use compiled as api;

#[cfg(stable_api_include_rust_impl)]
#[cfg_attr(ruby_eq_2_7, path = "stable_api/ruby_2_7.rs")]
#[cfg_attr(ruby_eq_3_0, path = "stable_api/ruby_3_0.rs")]
#[cfg_attr(ruby_eq_3_1, path = "stable_api/ruby_3_1.rs")]
#[cfg_attr(ruby_eq_3_2, path = "stable_api/ruby_3_2.rs")]
#[cfg_attr(ruby_eq_3_3, path = "stable_api/ruby_3_3.rs")]
#[cfg_attr(ruby_eq_3_4, path = "stable_api/ruby_3_4.rs")]
#[cfg_attr(ruby_eq_4_0, path = "stable_api/ruby_4_0.rs")]
mod rust;
#[cfg(not(stable_api_export_compiled_as_api))]
use rust as api;

impl std::fmt::Debug for api::Definition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StableApiDefinition")
            .field("VERSION_MAJOR", &api::Definition::VERSION_MAJOR)
            .field("VERSION_MINOR", &api::Definition::VERSION_MINOR)
            .finish()
    }
}

/// Get the default stable API definition for the current Ruby version.
#[inline(always)]
pub const fn get_default() -> &'static api::Definition {
    const API: api::Definition = api::Definition {};
    &API
}

/// Get the fallback stable API definition for the current Ruby version, which
/// is compiled C code that is linked into to this crate.
#[inline(always)]
#[cfg(stable_api_enable_compiled_mod)]
pub const fn get_compiled() -> &'static compiled::Definition {
    const COMPILED_API: compiled::Definition = compiled::Definition {};
    &COMPILED_API
}
