use quote::ToTokens;

const OPAQUE_STRUCTS: [&str; 2] = ["RString", "RArray"];

/// Generate opaque structs for the given bindings.
pub fn opaqueify_bindings(bindings: bindgen::Builder, wrapper_h: &mut String) -> bindgen::Builder {
    OPAQUE_STRUCTS.iter().fold(bindings, |bindings, name| {
        gen_opaque_struct(bindings, name, wrapper_h)
    })
}

/// Categorize all bindings into stable, unstable, and internal.
pub fn categorize_bindings(syntax: &mut syn::File) {
    let mut normal_items = Vec::new();
    let mut unstable_items = Vec::new();
    let mut internal_items = Vec::new();
    let mut excluded_items = Vec::new();
    let mut opaque_items = Vec::new();
    let mut opaque_idents_to_swap = Vec::new();

    for item in syntax.items.iter_mut() {
        if let syn::Item::Struct(s) = item {
            if s.ident.to_string().contains("rb_sys__Opaque__") {
                let new_name = s.ident.to_string().replace("rb_sys__Opaque__", "");
                s.ident = syn::Ident::new(&new_name, s.ident.span());
                opaque_idents_to_swap.push(new_name);

                opaque_items.push(item.clone());
            } else {
                normal_items.push(item.clone());
            }
        } else {
            if let syn::Item::Fn(ref mut f) = item {
                if f.sig.ident.to_string().contains("bindgen_test_") {
                    let body = &mut f.block;
                    let code = body.clone().to_token_stream().to_string();
                    let new_code = code.replace("rb_sys__Opaque__", "super::stable::");

                    *body = syn::parse_quote! {
                        {
                            #[allow(unused)]
                            use super::internal::*;
                            #new_code;
                        }
                    };
                }
            }

            normal_items.push(item.clone());
        }
    }

    for mut item in normal_items {
        if let syn::Item::Struct(s) = &mut item {
            if opaque_idents_to_swap.contains(&s.ident.to_string()) {
                internal_items.push(s.clone());
                s.attrs.push(syn::parse_quote! {
                    #[deprecated(note = "To improve ABI stability with ruby-head, usage of this internal struct has been deprecated. All necessary functions are now provided, so direct usage of internal fields should no longer be necessary. To migrate, please replace the usage of this internal struct with its counterpart in the `rb_sys::stable` module. For example, instead of `use rb_sys::rb_sys__Opaque__ExampleStruct;`, use `use rb_sys::stable::ExampleStruct;`. This struct will be removed in a future version of rb_sys.")]
                });
                unstable_items.push(item);
            } else {
                excluded_items.push(item);
            }
        } else {
            excluded_items.push(item);
        }
    }

    *syntax = syn::parse_quote! {
        /// Contains all items that are not yet categorized by ABI stability.
        /// These items are candidates for promotion to `stable` or `unstable`
        /// in the future.
        pub mod uncategorized {
            #(#excluded_items)*
        }

        /// Contains all items that are considered unstable ABI and should be
        /// avoided. Any items in this list offer a stable alternative for most
        /// use cases.
        pub mod unstable {
            use super::uncategorized::*;

            #(#unstable_items)*
        }

        /// Contains all items that are considered stable ABI and are safe to
        /// use. These items are intentionally opaque to prevent accidental
        /// compatibility issues.
        ///
        /// If you need to access the internals of these items, please open an
        /// issue.
        pub mod stable {
            #(#opaque_items)*
        }

        /// Unstable items for usage internally in rb_sys to avoid deprecated warnings.
        pub (crate) mod internal {
            use super::uncategorized::*;

            #(#internal_items)*
        }
    };
}

fn gen_opaque_struct(
    bindings: bindgen::Builder,
    name: &str,
    wrapper_h: &mut String,
) -> bindgen::Builder {
    let struct_name = format!("rb_sys__Opaque__{}", name);
    wrapper_h.push_str(&format!(
        "struct {} {{ struct {} dummy; }};\n",
        struct_name, name
    ));

    bindings
        .opaque_type(&struct_name)
        .allowlist_type(struct_name)
}
