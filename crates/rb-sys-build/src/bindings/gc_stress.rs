use proc_macro2::Span;
use quote::quote;

/// When the `gc-stress` feature is enabled, wrap all extern "C" functions
/// with calls to `rb_gc_start()` before and after the real function call.
/// This helps smoke out GC safety bugs in native extensions.
///
/// Functions that are excluded from wrapping:
/// - `rb_gc_*` functions (would cause infinite recursion)
/// - `ruby_x*` functions (memory allocators called during GC)
/// - Variadic functions (can't forward varargs in Rust)
pub fn wrap_functions_with_gc_stress(syntax: &mut syn::File) {
    let mut new_items: Vec<syn::Item> = Vec::new();

    for item in syntax.items.iter_mut() {
        if let syn::Item::ForeignMod(foreign_mod) = item {
            let mut kept_items = Vec::new();
            let mut raw_fns = Vec::new();
            let mut wrapper_fns = Vec::new();

            for foreign_item in foreign_mod.items.drain(..) {
                if let syn::ForeignItem::Fn(f) = foreign_item {
                    let name = f.sig.ident.to_string();

                    if should_skip(&name, &f) {
                        kept_items.push(syn::ForeignItem::Fn(f));
                        continue;
                    }

                    let (raw_fn, wrapper) = generate_wrapper(f);
                    raw_fns.push(raw_fn);
                    wrapper_fns.push(wrapper);
                } else {
                    kept_items.push(foreign_item);
                }
            }

            // Put back the items that weren't wrapped
            foreign_mod.items = kept_items;

            // Create a new extern block for the raw (renamed) functions
            if !raw_fns.is_empty() {
                let mut raw_block = foreign_mod.clone();
                raw_block.items = raw_fns.into_iter().map(syn::ForeignItem::Fn).collect();
                new_items.push(syn::Item::ForeignMod(raw_block));
            }

            // Add the wrapper functions as top-level items
            for wrapper in wrapper_fns {
                new_items.push(syn::Item::Fn(wrapper));
            }
        }
    }

    // Add a private helper that calls rb_gc_start() — avoids redeclaring it
    // in every wrapper function. rb_gc_start is already in the extern block
    // so we just call it directly.
    let gc_stress_helper: syn::Item = syn::parse_quote! {
        #[doc(hidden)]
        #[inline(never)]
        unsafe fn __rb_sys_gc_stress() {
            rb_gc_start();
        }
    };
    new_items.insert(0, gc_stress_helper);

    syntax.items.extend(new_items);
}

fn should_skip(name: &str, f: &syn::ForeignItemFn) -> bool {
    // Skip GC functions to avoid infinite recursion
    if name.starts_with("rb_gc_") {
        return true;
    }

    // Skip memory allocators (called during GC)
    if name.starts_with("ruby_x") {
        return true;
    }

    // Skip variadic functions (can't forward varargs)
    if f.sig.variadic.is_some() {
        return true;
    }

    false
}

fn generate_wrapper(mut f: syn::ForeignItemFn) -> (syn::ForeignItemFn, syn::ItemFn) {
    let original_name = f.sig.ident.clone();
    let original_name_str = original_name.to_string();
    let raw_name = syn::Ident::new(
        &format!("__rb_sys_raw_{}", original_name_str),
        original_name.span(),
    );

    // Collect parameter names for forwarding
    let param_names: Vec<_> = f
        .sig
        .inputs
        .iter()
        .enumerate()
        .map(|(i, arg)| match arg {
            syn::FnArg::Typed(pat_type) => {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    pat_ident.ident.clone()
                } else {
                    syn::Ident::new(&format!("__arg{}", i), Span::call_site())
                }
            }
            _ => syn::Ident::new(&format!("__arg{}", i), Span::call_site()),
        })
        .collect();

    // Clone the original attrs (docs etc) for the wrapper
    let wrapper_attrs = f.attrs.clone();

    // Build the raw extern fn: rename + add #[link_name] + make non-pub
    f.sig.ident = raw_name.clone();
    f.vis = syn::Visibility::Inherited;
    f.attrs
        .push(syn::parse_quote! { #[link_name = #original_name_str] });

    // Build the wrapper function
    let wrapper_sig = {
        let mut sig = f.sig.clone();
        sig.ident = original_name;
        sig.unsafety = Some(syn::token::Unsafe::default());
        sig.abi = Some(syn::Abi {
            extern_token: syn::token::Extern::default(),
            name: Some(syn::LitStr::new("C", Span::call_site())),
        });
        sig
    };

    let returns_never =
        matches!(&wrapper_sig.output, syn::ReturnType::Type(_, ty) if is_never_type(ty));
    let has_return = !matches!(&wrapper_sig.output, syn::ReturnType::Default);

    let call_expr = quote! { #raw_name(#(#param_names),*) };

    let body: syn::Block = if returns_never {
        syn::parse_quote! {{
            __rb_sys_gc_stress();
            #call_expr
        }}
    } else if has_return {
        syn::parse_quote! {{
            __rb_sys_gc_stress();
            let __ret = #call_expr;
            __rb_sys_gc_stress();
            __ret
        }}
    } else {
        syn::parse_quote! {{
            __rb_sys_gc_stress();
            #call_expr;
            __rb_sys_gc_stress();
        }}
    };

    let wrapper = syn::ItemFn {
        attrs: wrapper_attrs,
        vis: syn::parse_quote! { pub },
        sig: wrapper_sig,
        block: Box::new(body),
    };

    (f, wrapper)
}

fn is_never_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Never(_))
}
