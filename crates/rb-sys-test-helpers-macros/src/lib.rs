use proc_macro::TokenStream;
use quote::quote;
use syn::ItemFn;

/// A proc-macro which generates a `#[test]` function has access to a valid Ruby VM.
///
/// ```
/// use rb_sys_test_helpers_macros::ruby_test;
///
/// #[ruby_test]
/// fn test_it_works() {
///    unsafe { rb_sys::rb_eval_string("1 + 1\0".as_ptr() as _) };
/// }
/// ```
#[proc_macro_attribute]
pub fn ruby_test(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let input: ItemFn = match syn::parse2(input.into()) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error().into(),
    };

    let block = input.block;
    let attrs = input.attrs;
    let vis = input.vis;
    let sig = &input.sig;
    let test_fn = quote! {
        #[test]
        #(#attrs)*
        #vis #sig {
            rb_sys_test_helpers::with_ruby_vm(|| #block)
        }
    };

    test_fn.into()
}
