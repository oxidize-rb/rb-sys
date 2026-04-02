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

/// Functions verified against the Ruby C source to never allocate Ruby objects.
/// Wrapping these with rb_gc_start() is wasteful — skip them for performance.
const NON_ALLOCATING_FUNCTIONS: &[&str] = &[
    // -- Predicate functions (_p suffix) --
    "rb_obj_frozen_p",
    "rb_block_given_p",
    "rb_keyword_given_p",
    "rb_fiber_alive_p",
    "rb_proc_lambda_p",
    "rb_mutex_locked_p",
    "rb_method_basic_definition_p",
    "rb_bigzero_p",
    "rb_ary_shared_with_p",
    "rb_class_inherited_p",
    "rb_mod_include_p",
    "rb_io_closed_p",
    "rb_autoload_p",
    "rb_enc_dummy_p",
    "rb_enc_str_asciionly_p",
    "rb_enc_unicode_p",
    "rb_enc_symname_p",
    "rb_symname_p",
    "rb_reserved_fd_p",
    "rb_memory_view_available_p",
    "rb_absint_singlebit_p",
    "rb_typeddata_inherited_p",
    "rb_econv_has_convpath_p",
    "rb_file_directory_p",
    "rb_profile_frame_singleton_method_p",
    "rb_tracepoint_enabled_p",
    // -- Type checks / identity --
    "rb_obj_is_kind_of",
    "rb_obj_is_instance_of",
    "rb_obj_is_proc",
    "rb_obj_is_fiber",
    "rb_obj_is_method",
    "rb_typeddata_is_kind_of",
    "rb_check_typeddata",
    "rb_check_type",
    // -- Class/object accessors --
    "rb_obj_class",
    "rb_obj_classname",
    "rb_class_real",
    "rb_class_get_superclass",
    "rb_class_name",
    "rb_class2name",
    "rb_class_attached_object",
    // -- Object state --
    "rb_obj_freeze",
    "rb_obj_setup",
    // -- Symbol/ID --
    "rb_sym2id",
    "rb_id2name",
    "rb_sym2str",
    // -- Instance variable read --
    "rb_ivar_get",
    "rb_ivar_defined",
    // -- String (non-allocating) --
    "rb_str_length",
    "rb_str_hash",
    "rb_str_comparable",
    "rb_str_cmp",
    "rb_str_equal",
    // -- Array (non-allocating) --
    "rb_ary_entry",
    "rb_ary_freeze",
    // -- Hash (non-allocating) --
    "rb_hash_lookup",
    "rb_hash_lookup2",
    "rb_hash_size",
    "rb_hash_freeze",
    // -- Struct (non-allocating) --
    "rb_struct_size",
    "rb_struct_getmember",
    "rb_struct_aref",
    // -- Encoding accessors --
    "rb_enc_get_index",
    "rb_enc_get",
    "rb_enc_from_index",
    "rb_enc_to_index",
    "rb_enc_compatible",
    "rb_enc_check",
    "rb_usascii_encindex",
    "rb_utf8_encindex",
    "rb_ascii8bit_encindex",
    // -- Numeric conversions --
    "rb_num2long",
    "rb_num2ulong",
    "rb_num2int",
    "rb_fix2int",
    "rb_fix2uint",
    "rb_num2short",
    // -- Thread/fiber --
    "rb_thread_current",
    "rb_thread_main",
    "rb_thread_local_aref",
    "rb_thread_alone",
    "rb_fiber_current",
    // -- IO --
    "rb_io_descriptor",
    "rb_io_check_closed",
    // -- Range --
    "rb_range_beg_len",
    "rb_range_values",
    // -- Regexp --
    "rb_reg_options",
    // -- Control flow --
    "rb_protect",
    "rb_during_gc",
    "rb_respond_to",
    "rb_obj_respond_to",
    // -- Source location --
    "rb_sourcefile",
    "rb_sourceline",
    "rb_frame_this_func",
    "rb_frame_callee",
    // -- Profile frame (non-allocating subset) --
    "rb_profile_frame_path",
    "rb_profile_frame_label",
    "rb_profile_frame_base_label",
    // -- Memory view --
    "rb_memory_view_get",
];

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

    // Skip functions verified to never allocate Ruby objects
    if NON_ALLOCATING_FUNCTIONS.contains(&name) {
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
