/// Represents the kind of search path.
#[derive(Debug, PartialEq, Eq)]
pub enum SearchPathKind {
    Native,
    Framework,
}

/// Represents a library taht can be linked with Cargo.
#[derive(Debug, PartialEq, Eq)]
pub struct SearchPath {
    pub kind: SearchPathKind,
    pub name: String,
}

impl From<&str> for SearchPathKind {
    fn from(s: &str) -> Self {
        match s {
            "framework" => SearchPathKind::Framework,
            "native" => SearchPathKind::Native,
            _ => panic!("Unknown lib kind: {}", s),
        }
    }
}

impl From<&str> for SearchPath {
    fn from(s: &str) -> Self {
        let parts: Vec<_> = s.split('=').map(|s| s.to_owned()).collect();

        match parts.len() {
            1 => Self {
                kind: SearchPathKind::Native,
                name: parts
                    .first()
                    .expect("search path is empty")
                    .to_owned()
                    .to_owned(),
            },
            2 => Self {
                kind: parts.first().expect("no kind for lib").as_str().into(),
                name: parts
                    .last()
                    .expect("search path is empty")
                    .to_owned()
                    .to_owned(),
            },
            _ => panic!("Invalid library specification: {}", s),
        }
    }
}

impl std::fmt::Display for SearchPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            SearchPathKind::Framework => write!(f, "framework={}", self.name),
            SearchPathKind::Native => write!(f, "native={}", self.name),
        }
    }
}
