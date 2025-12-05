use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=../../data/toolchains.json");

    let toolchains_path = Path::new("../../data/toolchains.json");
    let toolchains_content =
        fs::read_to_string(toolchains_path).expect("Failed to read toolchains.json");

    let toolchains: serde_json::Value =
        serde_json::from_str(&toolchains_content).expect("Failed to parse toolchains.json");

    let mut generated_code = String::new();
    generated_code.push_str("// Auto-generated from data/toolchains.json\n");
    generated_code.push_str("// DO NOT EDIT MANUALLY\n\n");

    // Collect toolchain data
    let mut variants = Vec::new();
    if let Some(toolchains_array) = toolchains["toolchains"].as_array() {
        for toolchain in toolchains_array {
            let ruby_platform = toolchain["ruby-platform"].as_str().unwrap();
            let rust_target = toolchain["rust-target"].as_str().unwrap();
            let supported = toolchain["supported"].as_bool().unwrap_or(false);

            // Parse sysroot-paths as array of strings
            let sysroot_paths: Vec<String> = toolchain["sysroot-paths"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            // Convert ruby-platform to PascalCase variant name
            let variant_name = ruby_platform
                .split(|c| c == '-' || c == '_')
                .map(|s| {
                    let mut chars = s.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().chain(chars).collect(),
                    }
                })
                .collect::<String>();

            variants.push((
                variant_name,
                ruby_platform.to_string(),
                rust_target.to_string(),
                sysroot_paths,
                supported,
            ));
        }
    }

    // Generate enum
    generated_code.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n");
    generated_code.push_str("pub enum Toolchain {\n");
    for (variant_name, _, _, _, _) in &variants {
        generated_code.push_str(&format!("    {},\n", variant_name));
    }
    generated_code.push_str("}\n\n");

    // Generate impl block
    generated_code.push_str("impl Toolchain {\n");

    // ruby_platform method
    generated_code.push_str("    pub const fn ruby_platform(&self) -> &'static str {\n");
    generated_code.push_str("        match self {\n");
    for (variant_name, ruby_platform, _, _, _) in &variants {
        generated_code.push_str(&format!(
            "            Toolchain::{} => \"{}\",\n",
            variant_name, ruby_platform
        ));
    }
    generated_code.push_str("        }\n");
    generated_code.push_str("    }\n\n");

    // rust_target method
    generated_code.push_str("    pub const fn rust_target(&self) -> &'static str {\n");
    generated_code.push_str("        match self {\n");
    for (variant_name, _, rust_target, _, _) in &variants {
        generated_code.push_str(&format!(
            "            Toolchain::{} => \"{}\",\n",
            variant_name, rust_target
        ));
    }
    generated_code.push_str("        }\n");
    generated_code.push_str("    }\n\n");

    // sysroot_paths method
    generated_code.push_str("    pub const fn sysroot_paths(&self) -> &'static [&'static str] {\n");
    generated_code.push_str("        match self {\n");
    for (variant_name, _, _, sysroot_paths, _) in &variants {
        if sysroot_paths.is_empty() {
            generated_code.push_str(&format!(
                "            Toolchain::{} => &[],\n",
                variant_name
            ));
        } else {
            let paths_str = sysroot_paths
                .iter()
                .map(|p| format!("\"{}\"", p))
                .collect::<Vec<_>>()
                .join(", ");
            generated_code.push_str(&format!(
                "            Toolchain::{} => &[{}],\n",
                variant_name, paths_str
            ));
        }
    }
    generated_code.push_str("        }\n");
    generated_code.push_str("    }\n\n");

    // rake_compiler_image method
    generated_code.push_str("    pub fn rake_compiler_image(&self) -> String {\n");
    generated_code.push_str("        format!(\"ghcr.io/rake-compiler/rake-compiler-dock-image:1.10.0-mri-{}\", self.ruby_platform())\n");
    generated_code.push_str("    }\n\n");

    // supported method
    generated_code.push_str("    pub const fn supported(&self) -> bool {\n");
    generated_code.push_str("        match self {\n");
    for (variant_name, _, _, _, supported) in &variants {
        generated_code.push_str(&format!(
            "            Toolchain::{} => {},\n",
            variant_name, supported
        ));
    }
    generated_code.push_str("        }\n");
    generated_code.push_str("    }\n\n");

    // from_ruby_platform method
    generated_code
        .push_str("    pub fn from_ruby_platform(platform: &str) -> Option<Toolchain> {\n");
    generated_code.push_str("        match platform {\n");
    for (variant_name, ruby_platform, _, _, _) in &variants {
        generated_code.push_str(&format!(
            "            \"{}\" => Some(Toolchain::{}),\n",
            ruby_platform, variant_name
        ));
    }
    generated_code.push_str("            _ => None,\n");
    generated_code.push_str("        }\n");
    generated_code.push_str("    }\n\n");

    // from_rust_target method
    generated_code.push_str("    pub fn from_rust_target(target: &str) -> Option<Toolchain> {\n");
    generated_code.push_str("        match target {\n");
    for (variant_name, _, rust_target, _, _) in &variants {
        generated_code.push_str(&format!(
            "            \"{}\" => Some(Toolchain::{}),\n",
            rust_target, variant_name
        ));
    }
    generated_code.push_str("            _ => None,\n");
    generated_code.push_str("        }\n");
    generated_code.push_str("    }\n\n");

    // all_supported method
    generated_code.push_str("    pub fn all_supported() -> impl Iterator<Item = Toolchain> {\n");
    generated_code.push_str("        [\n");
    for (variant_name, _, _, _, _) in &variants {
        generated_code.push_str(&format!("            Toolchain::{},\n", variant_name));
    }
    generated_code.push_str("        ].into_iter().filter(|t| t.supported())\n");
    generated_code.push_str("    }\n");

    generated_code.push_str("}\n");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("toolchain_mappings.rs");
    fs::write(&dest_path, generated_code).expect("Failed to write generated code");
}
