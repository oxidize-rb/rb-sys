use std::collections::HashMap;

const COMPARABLE_RUBY_MAJORS: [u8; 4] = [1, 2, 3, 4];

const COMPARABLE_RUBY_MINORS: [(u8, u8); 10] = [
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
        rustc_cfg!("ruby_{}", self.major);
        rustc_cfg!("ruby_{}_{}", self.major, self.minor);
        rustc_cfg!("ruby_{}_{}_{}", self.major, self.minor, self.teeny);

        for v in &COMPARABLE_RUBY_MINORS {
            if self.major_minor() < *v {
                rustc_cfg!(r#"ruby_lt_{}_{}"#, v.0, v.1);
            }
            if self.major_minor() <= *v {
                rustc_cfg!(r#"ruby_lte_{}_{}"#, v.0, v.1);
            }
            if self.major_minor() == *v {
                rustc_cfg!(r#"ruby_{}_{}"#, v.0, v.1);
                rustc_cfg!(r#"ruby_eq_{}_{}"#, v.0, v.1);
            }
            if self.major_minor() >= *v {
                rustc_cfg!(r#"ruby_gte_{}_{}"#, v.0, v.1);
            }
            if self.major_minor() > *v {
                rustc_cfg!(r#"ruby_gt_{}_{}"#, v.0, v.1);
            }
        }

        for v in &COMPARABLE_RUBY_MAJORS {
            if self.major() < *v {
                rustc_cfg!(r#"ruby_lt_{}"#, v);
            }
            if self.major() <= *v {
                rustc_cfg!(r#"ruby_lte_{}"#, v);
            }
            if self.major() == *v {
                rustc_cfg!(r#"ruby_{}"#, v);
                rustc_cfg!(r#"ruby_eq_{}"#, v);
            }
            if self.major() >= *v {
                rustc_cfg!(r#"ruby_gte_{}"#, v);
            }
            if self.major() > *v {
                rustc_cfg!(r#"ruby_gt_{}"#, v);
            }
        }
    }
}

impl From<u8> for RubyVersion {
    fn from(major: u8) -> Self {
        Self {
            major: major as u8,
            minor: 0,
            teeny: 0,
        }
    }
}

impl From<(u8, u8)> for RubyVersion {
    fn from((major, minor): (u8, u8)) -> Self {
        Self {
            major: major as u8,
            minor: minor as u8,
            teeny: 0,
        }
    }
}

impl From<(u8, u8, u8)> for RubyVersion {
    fn from((major, minor, teeny): (u8, u8, u8)) -> Self {
        Self {
            major: major as u8,
            minor: minor as u8,
            teeny: teeny as u8,
        }
    }
}

fn verpart(env: &HashMap<String, String>, key: &str) -> u8 {
    env.get(key).unwrap().parse().unwrap()
}

impl RubyVersion {
    pub(crate) fn from_raw_environment(env: &HashMap<String, String>) -> Self {
        Self {
            major: verpart(env, "MAJOR"),
            minor: verpart(env, "MINOR"),
            teeny: verpart(env, "TEENY"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
