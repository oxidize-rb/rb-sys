use std::error::Error;
use syn::Item;

/// Append a link directive to each foreign module to the given syntax tree.
pub fn add_link_ruby_directives(
    syntax: &mut syn::File,
    link_name: &str,
    kind: &str,
) -> Result<(), Box<dyn Error>> {
    for item in syntax.items.iter_mut() {
        if let Item::ForeignMod(fmod) = item {
            fmod.attrs.push(syn::parse_quote! {
                #[link(name = #link_name, kind = #kind)]
            });
        }
    }

    Ok(())
}
