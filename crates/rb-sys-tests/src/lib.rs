#![allow(unused_unsafe)]

extern crate rb_sys;

#[cfg(test)]
mod basic_smoke_test;

#[cfg(test)]
mod ruby_macros_test;

#[cfg(test)]
mod value_type_test;

#[cfg(test)]
mod special_consts_test;

// TODO: Figure out why this is failing on Ruby 3.5
#[cfg(test)]
#[cfg(ruby_lte_3_4)]
mod tracking_allocator_test;

#[cfg(all(test, unix))]
mod memory_test;

#[cfg(test)]
mod stable_api_test;

#[cfg(test)]
mod symbol_test;
