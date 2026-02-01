use std::collections::HashMap;

const COMPARABLE_RUBY_MAJORS: [u8; 4] = [1, 2, 3, 4];

const COMPARABLE_RUBY_MINORS: [(u8, u8); 13] = [
    (2, 2),
    (2, 3),
    (2, 4),
    (2, 5),
    (2, 6),
    (2, 7),
    (3, 0),
    (3, 1),
    (3, 2),
    (3, 3),
    (3, 4),
    (4, 0),
    (4, 1),
];

/// The current Ruby version.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RubyVersion {
    major: u8,
    minor: u8,
    teeny: u8,
}

impl RubyVersion {
    /// The Ruby major version.
    pub fn major(&self) -> u8 {
        self.major
    }

    /// The Ruby minor version.
    pub fn minor(&self) -> u8 {
        self.minor
    }

    /// The Ruby teeny version.
    pub fn teeny(&self) -> u8 {
        self.teeny
    }

    /// The Ruby version as a u8 triple.
    pub fn major_minor_teeny(&self) -> (u8, u8, u8) {
        (self.major, self.minor, self.teeny)
    }

    /// The Ruby version as a u8 pair.
    pub fn major_minor(&self) -> (u8, u8) {
        (self.major, self.minor)
    }

    pub fn print_cargo_rustc_cfg(&self) {
        rustc_cfg!(true, "ruby_{}", self.major);
        rustc_cfg!(true, "ruby_{}_{}", self.major, self.minor);
        rustc_cfg!(true, "ruby_{}_{}_{}", self.major, self.minor, self.teeny);

        for v in &COMPARABLE_RUBY_MINORS {
            rustc_cfg!(self.major_minor() < *v, r#"ruby_lt_{}_{}"#, v.0, v.1);
            rustc_cfg!(self.major_minor() <= *v, r#"ruby_lte_{}_{}"#, v.0, v.1);
            rustc_cfg!(self.major_minor() == *v, r#"ruby_{}_{}"#, v.0, v.1);
            rustc_cfg!(self.major_minor() == *v, r#"ruby_eq_{}_{}"#, v.0, v.1);
            rustc_cfg!(self.major_minor() >= *v, r#"ruby_gte_{}_{}"#, v.0, v.1);
            rustc_cfg!(self.major_minor() > *v, r#"ruby_gt_{}_{}"#, v.0, v.1);
        }

        for v in &COMPARABLE_RUBY_MAJORS {
            rustc_cfg!(self.major() < *v, r#"ruby_lt_{}"#, v);
            rustc_cfg!(self.major() <= *v, r#"ruby_lte_{}"#, v);
            rustc_cfg!(self.major() == *v, r#"ruby_{}"#, v);
            rustc_cfg!(self.major() == *v, r#"ruby_eq_{}"#, v);
            rustc_cfg!(self.major() >= *v, r#"ruby_gte_{}"#, v);
            rustc_cfg!(self.major() > *v, r#"ruby_gt_{}"#, v);
        }
    }
}

impl From<u8> for RubyVersion {
    fn from(major: u8) -> Self {
        Self {
            major,
            minor: 0,
            teeny: 0,
        }
    }
}

impl From<(u8, u8)> for RubyVersion {
    fn from((major, minor): (u8, u8)) -> Self {
        Self {
            major,
            minor,
            teeny: 0,
        }
    }
}

impl From<(u8, u8, u8)> for RubyVersion {
    fn from((major, minor, teeny): (u8, u8, u8)) -> Self {
        Self {
            major,
            minor,
            teeny,
        }
    }
}

impl RubyVersion {
    pub(crate) fn from_raw_environment(env: &HashMap<String, String>) -> Self {
        match (env.get("MAJOR"), env.get("MINOR"), env.get("TEENY")) {
            (Some(major), Some(minor), Some(teeny)) => {
                let major = major.parse().expect("MAJOR is not a number");
                let minor = minor.parse().expect("MINOR is not a number");
                let teeny = teeny.parse().expect("TEENY is not a number");

                Self {
                    major,
                    minor,
                    teeny,
                }
            }
            _ => {
                let env_ruby_version = env.get("ruby_version").cloned().unwrap_or_else(|| {
                    std::env::var("RUBY_VERSION").expect("RUBY_VERSION is not set")
                });

                let mut ruby_version = env_ruby_version
                    .split('.')
                    .map(|s| parse_version_component(s, "RUBY_VERSION"));

                Self {
                    major: ruby_version.next().expect("major"),
                    minor: ruby_version.next().expect("minor"),
                    teeny: ruby_version.next().expect("teeny"),
                }
            }
        }
    }
}

fn parse_version_component(value: &str, name: &str) -> u8 {
    let digits: String = value.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        panic!("{} is not a number", name);
    }
    digits.parse().expect("version component is not a number")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_equality_from_tuple() {
        assert_eq!(RubyVersion::from((3, 0, 0)), RubyVersion::from((3, 0)));
        assert_ne!(RubyVersion::from((3, 0, 1)), RubyVersion::from((3, 0)));
    }

    #[test]
    fn test_from_hashmap() {
        let mut env = HashMap::new();
        env.insert("MAJOR".to_string(), "3".to_string());
        env.insert("MINOR".to_string(), "0".to_string());
        env.insert("TEENY".to_string(), "0".to_string());

        assert_eq!(
            RubyVersion::from_raw_environment(&env),
            RubyVersion::from((3, 0, 0))
        );
    }

    #[test]
    fn test_from_hashmap_with_ruby_version_suffix() {
        let mut env = HashMap::new();
        env.insert("ruby_version".to_string(), "4.1.0+1".to_string());

        assert_eq!(
            RubyVersion::from_raw_environment(&env),
            RubyVersion::from((4, 1, 0))
        );
    }

    #[test]
    fn test_from_env_with_ruby_version_suffix() {
        let _guard = ENV_LOCK.lock().unwrap();
        let previous = std::env::var("RUBY_VERSION").ok();

        std::env::set_var("RUBY_VERSION", "4.1.0+1");
        let env = HashMap::new();

        let parsed = RubyVersion::from_raw_environment(&env);

        if let Some(value) = previous {
            std::env::set_var("RUBY_VERSION", value);
        } else {
            std::env::remove_var("RUBY_VERSION");
        }

        assert_eq!(parsed, RubyVersion::from((4, 1, 0)));
    }

    #[test]
    fn test_from_hashmap_rejects_major_suffix() {
        let mut env = HashMap::new();
        env.insert("MAJOR".to_string(), "4+1".to_string());
        env.insert("MINOR".to_string(), "1".to_string());
        env.insert("TEENY".to_string(), "0".to_string());

        let result = std::panic::catch_unwind(|| RubyVersion::from_raw_environment(&env));

        assert!(result.is_err());
    }
}
