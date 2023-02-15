use std::fmt::Write;
use syn::Lit;

fn is_junk_key(name: &str) -> bool {
    name.starts_with("HAVE_")
        || name.starts_with("USE_")
        || name.starts_with("RUBY_BIRTH")
        || name.starts_with("RUBY_API_VERSION")
        || name.starts_with("RUBY_RELEASE_DATE")
        || name.starts_with("RUBY_AUTHOR") // <3 u mr. matz
        || name.ends_with("_H") // header files
        || name.contains("COMPILER_IS")
        || name.starts_with("SIZEOF_")
        || name == "FALSE"
        || name == "TRUE"
        || name.ends_with("_SOURCE")
        || name.starts_with("SIGNEDNESS")
        || name.starts_with('_')
        || name.starts_with("STDC")
}

fn is_junk_value(value: &Lit) -> bool {
    match value {
        Lit::ByteStr(lit_str) => lit_str.value().ends_with("\0".as_bytes()),
        _ => false,
    }
}

/// Filters out autoconf-style `#[cfg]` attributes.
#[derive(Debug)]
pub struct RemoveDefinesFilter;

impl syn::visit_mut::VisitMut for RemoveDefinesFilter {
    fn visit_file_mut(&mut self, file: &mut syn::File) {
        let mut shared_buf = String::with_capacity(64);

        file.items.retain(|item| {
            if let syn::Item::Const(item) = item {
                shared_buf.clear();
                write!(shared_buf, "{:.64}", item.ident).unwrap();

                if let syn::Visibility::Public(_) = item.vis {
                    if is_junk_key(&shared_buf) {
                        return false;
                    }
                }

                if let syn::Expr::Lit(syn::ExprLit { lit, .. }) = item.expr.as_ref() {
                    if is_junk_value(lit) {
                        return false;
                    }
                }

                true
            } else {
                true
            }
        });
    }
}
