#![allow(rustdoc::bare_urls)]
#![doc = include_str!("../readme.md")]

use proc_macro::{TokenStream, TokenTree};
use quote::quote;
use syn::{spanned::Spanned, ItemFn, ReturnType};

/// A proc-macro which generates a `#[test]` function has access to a valid Ruby VM.
///
/// Doing this properly it is not trivial, so this function abstracts away the
/// details. Under the hood, it ensures:
///
/// 1. The Ruby VM is setup and initialized once and only once.
/// 2. All code runs on the same OS thread.
/// 3. Exceptions are properly handled and propagated as Rust `Result<T,
///    RubyException>` values.
///
/// ### Example
///
/// ```
/// use rb_sys_test_helpers_macros::ruby_test;
///
/// #[ruby_test]
/// fn test_it_works() {
///    unsafe { rb_sys::rb_eval_string("1 + 1\0".as_ptr() as _) };
/// }
///
/// #[ruby_test(gc_stress)]
/// fn test_with_stress() {
///    unsafe { rb_sys::rb_eval_string("puts 'GC is stressing me out.'\0".as_ptr() as _) };
/// }
/// ```
///
/// Tests can also return a `Result` to use the `?` operator:
///
/// ```
/// use rb_sys_test_helpers_macros::ruby_test;
/// use std::error::Error;
///
/// #[ruby_test]
/// fn test_with_result() -> Result<(), Box<dyn Error>> {
///    let value = some_fallible_operation()?;
///    Ok(())
/// }
/// # fn some_fallible_operation() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
/// ```
#[proc_macro_attribute]
pub fn ruby_test(args: TokenStream, input: TokenStream) -> TokenStream {
    let input: ItemFn = match syn::parse2(input.into()) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error().into(),
    };

    let mut gc_stress = false;

    for arg in args {
        match arg {
            TokenTree::Ident(ident) => match ident.to_string().as_str() {
                "gc_stress" => gc_stress = true,
                kw => {
                    return syn::Error::new(kw.span(), format!("unknown argument: {}", kw))
                        .to_compile_error()
                        .into();
                }
            },
            _ => {
                return syn::Error::new(arg.span().into(), format!("expected identifier: {}", arg))
                    .to_compile_error()
                    .into();
            }
        }
    }

    let block = input.block;
    let attrs = input.attrs;
    let vis = input.vis;
    let sig = &input.sig;

    // Check if the function returns a Result type
    let returns_result = matches!(&sig.output, ReturnType::Type(_, _));

    let block = if gc_stress {
        quote! {
            rb_sys_test_helpers::with_gc_stress(|| {
                #block
            })
        }
    } else {
        quote! { #block }
    };

    let block = quote! {
        let ret = {
            #block
        };
        rb_sys_test_helpers::trigger_full_gc!();
        ret
    };

    // Helper to generate the error logging code
    let log_ruby_exception = quote! {
        match std::env::var("RUST_BACKTRACE") {
            Ok(val) if val == "1" || val == "full" => {
                eprintln!("ruby exception:");
                let errinfo = format!("{:#?}", err);
                let errinfo = errinfo.replace("\n", "\n    ");
                eprintln!("    {}", errinfo);
            },
            _ => (),
        }
    };

    // Generate different code based on whether the test returns a Result or not
    let test_fn = if returns_result {
        // For Result-returning tests, propagate errors properly
        quote! {
            #[test]
            #(#attrs)*
            #vis #sig {
                rb_sys_test_helpers::with_ruby_vm(|| {
                    let result = rb_sys_test_helpers::protect(|| {
                        #block
                    });

                    match result {
                        Err(err) => {
                            #log_ruby_exception
                            Err(err.into())
                        },
                        Ok(inner_result) => inner_result,
                    }
                }).expect("test execution failure")
            }
        }
    } else {
        // For unit-returning tests, use the original behavior
        quote! {
            #[test]
            #(#attrs)*
            #vis #sig {
                rb_sys_test_helpers::with_ruby_vm(|| {
                    let result = rb_sys_test_helpers::protect(|| {
                        #block
                    });

                    let ret = match result {
                        Err(err) => {
                            #log_ruby_exception
                            Err(err)
                        },
                        Ok(v) => Ok(v),
                    };

                    ret
                }).expect("test execution failure").expect("ruby exception");
            }
        }
    };

    test_fn.into()
}
