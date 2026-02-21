use std::fmt;
use std::str::FromStr;

/// A Cargo build profile.
#[derive(Debug, Clone)]
pub enum Profile {
    Release,
    Dev,
    Custom(String),
}

impl Profile {
    /// The directory name under `target/<triple>/` for this profile.
    pub fn dir_name(&self) -> &str {
        match self {
            Profile::Release => "release",
            Profile::Dev => "debug",
            Profile::Custom(name) => name.as_str(),
        }
    }

    /// The cargo CLI args for this profile.
    pub fn cargo_args(&self) -> Vec<&str> {
        match self {
            Profile::Release => vec!["--release"],
            Profile::Dev => vec![],
            Profile::Custom(name) => vec!["--profile", name.as_str()],
        }
    }
}

impl FromStr for Profile {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "release" => Profile::Release,
            "dev" => Profile::Dev,
            other => Profile::Custom(other.to_string()),
        })
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Profile::Release => write!(f, "release"),
            Profile::Dev => write!(f, "dev"),
            Profile::Custom(name) => write!(f, "{name}"),
        }
    }
}
