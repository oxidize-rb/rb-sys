use anyhow::{Context, Result};
use ruby_prism::{parse, Node};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::warn;

/// Serialized rbconfig format for JSON storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedRbConfig {
    pub prefix: String,
    pub config: HashMap<String, String>,
}

/// Parses rbconfig.rb files to extract Ruby configuration values
#[derive(Debug, Default)]
pub struct RbConfigParser {
    config: HashMap<String, String>,
}

impl RbConfigParser {
    /// Create a new parser instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse an rbconfig.rb file from a path
    pub fn from_file(path: &Path) -> Result<Self> {
        let source = std::fs::read(path)
            .with_context(|| format!("Failed to read rbconfig.rb from {}", path.display()))?;

        let mut parser = Self::new();
        parser.evaluate(&source);

        if parser.config.is_empty() {
            anyhow::bail!("No configuration values found in rbconfig.rb");
        }

        // Compute prefix from the rbconfig.rb file path and interpolate variables
        if let Some(prefix) = Self::compute_prefix(path) {
            parser.interpolate_variables(&prefix);
        } else {
            warn!(path = %path.display(), "Could not compute prefix from rbconfig.rb path");
        }

        Ok(parser)
    }

    /// Parse rbconfig.rb source code
    pub fn evaluate(&mut self, source: &[u8]) {
        let result = parse(source);
        self.visit(&result.node());
    }

    /// Visit an AST node and extract CONFIG assignments
    fn visit(&mut self, node: &Node<'_>) {
        // Handle ProgramNode -> get statements
        if let Some(prog) = node.as_program_node() {
            let statements = prog.statements();
            self.visit(&statements.as_node());
            return;
        }

        // Handle ModuleNode -> get body
        if let Some(module) = node.as_module_node() {
            if let Some(body) = module.body() {
                self.visit(&body);
            }
            return;
        }

        // Handle StatementsNode -> iterate body
        if let Some(stmts) = node.as_statements_node() {
            for statement in &stmts.body() {
                self.visit(&statement);
            }
            return;
        }

        // Handle CallNode -> extract CONFIG["key"] = "value"
        if let Some(call) = node.as_call_node() {
            // Pattern: CONFIG["key"] = "value" becomes CONFIG.[]=("key", "value")
            let name_id = call.name();
            let name_bytes = name_id.as_slice();

            if name_bytes != b"[]=" {
                return;
            }

            // Check if receiver is CONFIG constant
            let is_config_receiver = call
                .receiver()
                .and_then(|r| r.as_constant_read_node())
                .map(|c| c.name().as_slice() == b"CONFIG")
                .unwrap_or(false);

            if is_config_receiver {
                if let Some(args_node) = call.arguments() {
                    let args: Vec<_> = args_node.arguments().iter().collect();
                    if args.len() == 2 {
                        let key = self.extract_string(&args[0]);
                        let val = self.extract_string(&args[1]);
                        if let (Some(k), Some(v)) = (key, val) {
                            self.config.insert(k, v);
                        }
                    }
                }
            }
        }
    }

    /// Extract a string value from an AST node
    fn extract_string(&self, node: &Node<'_>) -> Option<String> {
        node.as_string_node()
            .map(|str_node| String::from_utf8_lossy(str_node.unescaped()).to_string())
    }

    /// Get a single configuration value
    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<&String> {
        self.config.get(key)
    }

    /// Get all configuration values as a HashMap
    #[allow(dead_code)]
    pub fn as_hash(&self) -> &HashMap<String, String> {
        &self.config
    }

    /// Convert to serializable format
    pub fn to_serialized(&self, prefix: &str) -> SerializedRbConfig {
        SerializedRbConfig {
            prefix: prefix.to_string(),
            config: self.config.clone(),
        }
    }

    /// Load from JSON file and re-interpolate paths at build time
    ///
    /// This method re-computes the prefix from the JSON file's location,
    /// making the paths portable across machines and cache locations.
    #[allow(dead_code)]
    pub fn from_json(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let serialized: SerializedRbConfig = serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse JSON from {}", path.display()))?;

        let mut parser = Self {
            config: serialized.config,
        };

        // CRITICAL: Re-compute prefix from JSON file location at build time
        // This makes paths portable across machines and cache locations
        if let Some(prefix) = Self::compute_prefix(path) {
            parser.interpolate_variables(&prefix);
        } else {
            anyhow::bail!(
                "Could not compute prefix from rbconfig.json path: {}. \
                Expected path structure: .../ruby-X.Y.Z/lib/ruby/X.Y.Z/arch/rbconfig.json",
                path.display()
            );
        }

        Ok(parser)
    }

    /// Compute the prefix path from the rbconfig.rb file location
    ///
    /// Given: /path/to/ruby-3.1.0/lib/ruby/3.1.0/aarch64-linux-gnu/rbconfig.rb
    /// Returns: /path/to/ruby-3.1.0
    pub fn compute_prefix(rbconfig_path: &Path) -> Option<String> {
        // Get the parent directories going up from rbconfig.rb
        // Expected structure: prefix/lib/ruby/{version}/{arch}/rbconfig.rb
        let path_str = rbconfig_path.to_string_lossy();

        // Find /lib/ruby/ and everything before it is the prefix
        if let Some(lib_ruby_pos) = path_str.find("/lib/ruby/") {
            return Some(path_str[..lib_ruby_pos].to_string());
        }

        None
    }

    /// Interpolate makefile-style variables in config values
    ///
    /// Expands $(var_name) references to their actual values through iterative substitution.
    /// Variables that reference undefined values are left unexpanded.
    fn interpolate_variables(&mut self, prefix: &str) {
        // Override prefix with the computed value from file path
        self.config.insert("prefix".to_string(), prefix.to_string());

        // Compile regex for finding $(var) patterns
        let var_regex = regex::Regex::new(r"\$\(([^)]+)\)").unwrap();

        // Maximum iterations to prevent infinite loops
        const MAX_ITERATIONS: usize = 10;

        for _iteration in 0..MAX_ITERATIONS {
            let mut substitutions_made = 0;
            let mut new_config = HashMap::new();

            // Try to expand variables in each config value
            for (key, value) in &self.config {
                if !value.contains("$(") {
                    // No variables to expand, keep as-is
                    new_config.insert(key.clone(), value.clone());
                    continue;
                }

                let mut expanded = value.clone();

                // Find all $(var) references in this value
                for cap in var_regex.captures_iter(value) {
                    let var_name = &cap[1];

                    // Look up the variable in config
                    if let Some(var_value) = self.config.get(var_name) {
                        // Only substitute if the variable value doesn't contain $(
                        // This ensures we expand from bottom-up
                        if !var_value.contains("$(") {
                            let pattern = format!("$({})", var_name);
                            expanded = expanded.replace(&pattern, var_value);
                            substitutions_made += 1;
                        }
                    }
                }

                new_config.insert(key.clone(), expanded);
            }

            // Update config with expanded values
            self.config = new_config;

            // If no substitutions were made, we're done
            if substitutions_made == 0 {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_config() {
        let source = br#"
module RbConfig
  CONFIG = {}
  CONFIG["MAJOR"] = "3"
  CONFIG["MINOR"] = "3"
  CONFIG["ruby_version"] = "3.3.0"
end
"#;

        let mut parser = RbConfigParser::new();
        parser.evaluate(source);

        assert_eq!(parser.get("MAJOR"), Some(&"3".to_string()));
        assert_eq!(parser.get("MINOR"), Some(&"3".to_string()));
        assert_eq!(parser.get("ruby_version"), Some(&"3.3.0".to_string()));
    }

    #[test]
    fn test_empty_source() {
        let source = b"";
        let mut parser = RbConfigParser::new();
        parser.evaluate(source);
        assert_eq!(parser.as_hash().len(), 0);
    }

    #[test]
    fn test_compute_prefix() {
        // Standard path
        let path = Path::new("/cache/rubies/aarch64-linux-gnu/ruby-3.1.0/lib/ruby/3.1.0/aarch64-linux-gnu/rbconfig.rb");
        assert_eq!(
            RbConfigParser::compute_prefix(path),
            Some("/cache/rubies/aarch64-linux-gnu/ruby-3.1.0".to_string())
        );

        // RC version with + suffix
        let path = Path::new("/usr/local/rake-compiler/ruby/aarch64-linux-gnu/ruby-3.3.0-rc1/lib/ruby/3.3.0+0/aarch64-linux-gnu/rbconfig.rb");
        assert_eq!(
            RbConfigParser::compute_prefix(path),
            Some("/usr/local/rake-compiler/ruby/aarch64-linux-gnu/ruby-3.3.0-rc1".to_string())
        );

        // Invalid path - no /lib/ruby/
        let path = Path::new("/some/random/path/rbconfig.rb");
        assert_eq!(RbConfigParser::compute_prefix(path), None);
    }

    #[test]
    fn test_interpolate_variables() {
        let source = br#"
module RbConfig
  CONFIG = {}
  CONFIG["prefix"] = "/usr/local"
  CONFIG["exec_prefix"] = "$(prefix)"
  CONFIG["libdir"] = "$(exec_prefix)/lib"
  CONFIG["RUBY_BASE_NAME"] = "ruby"
  CONFIG["ruby_version"] = "3.1.0"
  CONFIG["RUBY_VERSION_NAME"] = "$(RUBY_BASE_NAME)-$(ruby_version)"
  CONFIG["includedir"] = "$(prefix)/include"
  CONFIG["rubyhdrdir"] = "$(includedir)/$(RUBY_VERSION_NAME)"
  CONFIG["arch"] = "aarch64-linux-gnu"
  CONFIG["rubyarchhdrdir"] = "$(rubyhdrdir)/$(arch)"
end
"#;

        let mut parser = RbConfigParser::new();
        parser.evaluate(source);

        // Interpolate with a test prefix
        parser.interpolate_variables("/test/prefix");

        // Check that variables were expanded
        assert_eq!(parser.get("prefix"), Some(&"/test/prefix".to_string()));
        assert_eq!(parser.get("exec_prefix"), Some(&"/test/prefix".to_string()));
        assert_eq!(parser.get("libdir"), Some(&"/test/prefix/lib".to_string()));
        assert_eq!(
            parser.get("RUBY_VERSION_NAME"),
            Some(&"ruby-3.1.0".to_string())
        );
        assert_eq!(
            parser.get("includedir"),
            Some(&"/test/prefix/include".to_string())
        );
        assert_eq!(
            parser.get("rubyhdrdir"),
            Some(&"/test/prefix/include/ruby-3.1.0".to_string())
        );
        assert_eq!(
            parser.get("rubyarchhdrdir"),
            Some(&"/test/prefix/include/ruby-3.1.0/aarch64-linux-gnu".to_string())
        );
    }

    #[test]
    fn test_interpolate_undefined_variables() {
        let source = br#"
module RbConfig
  CONFIG = {}
  CONFIG["some_path"] = "$(UNDEFINED_VAR)/path"
  CONFIG["other_path"] = "/absolute/path"
end
"#;

        let mut parser = RbConfigParser::new();
        parser.evaluate(source);
        parser.interpolate_variables("/test/prefix");

        // Undefined variables should remain unexpanded
        assert_eq!(
            parser.get("some_path"),
            Some(&"$(UNDEFINED_VAR)/path".to_string())
        );
        assert_eq!(
            parser.get("other_path"),
            Some(&"/absolute/path".to_string())
        );
    }

    #[test]
    fn test_interpolate_complex_expressions() {
        let source = br#"
module RbConfig
  CONFIG = {}
  CONFIG["CC"] = "gcc"
  CONFIG["DLDSHARED"] = "$(CC) -shared"
  CONFIG["RUBY_BASE_NAME"] = "ruby"
  CONFIG["libdir"] = "/usr/lib"
  CONFIG["LIBRUBYARG"] = "-L$(libdir) -l$(RUBY_BASE_NAME)"
end
"#;

        let mut parser = RbConfigParser::new();
        parser.evaluate(source);
        parser.interpolate_variables("/test/prefix");

        // Check that multiple variables in one value are expanded
        assert_eq!(parser.get("DLDSHARED"), Some(&"gcc -shared".to_string()));
        assert_eq!(
            parser.get("LIBRUBYARG"),
            Some(&"-L/usr/lib -lruby".to_string())
        );
    }

    #[test]
    fn test_from_json_recomputes_prefix() {
        use tempfile::tempdir;

        // Create a test directory structure mimicking the cache layout
        let temp_dir = tempdir().unwrap();
        let ruby_dir = temp_dir
            .path()
            .join("ruby-3.4.5")
            .join("lib")
            .join("ruby")
            .join("3.4.0")
            .join("x86_64-linux-gnu");
        std::fs::create_dir_all(&ruby_dir).unwrap();

        // Create a JSON file with OLD absolute paths (simulating a moved cache)
        let json_path = ruby_dir.join("rbconfig.json");
        let json_content = r#"{
            "prefix": "/old/machine/path/ruby-3.4.5",
            "config": {
                "prefix": "/old/machine/path/ruby-3.4.5",
                "exec_prefix": "$(prefix)",
                "bindir": "$(exec_prefix)/bin",
                "libdir": "$(exec_prefix)/lib",
                "includedir": "$(prefix)/include",
                "arch": "x86_64-linux-gnu",
                "rubyhdrdir": "$(includedir)/ruby-3.4.0",
                "rubyarchhdrdir": "$(rubyhdrdir)/$(arch)"
            }
        }"#;
        std::fs::write(&json_path, json_content).unwrap();

        // Load from JSON - should re-compute prefix from file location
        let parser = RbConfigParser::from_json(&json_path).unwrap();

        // The new prefix should be computed from the JSON file's location
        let expected_prefix = temp_dir
            .path()
            .join("ruby-3.4.5")
            .to_string_lossy()
            .to_string();

        // Verify prefix was re-computed
        assert_eq!(parser.get("prefix"), Some(&expected_prefix));

        // Verify all paths were re-interpolated with the new prefix
        assert_eq!(parser.get("exec_prefix"), Some(&expected_prefix));
        assert_eq!(
            parser.get("bindir"),
            Some(&format!("{}/bin", expected_prefix))
        );
        assert_eq!(
            parser.get("libdir"),
            Some(&format!("{}/lib", expected_prefix))
        );
        assert_eq!(
            parser.get("includedir"),
            Some(&format!("{}/include", expected_prefix))
        );
        assert_eq!(
            parser.get("rubyhdrdir"),
            Some(&format!("{}/include/ruby-3.4.0", expected_prefix))
        );
        assert_eq!(
            parser.get("rubyarchhdrdir"),
            Some(&format!(
                "{}/include/ruby-3.4.0/x86_64-linux-gnu",
                expected_prefix
            ))
        );

        // Verify arch is preserved (not a path, shouldn't change)
        assert_eq!(parser.get("arch"), Some(&"x86_64-linux-gnu".to_string()));
    }

    #[test]
    fn test_from_json_fails_with_invalid_path() {
        use tempfile::tempdir;

        // Create a JSON file in a location that doesn't match expected structure
        let temp_dir = tempdir().unwrap();
        let json_path = temp_dir.path().join("rbconfig.json");
        let json_content = r#"{"prefix": "/test", "config": {"key": "value"}}"#;
        std::fs::write(&json_path, json_content).unwrap();

        // Should fail because path doesn't contain /lib/ruby/
        let result = RbConfigParser::from_json(&json_path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Could not compute prefix"));
    }

    #[test]
    #[cfg_attr(not(target_os = "macos"), ignore)]
    fn test_real_rbconfig_if_exists() {
        // This test only runs on macOS where we know the cache location
        let rbconfig_path = std::path::PathBuf::from(
            std::env::var("HOME").unwrap() + 
            "/.cache/rb-sys/rubies/aarch64-linux-gnu/ruby-3.3.0-rc1/lib/ruby/3.3.0+0/aarch64-linux-gnu/rbconfig.rb"
        );

        if !rbconfig_path.exists() {
            println!("Skipping test - rbconfig.rb not found");
            return;
        }

        // Parse the real rbconfig.rb
        let parser =
            RbConfigParser::from_file(&rbconfig_path).expect("Failed to parse rbconfig.rb");

        // Verify key values are interpolated (no $(...) should remain)
        let rubyhdrdir = parser.get("rubyhdrdir").expect("rubyhdrdir not found");
        let rubyarchhdrdir = parser
            .get("rubyarchhdrdir")
            .expect("rubyarchhdrdir not found");
        let includedir = parser.get("includedir").expect("includedir not found");
        let prefix = parser.get("prefix").expect("prefix not found");

        // These should all be absolute paths without $(...)
        assert!(
            !rubyhdrdir.contains("$("),
            "rubyhdrdir still has variables: {}",
            rubyhdrdir
        );
        assert!(
            !rubyarchhdrdir.contains("$("),
            "rubyarchhdrdir still has variables: {}",
            rubyarchhdrdir
        );
        assert!(
            !includedir.contains("$("),
            "includedir still has variables: {}",
            includedir
        );
        assert!(
            rubyhdrdir.starts_with("/"),
            "rubyhdrdir is not absolute: {}",
            rubyhdrdir
        );
        assert!(
            rubyarchhdrdir.starts_with("/"),
            "rubyarchhdrdir is not absolute: {}",
            rubyarchhdrdir
        );

        // Verify the structure is correct
        assert!(
            rubyhdrdir.contains("include/ruby-"),
            "rubyhdrdir doesn't contain expected path"
        );
        assert!(
            rubyarchhdrdir.contains("aarch64-linux-gnu"),
            "rubyarchhdrdir doesn't contain arch"
        );

        println!("✓ Interpolation successful:");
        println!("  prefix: {}", prefix);
        println!("  includedir: {}", includedir);
        println!("  rubyhdrdir: {}", rubyhdrdir);
        println!("  rubyarchhdrdir: {}", rubyarchhdrdir);
    }
}
