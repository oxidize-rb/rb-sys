/// Represents the kind of library.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LibraryKind {
    Framework,
    Dylib,
    Static,
    None,
}

/// Represents a search path that can be linked with Cargo.
#[derive(Debug, PartialEq, Eq)]
pub struct Library {
    pub kind: LibraryKind,
    pub name: String,
    pub modifiers: Vec<String>,
}

impl Library {
    pub fn is_static(&self) -> bool {
        self.kind == LibraryKind::Static
    }
}

impl From<&str> for LibraryKind {
    fn from(s: &str) -> Self {
        match s {
            "framework" => LibraryKind::Framework,
            "dylib" => LibraryKind::Dylib,
            "static" => LibraryKind::Static,
            _ => LibraryKind::None,
        }
    }
}

impl From<&str> for Library {
    fn from(s: &str) -> Self {
        let parts: Vec<_> = s.splitn(2, '=').collect();

        match parts.len() {
            1 => (LibraryKind::None, sanitize_library_name(parts[0])).into(),
            2 => (parts[0], sanitize_library_name(parts[1])).into(),
            _ => panic!("Invalid library specification: {}", s),
        }
    }
}

impl From<String> for Library {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

fn sanitize_library_name(name: &str) -> &str {
    name.trim_end_matches(".lib")
}

impl<K, L> From<(K, L)> for Library
where
    K: Into<LibraryKind>,
    L: Into<String>,
{
    fn from((kind, name): (K, L)) -> Self {
        Self {
            kind: kind.into(),
            name: name.into(),
            modifiers: vec![],
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

        match self.kind {
            LibraryKind::Framework => write!(f, "framework={}", self.name),
            LibraryKind::Dylib => write!(f, "dylib{}={}", modifiers, self.name),
            LibraryKind::Static => write!(f, "static{}={}", modifiers, self.name),
            LibraryKind::None => write!(f, "{}", self.name),
        }
    }
}
