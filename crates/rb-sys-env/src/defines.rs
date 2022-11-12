use std::{collections::HashMap, rc::Rc};

/// The DEFINES variables from libruby.
#[derive(Debug, Clone)]
pub struct Defines {
    raw_environment: Rc<HashMap<String, String>>,
}

impl Defines {
    pub(crate) fn from_raw_environment(raw_environment: Rc<HashMap<String, String>>) -> Self {
        Self { raw_environment }
    }

    /// Determines the given key is true.
    pub fn is_value_true(&self, key: &str) -> bool {
        self.raw_environment
            .get(format!("DEFINES_{}", key).as_str())
            .map(|v| v == "1" || v == "true")
            .unwrap_or(false)
    }

    /// Fetches the raw value for the given key.
    pub fn get_raw_value(&self, key: &str) -> Option<&str> {
        self.raw_environment
            .get(format!("DEFINES_{}", key).as_str())
            .map(|v| v.as_str())
    }

    pub(crate) fn print_cargo_rustc_cfg(&self) {
        for (key, val) in self.raw_environment.iter() {
            if key.starts_with("DEFINES_") && val == "true" {
                let key = key.trim_start_matches("DEFINES_");
                rustc_cfg!("ruby_{}", key.to_lowercase());
            }
        }
    }
}
