#![allow(rustdoc::bare_urls)]
#![doc = include_str!("../readme.md")]

use proc_macro::{TokenStream, TokenTree};
use quote::quote;
use syn::{spanned::Spanned, ItemFn};

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
            #block;
        };
        rb_sys_test_helpers::trigger_full_gc!();
        ret
    };

    let test_fn = quote! {
        #[test]
        #(#attrs)*
        #vis #sig {
            rb_sys_test_helpers::with_ruby_vm(|| {
                let result = rb_sys_test_helpers::protect(|| {
                    #block
                });

                let ret = match result {
                    Err(err) => {
                        match std::env::var("RUST_BACKTRACE") {
                            Ok(val) if val == "1" => {
                                eprintln!("ruby exception:");
                                let errinfo = format!("{:#?}", err);
                                let errinfo = errinfo.replace("\n", "\n    ");
                                eprintln!("    {}", errinfo);
                            },
                            _ => (),
                        }
                        panic!("{}", err.inspect());
                    },
                    Ok(v) => v,
                };

                ret
            })
        }
    };

    test_fn.into()
}
