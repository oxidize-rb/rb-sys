use syn::{parse_quote, visit_mut::VisitMut};

#[derive(Debug)]
pub struct RemapNativeTypes;

impl VisitMut for RemapNativeTypes {
    fn visit_type_mut(&mut self, i: &mut syn::Type) {
        match i {
            syn::Type::Array(arr) => {
                let elem = arr.elem.as_mut();
                if let syn::Type::Path(syn::TypePath { path, .. }) = elem {
                    if path.segments.len() == 1 {
                        if let Some(ident) = path.segments.first().map(|s| &s.ident) {
                            let newty: syn::Type = match ident {
                                _ if ident == "__int8_t" => parse_quote! { i8 },
                                _ if ident == "__int16_t" => parse_quote! { i16 },
                                _ if ident == "__int32_t" => parse_quote! { i32 },
                                _ if ident == "__int64_t" => parse_quote! { i64 },
                                _ if ident == "__uint8_t" => parse_quote! { u8 },
                                _ if ident == "__uint16_t" => parse_quote! { u16 },
                                _ if ident == "__uint32_t" => parse_quote! { u32 },
                                _ if ident == "__uint64_t" => parse_quote! { u64 },
                                _ if ident == "stat" => parse_quote! { crate::os::raw::stat },
                                _ => return,
                            };

                            arr.elem = Box::new(newty);
                        }
                    }
                };
            }
            syn::Type::Path(syn::TypePath { path, .. }) => {
                if path.segments.len() == 1 {
                    if let Some(ident) = path.segments.first().map(|s| &s.ident) {
                        let newty: syn::Type = match ident {
                            _ if ident == "__darwin_time_t" => parse_quote! { time_t },
                            _ if ident == "__darwin_pid_t" => parse_quote! { pid_t },
                            _ if ident == "__darwin_off_t" => parse_quote! { off_t },
                            _ if ident == "__darwin_mode_t" => parse_quote! { mode_t },
                            _ if ident == "__darwin_suseconds_t" => {
                                parse_quote! { suseconds_t }
                            }
                            _ => return,
                        };

                        *i = newty;
                    }
                }
            }
            _ => {}
        }
    }
}
