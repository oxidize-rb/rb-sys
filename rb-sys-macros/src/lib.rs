#![recursion_limit = "128"]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn rb_extern(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(item);
    let ret = quote! {
        #[allow(non_snake_case)]
        #[no_mangle]
        extern "C" #input
    };

    ret.into()
}

#[proc_macro_attribute]
pub fn rb_extension_init(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(item);
    let ret = quote! {
        $crate::ruby_abi_magic!();

        #[rb_extern]
        #input
    };

    ret.into()
}
