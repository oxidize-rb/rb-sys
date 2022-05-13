use std::sync::Once;

static INIT: Once = Once::new();

pub fn setup_ruby_vm() {
    INIT.call_once(|| unsafe {
        let variable_in_this_stack_frame: rb_sys::VALUE = std::mem::zeroed();
        rb_sys::ruby_init_stack(variable_in_this_stack_frame as *mut _);
        rb_sys::ruby_init();
    });
}
