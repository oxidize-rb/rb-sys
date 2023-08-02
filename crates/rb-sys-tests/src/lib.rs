#![allow(unused_unsafe)]

extern crate rb_sys;

#[cfg(test)]
mod basic_smoke_test;

#[cfg(test)]
mod ruby_abi_version_test;

#[cfg(test)]
mod ruby_macros_test;

#[cfg(test)]
mod value_type_test;

#[cfg(test)]
mod special_consts_test;

#[cfg(test)]
mod tracking_allocator_test;

#[cfg(all(test, unix))]
mod memory_test;

#[cfg(test)]
mod stable_api_test;
