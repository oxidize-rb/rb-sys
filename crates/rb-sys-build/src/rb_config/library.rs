use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
};

/// A library that is linked to by the Ruby interpreter.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Library {
    Framework(Libname),
    Dylib(Libname),
    Static(Libname),
    Unknown(Libname),
}

impl Library {
    pub fn is_static(&self) -> bool {
        matches!(self, Self::Static(_))
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Framework(name) => name.as_ref(),
            Self::Dylib(name) => name.as_ref(),
            Self::Static(name) => name.as_ref(),
            Self::Unknown(name) => name.as_ref(),
        }
    }

    pub fn kind(&self) -> Option<&'static str> {
        match self {
            Self::Framework(_) => Some("framework"),
            Self::Dylib(_) => Some("dylib"),
            Self::Static(_) => Some("static"),
            Self::Unknown(_) => None,
        }
    }

    pub fn to_cargo_directive(&self) -> String {
        match self {
            Library::Framework(name) => format!("framework={}", name),
            Library::Dylib(name) => format!("dylib={}", name),
            Library::Static(name) => format!("static={}", name),
            Library::Unknown(name) => name.to_string(),
        }
    }
}

impl From<&str> for Library {
    fn from(s: &str) -> Self {
        let parts: Vec<_> = s.splitn(2, '=').collect();

        match parts.len() {
            1 => Library::Unknown(Libname::new(parts[0])),
            2 => (parts[0], parts[1]).into(),
            _ => panic!("Invalid library specification: {}", s),
        }
    }
}

impl From<String> for Library {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

impl From<(&str, &str)> for Library {
    fn from((kind, name): (&str, &str)) -> Self {
        match kind {
            "framework" => Library::Framework(name.into()),
            "dylib" => Library::Dylib(name.into()),
            "static" => Library::Static(name.into()),
            _ => Library::Unknown(name.into()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Libname(String);

impl Libname {
    pub fn new<T: AsRef<str>>(name: T) -> Self {
        let name = name
            .as_ref()
            .trim_end_matches(".lib")
            .trim_start_matches("-l");

        Self(name.to_string())
    }
}

impl AsRef<str> for Libname {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Display for Libname {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Libname {
    fn from(s: &str) -> Self {
        Self::new(Cow::Borrowed(s))
    }
}

impl From<String> for Libname {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_leading_link_flag() {
        let result: Library = "-lfoo".to_string().into();

        assert_eq!(result.name(), "foo");
    }

    #[test]
    fn test_trim_trailing_lib_extension() {
        let result: Library = "foo.lib".to_string().into();

        assert_eq!(result.name(), "foo");
    }

    #[test]
    fn test_trim_leading_link_flag_and_trailing_lib_extension() {
        let result: Library = "-lfoo.lib".to_string().into();

        assert_eq!(result.name(), "foo");
    }

    #[test]
    fn test_display_framework() {
        let result: Library = "framework=foo".to_string().into();

        assert_eq!(result.to_cargo_directive(), "framework=foo");
    }

    #[test]
    fn test_display_dylib() {
        let result: Library = "dylib=foo".to_string().into();

        assert_eq!(result.to_cargo_directive(), "dylib=foo");
    }

    #[test]
    fn test_display_static() {
        let result: Library = "static=-lfoo".to_string().into();

        assert_eq!(result.to_cargo_directive(), "static=foo");
    }

    #[test]
    fn test_display_none() {
        let result: Library = "foo".to_string().into();

        assert_eq!(result.to_cargo_directive(), "foo");
    }
}
