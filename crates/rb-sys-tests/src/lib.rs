#![allow(unused_unsafe)]

extern crate rb_sys;

use ctor::ctor;

use std::sync::atomic::{AtomicBool, Ordering};

static INITED: AtomicBool = AtomicBool::new(false);

#[ctor]
fn vm_init() {
    if INITED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        unsafe {
            let var_in_stack_frame = std::mem::zeroed();
            let argv: [*mut std::os::raw::c_char; 0] = [];
            let argv = argv.as_ptr();
            let mut argc = 0;

            rb_sys::ruby_init_stack(var_in_stack_frame);
            rb_sys::ruby_sysinit(&mut argc, argv as _);
            rb_sys::ruby_init();
        }
    }
}

#[macro_use]
mod helpers;

#[cfg(test)]
mod basic_smoke_test;

#[cfg(test)]
mod ruby_abi_version_test;

#[cfg(all(test, unix, feature = "ruby-macros"))]
mod ruby_macros_test;

#[cfg(test)]
mod value_type_test;

#[cfg(test)]
mod special_consts_test;
