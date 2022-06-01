/// Represents the kind of library.
#[derive(Debug, PartialEq, Eq)]
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
                name: parts
                    .first()
                    .expect("lib name is empty")
                    .to_owned()
                    .to_owned(),
            },
            2 => Library {
                kind: parts.first().expect("no kind for lib").as_str().into(),
                name: parts
                    .last()
                    .expect("lib name is empty")
                    .to_owned()
                    .to_owned(),
            },
            _ => panic!("Invalid library specification: {}", s),
        }
    }
}

impl std::fmt::Display for Library {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.kind {
            LibraryKind::Framework => write!(f, "framework={}", self.name),
            LibraryKind::Dylib => write!(f, "dylib={}", self.name),
            LibraryKind::Static => write!(f, "static={}", self.name),
            LibraryKind::Native => write!(f, "native={}", self.name),
        }
    }
}
