use quote::__private::{Literal, TokenTree};
use std::borrow::Cow;
use std::error::Error;
use syn::Item;
use syn::{Attribute, __private::ToTokens};

/// Turn the cruby comments into rustdoc comments.
pub fn tidy(syntax: &mut syn::File) -> Result<(), Box<dyn Error>> {
    for item in syntax.items.iter_mut() {
        match item {
            Item::ForeignMod(fmod) => {
                for item in fmod.items.iter_mut() {
                    match item {
                        syn::ForeignItem::Fn(f) => clean_doc_attrs(&mut f.attrs),
                        syn::ForeignItem::Static(s) => clean_doc_attrs(&mut s.attrs),
                        syn::ForeignItem::Type(s) => clean_doc_attrs(&mut s.attrs),
                        _ => {}
                    }
                }
            }
            Item::Type(t) => clean_doc_attrs(&mut t.attrs),
            Item::Struct(s) => {
                clean_doc_attrs(&mut s.attrs);

                for f in s.fields.iter_mut() {
                    clean_doc_attrs(&mut f.attrs);
                }
            }
            Item::Enum(e) => {
                clean_doc_attrs(&mut e.attrs);

                for v in e.variants.iter_mut() {
                    clean_doc_attrs(&mut v.attrs);
                }
            }
            Item::Union(u) => {
                clean_doc_attrs(&mut u.attrs);

                for u in u.fields.named.iter_mut() {
                    clean_doc_attrs(&mut u.attrs);
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn tidy_line(attr: &mut Attribute) -> bool {
    use regex::Regex;

    lazy_static::lazy_static! {
        static ref URL_REGEX: Regex = Regex::new(r#"https?://[^\s'"]+"#).unwrap();
        static ref DOC_SECTION_REGEX: Regex = Regex::new(r"^@(warning|internal|private|note)\s*").unwrap();
        static ref PARAM_DIRECTIVE_REGEX: Regex = Regex::new(r"^(@\w+)\[(\S+)\]\s+(.*)$").unwrap();
        static ref OTHER_DIRECTIVE_REGEX: Regex = Regex::new(r"^(@\w+)\s+(.*)$").unwrap();
        static ref BARE_CODE_REF_REGEX: Regex = Regex::new(r"(\b)(rb_(\w|_)+)(\(|\))*").unwrap();
    }

    if !attr.path.is_ident("doc") {
        return false;
    }

    let mut deprecated: bool = false;

    let new_tokens = attr
        .tokens
        .to_token_stream()
        .into_iter()
        .map(|token| {
            if let TokenTree::Literal(l) = token {
                let cleaned = l.to_string();
                let cleaned = cleaned.trim_matches('"').trim();
                let cleaned = URL_REGEX.replace_all(cleaned, "<${0}>");
                let cleaned =
                    DOC_SECTION_REGEX.replace_all(&cleaned, |captures: &regex::Captures| {
                        if let Some(header) = captures.get(1) {
                            format!("---\n ### {}\n", capitalize(header.as_str())).into()
                        } else {
                            Cow::Borrowed("")
                        }
                    });
                let cleaned = PARAM_DIRECTIVE_REGEX.replace(&cleaned, "- **$1** `$2` $3");
                let cleaned = OTHER_DIRECTIVE_REGEX.replace(&cleaned, "- **$1** $2");
                let cleaned = BARE_CODE_REF_REGEX.replace_all(&cleaned, "${1}[`${2}`]");

                if cleaned.is_empty() {
                    return TokenTree::Literal(Literal::string("\n"));
                }

                if cleaned.contains("@deprecated") {
                    deprecated = true;
                }

                Literal::string(&cleaned).into()
            } else {
                token
            }
        })
        .collect();

    attr.tokens = new_tokens;
    deprecated
}

fn clean_doc_attrs(attrs: &mut Vec<Attribute>) {
    let mut deprecated: bool = false;

    for attr in attrs.iter_mut() {
        if tidy_line(attr) {
            deprecated = true;
        };
    }

    if deprecated {
        attrs.push(syn::parse_quote! {
            #[deprecated]
        })
    }
}

fn capitalize(input: &str) -> Cow<'_, str> {
    if let Some(first) = input.chars().next() {
        if first.is_ascii_uppercase() && input[1..].chars().all(|c| c.is_ascii_lowercase()) {
            Cow::Borrowed(input)
        } else {
            let mut result = String::with_capacity(input.len());
            result.push(first.to_ascii_uppercase());
            result.push_str(&input[1..].to_ascii_lowercase());
            Cow::Owned(result)
        }
    } else {
        Cow::Borrowed(input)
    }
}
