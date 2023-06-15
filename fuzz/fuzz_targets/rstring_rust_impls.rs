#![no_main]

use libfuzzer_sys::fuzz_target;
use rb_sys_test_helpers::setup_ruby_unguarded;

fuzz_target!(|data: &str| {
    unsafe {
        static INIT: std::sync::Once = std::sync::Once::new();

        INIT.call_once(|| {
            setup_ruby_unguarded();
        });

        let mut state = 0;

        rb_sys::rb_eval_string_protect(
            "Encoding.default_external = 'UTF-8'\0".as_ptr() as _,
            &mut state as _,
        );
        rb_sys::rb_eval_string_protect(
            "Encoding.default_internal = 'UTF-8'\0".as_ptr() as _,
            &mut state as _,
        );

        if state != 0 {
            panic!("Ruby error: {}", rb_sys::rb_errinfo());
        }

        let rb_string = rb_sys::rb_utf8_str_new(data.as_ptr() as _, data.len() as _);
        let mut pretty_printed = rb_sys::rb_inspect(rb_string);
        let serialized = rb_sys::rb_string_value_cstr(&mut pretty_printed);

        let mut state = 0;
        let ruby_string = rb_sys::rb_eval_string_protect(serialized, &mut state as _);
        if state != 0 {
            rb_sys::rb_p(rb_sys::rb_errinfo());
            panic!("Ruby error: {}", state);
        }

        // rstring_len
        {
            let rust_result = rb_sys::unlinkable::rust_impls::rstring_len(ruby_string);
            let compiled_c_result = rb_sys::unlinkable::compiled_c_impls::rstring_len(ruby_string);

            assert_eq!(compiled_c_result, rust_result);
        }

        // rstring_ptr
        {
            let rust_result = rb_sys::unlinkable::rust_impls::rstring_ptr(ruby_string);
            let compiled_c_result = rb_sys::unlinkable::compiled_c_impls::rstring_ptr(ruby_string);

            assert_eq!(compiled_c_result, rust_result);
        }
    }
});
