/// Represents the kind of library.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LibraryKind {
    Framework,
    Dylib,
    Static,
    Native,
}

/// Represents a search path that can be linked with Cargo.
#[derive(Debug, PartialEq, Eq)]
pub struct Library {
    pub kind: LibraryKind,
    pub name: String,
    pub rename: Option<String>,
    pub modifiers: Vec<String>,
}

impl Library {
    /// Creates a new library.
    pub fn new(
        name: String,
        kind: LibraryKind,
        rename: Option<String>,
        modifiers: Vec<String>,
    ) -> Self {
        Self {
            kind,
            name,
            rename,
            modifiers,
        }
    }
}

impl From<&str> for LibraryKind {
    fn from(s: &str) -> Self {
        match s {
            "framework" => LibraryKind::Framework,
            "dylib" => LibraryKind::Dylib,
            "static" => LibraryKind::Static,
            "native" => LibraryKind::Native,
            _ => panic!("Unknown lib kind: {}", s),
        }
    }
}

impl From<&str> for Library {
    fn from(s: &str) -> Self {
        let parts: Vec<_> = s.split('=').map(|s| s.to_owned()).collect();

        match parts.len() {
            1 => Library {
                kind: LibraryKind::Native,
                name: parts.first().expect("lib name is empty").to_owned(),
                rename: None,
                modifiers: vec![],
            },
            2 => Library {
                kind: parts.first().expect("no kind for lib").as_str().into(),
                name: parts.last().expect("lib name is empty").to_owned(),
                rename: None,
                modifiers: vec![],
            },
            _ => panic!("Invalid library specification: {}", s),
        }
    }
}

impl std::fmt::Display for Library {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let modifiers = if self.modifiers.is_empty() {
            String::new()
        } else {
            format!(":{}", self.modifiers.join(","))
        };

        let rename = if let Some(rename) = &self.rename {
            format!("{}:", rename)
        } else {
            String::new()
        };

        match self.kind {
            LibraryKind::Framework => write!(f, "framework={}", self.name),
            LibraryKind::Dylib => write!(f, "dylib{}={}{}", modifiers, rename, self.name),
            LibraryKind::Static => write!(f, "static{}={}{}", modifiers, rename, self.name),
            LibraryKind::Native => write!(f, "{}{}", rename, self.name),
        }
    }
}
