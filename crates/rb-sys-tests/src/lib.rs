extern crate rb_sys;

#[cfg(not(windows_broken_vm_init_3_1))]
use ctor::ctor;

#[cfg(not(windows_broken_vm_init_3_1))]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(not(windows_broken_vm_init_3_1))]
static INITED: AtomicBool = AtomicBool::new(false);

#[cfg(not(windows_broken_vm_init_3_1))]
#[ctor]
fn vm_init() {
    if INITED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        let var_in_stack_frame = unsafe { std::mem::zeroed() };
        unsafe { rb_sys::ruby_init_stack(var_in_stack_frame) };
        unsafe { rb_sys::ruby_init() };
    }
}

#[cfg(test)]
mod basic_smoke_test;
#[cfg(test)]
mod ruby_abi_version_test;
#[cfg(all(test, unix, feature = "ruby-macros"))]
mod ruby_macros_test;
