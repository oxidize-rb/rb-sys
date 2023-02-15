use bindgen::callbacks::ParseCallbacks;
use quote::{ToTokens, __private::TokenTree};
use regex::Regex;
use std::borrow::Cow;

lazy_static::lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(r#"https?://[^\s'"]+"#).unwrap();
    static ref DOC_SECTION_REGEX: Regex = Regex::new(r"\s*@(warning|internal|private|note)\s*").unwrap();
    static ref PARAM_DIRECTIVE_REGEX: Regex = Regex::new(r"\s*(@\w+)\[(\S+)\]\s+([^\n]+)").unwrap();
    static ref OTHER_DIRECTIVE_REGEX: Regex = Regex::new(r"\s*(@\w+)\s+([^\n]+)").unwrap();
    static ref BARE_CODE_REF_REGEX: Regex = Regex::new(r"(\b)(rb_(\w|_)+)(\(|\))*").unwrap();
    static ref NO_NEWLINES_AND_SINGLE_SPACE_REGEX: Regex = Regex::new(r"\s+").unwrap();
    static ref UNESCAPE_REGEX: Regex = Regex::new(r"\\n").unwrap();
    static ref TWO_SPACES_REGEX: Regex = Regex::new(r"([a-z])  ([a-z])").unwrap();
    static ref EXTRANEOUS_NEWLINE_REGEX: Regex = Regex::new(r"([a-z])\n ([a-z])").unwrap();
}

/// Add deprecation warnings to the generated bindings.
#[derive(Debug)]
pub struct DeprecationWarnings;

impl syn::visit_mut::VisitMut for DeprecationWarnings {
    fn visit_attribute_mut(&mut self, attr: &mut syn::Attribute) {
        let tokens = &attr.tokens;
        let path = &attr.path;

        if path.is_ident("doc") {
            for s in tokens.to_token_stream() {
                if let TokenTree::Literal(lit) = s {
                    let doc = lit.to_string();
                    let doc = doc.trim_matches('"').trim();

                    if let Some(deprecated_idx) = doc.find("@deprecated") {
                        let doc = UNESCAPE_REGEX.replace_all(doc, "\n");
                        let msg = NO_NEWLINES_AND_SINGLE_SPACE_REGEX.replace_all(&doc, " ");

                        let msg: Cow<str> = if deprecated_idx == 0 {
                            msg.trim_start_matches("@deprecated").into()
                        } else {
                            msg.replace("@deprecated", "").into()
                        };

                        let msg = msg.trim();

                        if msg.is_empty() {
                            *attr = syn::parse_quote! {
                                #[deprecated]
                            };
                        } else {
                            *attr = syn::parse_quote! {
                                #[deprecated(note = #msg)]
                            };
                        };

                        return;
                    }
                }
            }
        }
    }
}

/// Process the doc attributes.
#[derive(Debug)]
pub(crate) struct DocCallbacks;

impl ParseCallbacks for DocCallbacks {
    fn process_comment(&self, comment: &str) -> Option<String> {
        let cleaned = comment.trim_matches('"').trim();

        if comment.is_empty() {
            return Some("\n".into());
        }

        if cleaned.contains("@deprecated") {
            return Some(cleaned.to_string());
        }

        let cleaned = URL_REGEX.replace_all(cleaned, "<${0}>");

        let cleaned = DOC_SECTION_REGEX.replace_all(&cleaned, |captures: &regex::Captures| {
            if let Some(header) = captures.get(1) {
                format!("\n---\n ### {}\n", capitalize(header.as_str())).into()
            } else {
                Cow::Borrowed("")
            }
        });
        let cleaned = PARAM_DIRECTIVE_REGEX.replace_all(&cleaned, "\n- **$1** `$2` $3");
        let cleaned = OTHER_DIRECTIVE_REGEX.replace_all(&cleaned, "\n- **$1** $2");
        let cleaned = BARE_CODE_REF_REGEX.replace_all(&cleaned, "${1}[`${2}`]");
        let cleaned = TWO_SPACES_REGEX.replace_all(&cleaned, "${1} ${2}");
        let cleaned = EXTRANEOUS_NEWLINE_REGEX.replace_all(&cleaned, "${1} ${2}");

        Some(cleaned.to_string())
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
