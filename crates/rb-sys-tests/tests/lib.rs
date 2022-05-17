extern crate rb_sys;

use ctor::ctor;
use std::sync::atomic::{AtomicBool, Ordering};

static INITED: AtomicBool = AtomicBool::new(false);

#[ctor]
fn vm_init() {
    match INITED.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(_) => {
            let var_in_stack_frame = unsafe { std::mem::zeroed() };
            unsafe { rb_sys::ruby_init_stack(var_in_stack_frame) };
            unsafe { rb_sys::ruby_init() };
        }
        Err(_) => {}
    }
}

mod basic_smoke_test;
mod ruby_abi_version_test;
mod ruby_macros_test;
