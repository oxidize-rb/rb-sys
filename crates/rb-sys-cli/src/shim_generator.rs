use anyhow::{Context, Result};
use indoc::formatdoc;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Ruby header directories for cross-compilation
#[derive(Debug)]
struct RubyHeaders {
    rubyhdrdir: String,
    rubyarchhdrdir: String,
    rbconfig_path: Option<PathBuf>,
}

/// Generates compiler shims that wrap zig cc for cross-compilation
pub struct ShimGenerator {
    /// The directory where shims (scripts/binaries) will be placed
    pub shim_dir: PathBuf,
    /// The specific Zig executable to wrap
    pub zig_path: PathBuf,
}

impl ShimGenerator {
    pub fn new(shim_dir: PathBuf, zig_path: PathBuf) -> Self {
        Self {
            shim_dir,
            zig_path,
        }
    }

    /// Prepare the shim directory by cleaning and creating it
    pub fn prepare(&self) -> Result<()> {
        if self.shim_dir.exists() {
            fs::remove_dir_all(&self.shim_dir)
                .context("Failed to clean existing shim directory")?;
        }
        fs::create_dir_all(&self.shim_dir).context("Failed to create shim directory")?;
        Ok(())
    }

    /// Generate all necessary shims for the current platform
    pub fn generate(&self) -> Result<()> {
        self.prepare()?;

        if cfg!(windows) {
            self.generate_windows_shims()?;
        } else {
            self.generate_unix_shims()?;
        }

        Ok(())
    }

    /// Generate Unix-style bash script shims
    #[cfg(unix)]
    pub fn generate_unix_shims(&self) -> Result<()> {
        let zig_path_str = self
            .zig_path
            .to_str()
            .context("Invalid UTF-8 in zig path")?;

        // Generate CC shim
        let cc_script = self.generate_unix_wrapper_script(zig_path_str, "cc");
        let cc_path = self.shim_dir.join("cc");
        fs::write(&cc_path, cc_script).context("Failed to write cc shim")?;
        Self::make_executable(&cc_path)?;

        // Generate CXX shim
        let cxx_script = self.generate_unix_wrapper_script(zig_path_str, "c++");
        let cxx_path = self.shim_dir.join("cxx");
        fs::write(&cxx_path, cxx_script).context("Failed to write cxx shim")?;
        Self::make_executable(&cxx_path)?;

        // Generate AR shim
        let ar_script = self.generate_unix_wrapper_script(zig_path_str, "ar");
        let ar_path = self.shim_dir.join("ar");
        fs::write(&ar_path, ar_script).context("Failed to write ar shim")?;
        Self::make_executable(&ar_path)?;

        Ok(())
    }

    #[cfg(unix)]
    fn generate_unix_wrapper_script(&self, zig_path: &str, subcommand: &str) -> String {
        formatdoc! {r#"
            #!/usr/bin/env bash
            set -e

            # Debug: Log invocation if RB_SYS_DEBUG_SHIM is set
            if [ -n "$RB_SYS_DEBUG_SHIM" ]; then
                echo "DEBUG: Zig shim {subcommand} called with: $@" >&2
            fi

            # Array to hold cleaned arguments
            declare -a clean_args=()

            # Process arguments
            skip_next=false
            for ((i=1; i<=$#; i++)); do
                if [ "$skip_next" = true ]; then
                    skip_next=false
                    continue
                fi
                
                arg="${{!i}}"
                
                if [ "$arg" = "-target" ]; then
                    # Get the next argument (the triple)
                    j=$((i+1))
                    triple="${{!j}}"
                    
                    # Strip -unknown- from the triple
                    clean_triple="${{triple//-unknown-/-}}"
                    
                    # Append glibc version if set and targeting Linux
                    if [ -n "$GEM_FORGE_GLIBC" ] && [[ "$clean_triple" == *-linux-* ]]; then
                        clean_triple="${{clean_triple}}.${{GEM_FORGE_GLIBC}}"
                    fi
                    
                    clean_args+=("-target" "$clean_triple")
                    skip_next=true
                else
                    clean_args+=("$arg")
                fi
            done

            # Debug: Log final command if RB_SYS_DEBUG_SHIM is set
            if [ -n "$RB_SYS_DEBUG_SHIM" ]; then
                echo "DEBUG: Executing: {zig_path} {subcommand} ${{clean_args[@]}}" >&2
            fi

            # Execute zig with the cleaned arguments
            exec "{zig_path}" {subcommand} "${{clean_args[@]}}"
        "#,
        zig_path = zig_path,
        subcommand = subcommand
        }
    }

    #[cfg(unix)]
    fn make_executable(path: &Path) -> Result<()> {
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms)?;
        Ok(())
    }

    /// Generate Windows binary shims
    #[cfg(windows)]
    pub fn generate_windows_shims(&self) -> Result<()> {
        // Write the shim source code
        let shim_source = self.generate_windows_shim_source()?;
        let shim_source_path = self.shim_dir.join("shim_source.rs");
        fs::write(&shim_source_path, shim_source)
            .context("Failed to write Windows shim source")?;

        // Compile the shim
        let shim_exe_path = self.shim_dir.join("shim.exe");
        let status = Command::new("rustc")
            .arg(&shim_source_path)
            .arg("-o")
            .arg(&shim_exe_path)
            .arg("-O")
            .status()
            .context("Failed to compile Windows shim")?;

        if !status.success() {
            anyhow::bail!("Failed to compile Windows shim binary");
        }

        // Copy to cc.exe, cxx.exe, ar.exe
        for name in &["cc.exe", "cxx.exe", "ar.exe"] {
            let dest = self.shim_dir.join(name);
            fs::copy(&shim_exe_path, &dest)
                .with_context(|| format!("Failed to copy shim to {}", name))?;
        }

        Ok(())
    }

    #[cfg(windows)]
    fn generate_windows_shim_source(&self) -> Result<String> {
        let zig_path = self
            .zig_path
            .to_str()
            .context("Zig path contains invalid UTF-8 characters")?;

        let result = format!(
            r#"
use std::env;
use std::ffi::OsString;
use std::process::{{Command, exit}};

fn main() {{
    let args: Vec<OsString> = env::args_os().collect();
    
    // Determine subcommand based on executable name
    let exe_name = args[0].to_string_lossy();
    let subcommand = if exe_name.contains("ar") {{
        "ar"
    }} else if exe_name.contains("cxx") || exe_name.contains("c++") {{
        "c++"
    }} else {{
        "cc"
    }};
    
    // Skip the first arg (executable name)
    let cli_args = &args[1..];
    
    let mut clean_args = Vec::new();
    let mut skip_next = false;
    
    for (i, arg) in cli_args.iter().enumerate() {{
        if skip_next {{
            skip_next = false;
            continue;
        }}
        
        let s = arg.to_string_lossy();
        
        if s == "-target" {{
            if let Some(triple_os) = cli_args.get(i + 1) {{
                let triple = triple_os.to_string_lossy();
                
                // Strip -unknown- from triple
                let clean_triple = triple.replace("-unknown-", "-");
                
                // Append glibc version if set
                let final_triple = if let Ok(glibc) = env::var("GEM_FORGE_GLIBC") {{
                    if clean_triple.contains("-linux-") {{
                        format!("{{}}.{{}}", clean_triple, glibc)
                    }} else {{
                        clean_triple.to_string()
                    }}
                }} else {{
                    clean_triple.to_string()
                }};
                
                clean_args.push(OsString::from("-target"));
                clean_args.push(OsString::from(final_triple));
                skip_next = true;
            }} else {{
                clean_args.push(arg.clone());
            }}
        }} else {{
            clean_args.push(arg.clone());
        }}
    }}
    
    // Invoke Zig
    let status = Command::new("{zig_path}")
        .arg(subcommand)
        .args(clean_args)
        .status()
        .unwrap_or_else(|e| {{
            eprintln!("Failed to execute zig: {{}}", e);
            exit(1);
        }});
    
    exit(status.code().unwrap_or(1));
}}
"#,
            zig_path = zig_path.replace("\\", "\\\\")
        );
        
        Ok(result)
    }

    // Stub implementations for non-Unix platforms
    #[cfg(not(unix))]
    pub fn generate_unix_shims(&self) -> Result<()> {
        anyhow::bail!("Unix shims cannot be generated on non-Unix platforms")
    }

    #[cfg(not(windows))]
    pub fn generate_windows_shims(&self) -> Result<()> {
        anyhow::bail!("Windows shims cannot be generated on non-Windows platforms")
    }

    /// Get environment variables needed for the build
    pub fn get_shim_env(&self, target_triple: &str, ruby_version: Option<&str>) -> HashMap<String, String> {
        eprintln!("DEBUG: get_shim_env called for target: {}", target_triple);
        let mut env = HashMap::new();

        let cc_path = self.get_shim_path("cc");
        let cxx_path = self.get_shim_path("cxx");
        let ar_path = self.get_shim_path("ar");

        // NOTE: We do NOT set generic CC/CXX/AR because that would affect host builds
        // (like proc-macros) and zig doesn't support all host platform linker flags.
        // Instead, we only set target-specific variables.

        // Target-specific with underscores (primary cc-rs lookup)
        let triple_underscores = target_triple.replace("-", "_");
        env.insert(format!("CC_{}", triple_underscores), cc_path.clone());
        env.insert(format!("CXX_{}", triple_underscores), cxx_path.clone());
        env.insert(format!("AR_{}", triple_underscores), ar_path.clone());

        // Target-specific with dashes (legacy fallback)
        env.insert(format!("CC_{}", target_triple), cc_path.clone());
        env.insert(format!("CXX_{}", target_triple), cxx_path.clone());
        env.insert(format!("AR_{}", target_triple), ar_path.clone());

        // Disable cc-rs defaults to prevent host flag leakage
        env.insert("CRATE_CC_NO_DEFAULTS".to_string(), "1".to_string());

        // Cargo linker configuration
        let triple_upper_underscores = triple_underscores.to_uppercase();
        env.insert(
            format!("CARGO_TARGET_{}_LINKER", triple_upper_underscores),
            cc_path.clone(),
        );
        
        // Pass target triple to the linker so Zig knows it's cross-compiling
        // This is critical for Zig to use its built-in libc instead of searching for system libraries
        let link_args = format!("-C link-arg=-target -C link-arg={}", target_triple.replace("-unknown-", "-"));
        env.insert(
            format!("CARGO_TARGET_{}_RUSTFLAGS", triple_upper_underscores),
            link_args,
        );

        // Configure bindgen to use zig's headers for cross-compilation
        if let Some(zig_include_paths) = self.get_zig_include_paths(target_triple) {
            let mut bindgen_args = Vec::new();
            for path in &zig_include_paths {
                bindgen_args.push(format!("-I{}", path));
            }
            let bindgen_args_str = bindgen_args.join(" ");
            
            // Set both generic and target-specific versions
            env.insert("BINDGEN_EXTRA_CLANG_ARGS".to_string(), bindgen_args_str.clone());
            env.insert(format!("BINDGEN_EXTRA_CLANG_ARGS_{}", triple_underscores), bindgen_args_str.clone());
            env.insert(format!("BINDGEN_EXTRA_CLANG_ARGS_{}", target_triple), bindgen_args_str);
        }

        // Configure Ruby headers from cache
        if let Some(ruby_headers) = self.find_ruby_headers(target_triple, ruby_version) {
            // Parse rbconfig.rb and export ALL configuration values as RBCONFIG_* env vars
            // The rbconfig parser will interpolate variables like $(prefix), $(includedir), etc.
            // to produce absolute paths for rubyhdrdir, rubyarchhdrdir, and other config values.
            if let Some(rbconfig_path) = ruby_headers.rbconfig_path {
                match crate::rbconfig_parser::RbConfigParser::from_file(&rbconfig_path) {
                    Ok(rbconfig) => {
                        for (key, value) in rbconfig.as_hash() {
                            env.insert(format!("RBCONFIG_{}", key), value.clone());
                        }
                        eprintln!("Loaded {} interpolated rbconfig values from {}", 
                            rbconfig.as_hash().len(), 
                            rbconfig_path.display());
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse rbconfig.rb: {}", e);
                        
                        // Fallback: Use computed header paths if rbconfig parsing failed
                        env.insert("RBCONFIG_rubyhdrdir".to_string(), ruby_headers.rubyhdrdir.clone());
                        env.insert("RBCONFIG_rubyarchhdrdir".to_string(), ruby_headers.rubyarchhdrdir);
                    }
                }
            } else {
                eprintln!("Warning: rbconfig.rb not found for target {}", target_triple);
                
                // Fallback: Use computed header paths if rbconfig.rb not found
                env.insert("RBCONFIG_rubyhdrdir".to_string(), ruby_headers.rubyhdrdir.clone());
                env.insert("RBCONFIG_rubyarchhdrdir".to_string(), ruby_headers.rubyarchhdrdir);
            }
        }

        // Set target-appropriate linker flags to override host defaults
        // This prevents macOS-specific flags from being used when cross-compiling to Linux
        if target_triple.contains("linux") {
            // For Linux targets, use -z lazy instead of macOS-specific flags
            env.insert("RBCONFIG_DLDFLAGS".to_string(), "-Wl,-z,lazy".to_string());
        }

        // rb-sys cross-compilation signals
        env.insert("RB_SYS_CROSS_COMPILING".to_string(), "1".to_string());
        env.insert("RBCONFIG_CROSS_COMPILING".to_string(), "yes".to_string());
        env.insert("RUBY_STATIC".to_string(), "true".to_string());

        env
    }

    /// Find Ruby headers in the cache for the given target
    fn find_ruby_headers(&self, target_triple: &str, ruby_version: Option<&str>) -> Option<RubyHeaders> {
        // Import the cache directory function from extractor module
        use crate::extractor::get_cache_dir;
        
        let cache_dir = get_cache_dir().ok()?;
        
        // Map Rust target triple to cache directory name (strips -unknown-)
        let cache_target = target_triple.replace("-unknown-", "-");
        
        let rubies_dir = cache_dir.join("rubies").join(&cache_target);
        
        if !rubies_dir.exists() {
            return None;
        }
        
        // Find Ruby version directory
        let ruby_dir = if let Some(version) = ruby_version {
            // Use specified version
            let dir = rubies_dir.join(format!("ruby-{}", version));
            if dir.exists() { Some(dir) } else { None }
        } else {
            // Auto-detect: find the newest Ruby version
            let mut versions: Vec<_> = std::fs::read_dir(&rubies_dir)
                .ok()?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_type().ok().map(|ft| ft.is_dir()).unwrap_or(false)
                        && e.file_name().to_string_lossy().starts_with("ruby-")
                })
                .collect();
            
            // Sort by name (newer versions sort last)
            versions.sort_by_key(|e| e.file_name());
            
            versions.last().map(|e| e.path())
        }?;
        
        // Find the versioned include directory
        let include_base = ruby_dir.join("include");
        
        if !include_base.exists() {
            return None;
        }
        
        // Look for ruby-X.Y.Z+N directory
        let versioned_include = std::fs::read_dir(&include_base)
            .ok()?
            .filter_map(|e| e.ok())
            .find(|e| {
                e.file_type().ok().map(|ft| ft.is_dir()).unwrap_or(false)
                    && e.file_name().to_string_lossy().starts_with("ruby-")
            })?
            .path();
        
        // The arch-specific headers are in a subdirectory matching the target
        let arch_include = versioned_include.join(&cache_target);
        
        if !arch_include.exists() {
            return None;
        }
        
        // Find rbconfig.rb - search under ruby_dir/lib/ruby/
        let rbconfig_path = Self::find_rbconfig_rb(&ruby_dir, &cache_target);
        
        Some(RubyHeaders {
            rubyhdrdir: versioned_include.to_string_lossy().to_string(),
            rubyarchhdrdir: arch_include.to_string_lossy().to_string(),
            rbconfig_path,
        })
    }

    /// Search for rbconfig.rb under ruby_dir/lib/ruby/
    fn find_rbconfig_rb(ruby_dir: &Path, target: &str) -> Option<PathBuf> {
        let lib_ruby_dir = ruby_dir.join("lib").join("ruby");
        
        if !lib_ruby_dir.exists() {
            return None;
        }
        
        // Search for rbconfig.rb recursively under lib/ruby/
        // Expected paths like: lib/ruby/3.3.0/aarch64-linux/rbconfig.rb
        for entry in std::fs::read_dir(&lib_ruby_dir).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();
            
            if !path.is_dir() {
                continue;
            }
            
            // Look for target-specific subdirectory
            let target_dir = path.join(target);
            if target_dir.exists() {
                let rbconfig = target_dir.join("rbconfig.rb");
                if rbconfig.exists() {
                    return Some(rbconfig);
                }
            }
        }
        
        None
    }

    fn get_zig_include_paths(&self, target_triple: &str) -> Option<Vec<String>> {
        eprintln!("DEBUG: get_zig_include_paths called for target: {}", target_triple);
        eprintln!("DEBUG: zig_path = {:?}", self.zig_path);
        
        // Try to find zig's libc include directory for the target
        let zig_path_str = match self.zig_path.to_str() {
            Some(s) => {
                eprintln!("DEBUG: zig_path converted to string: {}", s);
                s
            }
            None => {
                eprintln!("WARNING: zig_path contains invalid UTF-8: {:?}", self.zig_path);
                return None;
            }
        };
        
        eprintln!("DEBUG: Looking up Zig include paths for target: {}", target_triple);
        eprintln!("DEBUG: Using Zig from: {}", zig_path_str);
        
        // Get zig's lib directory
        let output = match std::process::Command::new(zig_path_str)
            .arg("env")
            .output()
        {
            Ok(output) => output,
            Err(e) => {
                eprintln!("WARNING: Failed to execute 'zig env': {}", e);
                return None;
            }
        };
        
        if !output.status.success() {
            eprintln!("WARNING: 'zig env' exited with non-zero status: {}", output.status);
            eprintln!("  stderr: {}", String::from_utf8_lossy(&output.stderr));
            return None;
        }
        
        let env_output = match String::from_utf8(output.stdout) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("WARNING: 'zig env' output is not valid UTF-8: {}", e);
                return None;
            }
        };
        
        eprintln!("DEBUG: zig env output (first 100 chars): {}", 
            env_output.chars().take(100).collect::<String>());
        
        // Parse the Zig-format output to find lib_dir
        // Format: .lib_dir = "/path/to/zig/lib/zig",
        let lib_dir = env_output
            .lines()
            .find(|line| line.contains(".lib_dir"))
            .and_then(|line| {
                // Extract the path between quotes
                let start = line.find('"')?;
                let end = line[start + 1..].find('"')?;
                Some(line[start + 1..start + 1 + end].to_string())
            });
        
        let lib_dir = match lib_dir {
            Some(dir) => {
                eprintln!("DEBUG: Zig lib_dir: {}", dir);
                dir
            }
            None => {
                eprintln!("WARNING: Could not find lib_dir in 'zig env' output");
                return None;
            }
        };
        
        // Map Rust triple to zig triple for include path
        let zig_target = match self.map_rust_to_zig_include_target(target_triple) {
            Some(target) => {
                eprintln!("DEBUG: Mapped {} → {}", target_triple, target);
                target
            }
            None => {
                eprintln!("WARNING: No Zig target mapping for Rust triple: {}", target_triple);
                eprintln!("  This platform may not be supported for cross-compilation with Zig headers");
                return None;
            }
        };
        
        // Build include paths based on target type
        let mut paths = vec![
            format!("{}/libc/include/{}", lib_dir, zig_target),
        ];
        
        // Add generic headers for Linux targets
        if target_triple.contains("linux") {
            paths.push(format!("{}/libc/include/any-linux-any", lib_dir));
            
            // Add glibc or musl headers depending on target
            if target_triple.contains("musl") {
                paths.push(format!("{}/libc/include/generic-musl", lib_dir));
            } else {
                paths.push(format!("{}/libc/include/generic-glibc", lib_dir));
            }
        }
        
        eprintln!("DEBUG: Candidate include paths (before filtering):");
        for path in &paths {
            eprintln!("  - {}", path);
        }
        
        // Filter to only include paths that exist
        let original_count = paths.len();
        paths.retain(|p| {
            let exists = std::path::Path::new(p).exists();
            if !exists {
                eprintln!("DEBUG: Path does not exist: {}", p);
            }
            exists
        });
        
        eprintln!("DEBUG: Valid include paths: {}/{}", paths.len(), original_count);
        
        if paths.is_empty() {
            eprintln!("WARNING: No valid Zig include paths found for target: {}", target_triple);
            return None;
        }
        
        eprintln!("✓ Using Zig libc headers for cross-compilation:");
        for path in &paths {
            eprintln!("  {}", path);
        }
        
        Some(paths)
    }

    fn map_rust_to_zig_include_target(&self, rust_triple: &str) -> Option<&'static str> {
        // Map Rust target triples to zig's libc include directory names
        // Based on targets defined in data/toolchains.json
        match rust_triple {
            // Linux ARM targets
            "arm-unknown-linux-gnueabihf" => Some("arm-linux-gnu"),
            "armv7-unknown-linux-gnueabihf" => Some("arm-linux-gnu"),
            
            // Linux AArch64 targets
            "aarch64-unknown-linux-gnu" => Some("aarch64-linux-gnu"),
            "aarch64-unknown-linux-musl" => Some("aarch64-linux-musl"),
            
            // Linux x86 targets
            "i686-unknown-linux-gnu" => Some("x86-linux-gnu"),
            
            // Linux x86_64 targets
            // Note: x86_64-linux-gnu-specific dir doesn't exist, use x86-linux-gnu
            // which has similar enough headers for bindgen
            "x86_64-unknown-linux-gnu" => Some("x86-linux-gnu"),
            "x86_64-unknown-linux-musl" => Some("x86_64-linux-musl"),
            
            // macOS and Windows targets don't use Zig's libc headers
            // They use SDK headers or native system headers
            "aarch64-apple-darwin" => None,
            "x86_64-apple-darwin" => None,
            "x86_64-pc-windows-gnu" => None,
            "aarch64-pc-windows-gnullvm" => None,
            "i686-pc-windows-gnu" => None,
            
            // Unknown target
            _ => None,
        }
    }

    fn get_shim_path(&self, name: &str) -> String {
        let filename = if cfg!(windows) {
            format!("{}.exe", name)
        } else {
            name.to_string()
        };

        self.shim_dir
            .join(filename)
            .to_str()
            .unwrap_or_else(|| {
                eprintln!("Warning: Shim path contains invalid UTF-8, using lossy conversion");
                ""
            })
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shim_generator_new() {
        let shim_dir = PathBuf::from("/tmp/shims");
        let zig_path = PathBuf::from("/usr/bin/zig");
        let generator = ShimGenerator::new(shim_dir.clone(), zig_path.clone());

        assert_eq!(generator.shim_dir, shim_dir);
        assert_eq!(generator.zig_path, zig_path);
    }

    #[test]
    fn test_get_shim_env() {
        let shim_dir = PathBuf::from("/tmp/shims");
        let zig_path = PathBuf::from("/usr/bin/zig");
        let generator = ShimGenerator::new(shim_dir, zig_path);

        let env = generator.get_shim_env("x86_64-unknown-linux-gnu", None);

        assert!(env.contains_key("CC_x86_64_unknown_linux_gnu"));
        assert!(env.contains_key("CRATE_CC_NO_DEFAULTS"));
        assert_eq!(env.get("CRATE_CC_NO_DEFAULTS").unwrap(), "1");
        assert_eq!(env.get("RB_SYS_CROSS_COMPILING").unwrap(), "1");
    }
}
