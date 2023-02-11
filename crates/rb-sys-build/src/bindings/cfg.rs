use std::error::Error;

use quote::ToTokens;
use syn::{Expr, ExprLit, ItemConst, Lit};

/// Represents a parsed key-value pair from the bindings, either a `#define` or
/// a `const` value from C.
#[derive(Debug, Clone)]
pub enum Item {
    Defines((String, String)),
    Constant((&'static str, String)),
}

impl Item {
    pub fn as_cargo_cfg(&self) -> String {
        match self {
            Self::Defines(_) => format!("cargo:defines_{}={}", self.key(), self.value()),
            Self::Constant(_) => format!("cargo:{}={}", self.key(), self.value()),
        }
    }

    pub fn as_rustc_cfg(&self) -> String {
        format!("cargo:rustc-cfg={}=\"{}\"", self.key(), self.value())
    }

    fn key(&self) -> String {
        match self {
            Self::Defines((key, _)) => key.to_lowercase(),
            Self::Constant((key, _)) => key.to_lowercase(),
        }
    }

    fn value(&self) -> &str {
        match self {
            Self::Defines((_, val)) => val.trim_matches('\n'),
            Self::Constant((_, val)) => val.trim_matches('\n'),
        }
    }
}

// Add things like `#[cfg(ruby_use_transient_heap = "true")]` to the bindings config
pub fn extract(syntax: &syn::File) -> Result<Vec<Item>, Box<dyn Error>> {
    let mut vec = Vec::new();
    fn is_defines(line: &str) -> bool {
        line.starts_with("HAVE_RUBY")
            || line.starts_with("HAVE_RB")
            || line.starts_with("USE")
            || line.starts_with("RUBY_DEBUG")
            || line.starts_with("RUBY_NDEBUG")
    }

    for item in syntax.items.iter() {
        if let syn::Item::Const(item) = item {
            let conf = Value::new(item);
            let conf_name = conf.name();

            if is_defines(&conf_name) {
                let name = conf_name.to_lowercase();
                let val = conf.value_bool().to_string();
                vec.push(Item::Defines((name.clone(), val.clone())));
            }

            if conf_name.starts_with("RUBY_ABI_VERSION") {
                vec.push(Item::Constant(("ruby_abi_version", conf.value_string())));
            }
        }
    }

    Ok(vec)
}

/// An autoconf constant in the bindings
struct Value<'a> {
    item: &'a syn::ItemConst,
}

impl<'a> Value<'a> {
    pub fn new(item: &'a ItemConst) -> Self {
        Self { item }
    }

    pub fn name(&self) -> String {
        self.item.ident.to_string()
    }

    pub fn value_string(&self) -> String {
        match &*self.item.expr {
            Expr::Lit(ExprLit { lit, .. }) => lit.to_token_stream().to_string(),
            _ => panic!(
                "Could not convert HAVE_* constant to string: {:#?}",
                self.item.ident
            ),
        }
    }

    pub fn value_bool(&self) -> bool {
        match &*self.item.expr {
            Expr::Lit(ExprLit {
                lit: Lit::Int(ref lit),
                ..
            }) => lit.base10_parse::<u8>().unwrap() != 0,
            Expr::Lit(ExprLit {
                lit: Lit::Bool(ref lit),
                ..
            }) => lit.value,
            _ => panic!(
                "Could not convert HAVE_* constant to bool: {:#?}",
                self.item.ident
            ),
        }
    }
}
