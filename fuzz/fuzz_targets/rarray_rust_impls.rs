#![no_main]

use libfuzzer_sys::fuzz_target;
use rb_sys_test_helpers::setup_ruby_unguarded;

fuzz_target!(|data: Vec<u64>| {
    unsafe {
        static INIT: std::sync::Once = std::sync::Once::new();

        INIT.call_once(|| {
            setup_ruby_unguarded();
        });

        let mut state = 0;
        let eval_string = format!("{:?}\0", data);
        let ruby_array = rb_sys::rb_eval_string_protect(eval_string.as_ptr() as _, &mut state as _);

        if state != 0 {
            rb_sys::rb_p(rb_sys::rb_errinfo());
            panic!("Ruby error: {}", state);
        }

        {
            let rust_result = rb_sys::unlinkable::rust_impls::rarray_len(ruby_array);
            let compiled_c_result = rb_sys::unlinkable::compiled_c_impls::rarray_len(ruby_array);

            assert_eq!(compiled_c_result, rust_result);
        }

        {
            let rust_result = rb_sys::unlinkable::rust_impls::rarray_const_ptr(ruby_array);
            let compiled_c_result =
                rb_sys::unlinkable::compiled_c_impls::rarray_const_ptr(ruby_array);

            assert_eq!(compiled_c_result, rust_result);
        }
    }
});
