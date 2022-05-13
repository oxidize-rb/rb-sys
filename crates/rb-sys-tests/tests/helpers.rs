use std::sync::Once;

static INIT: Once = Once::new();

pub fn setup_ruby_vm() {
    INIT.call_once(|| unsafe {
        rb_sys::ruby_init();
    });
}
