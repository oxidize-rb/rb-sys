use syn::{visit_mut::VisitMut, ItemForeignMod};

/// Add Ruby link attributes to the foreign exetern "C" modules.
#[derive(Debug)]
pub(crate) struct AddRubyLinkDirectives {
    pub link_name: String,
    pub kind: String,
}

impl AddRubyLinkDirectives {
    pub fn new(link_name: &str, kind: &str) -> Self {
        Self {
            link_name: link_name.to_string(),
            kind: kind.to_string(),
        }
    }
}

impl VisitMut for AddRubyLinkDirectives {
    fn visit_item_foreign_mod_mut(&mut self, item: &mut ItemForeignMod) {
        let name = &self.link_name;
        let kind = &self.kind;

        item.attrs.push(syn::parse_quote! {
            #[link(name = #name, kind = #kind)]
        });
    }
}
