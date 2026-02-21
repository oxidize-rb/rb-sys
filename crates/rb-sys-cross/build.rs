use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

use serde::Deserialize;

#[derive(Deserialize)]
struct ToolchainsFile {
    zig: ZigGlobal,
    toolchains: Vec<Toolchain>,
}

#[derive(Deserialize)]
struct ZigGlobal {
    #[serde(rename = "configure-overrides")]
    configure_overrides: std::collections::BTreeMap<String, String>,
    #[serde(rename = "config-h-fixups")]
    config_h_fixups: Vec<String>,
}

#[derive(Deserialize)]
struct Toolchain {
    #[serde(rename = "ruby-platform")]
    ruby_platform: String,
    #[serde(rename = "rust-target")]
    rust_target: String,
    #[serde(default)]
    aliases: Vec<String>,
    zig: ZigPlatform,
}

#[derive(Deserialize)]
struct ZigPlatform {
    supported: bool,
    #[serde(rename = "autoconf-host")]
    autoconf_host: Option<String>,
    #[serde(rename = "glibc-version")]
    glibc_version: Option<String>,
}

fn main() {
    let toolchains_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("data")
        .join("toolchains.json");

    println!(
        "cargo:rerun-if-changed={}",
        toolchains_path.canonicalize().unwrap().display()
    );

    let contents = fs::read_to_string(&toolchains_path).unwrap_or_else(|e| {
        panic!(
            "failed to read {}: {e}",
            toolchains_path.display()
        )
    });

    let data: ToolchainsFile = serde_json::from_str(&contents).unwrap_or_else(|e| {
        panic!("failed to parse toolchains.json: {e}")
    });

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("platforms_generated.rs");
    let mut out = fs::File::create(&out_path).unwrap();

    writeln!(out, "// Auto-generated from data/toolchains.json — do not edit").unwrap();
    writeln!(out).unwrap();

    // Emit PLATFORMS
    writeln!(out, "pub static PLATFORMS: &[Platform] = &[").unwrap();
    for tc in &data.toolchains {
        if !tc.zig.supported {
            continue;
        }
        let autoconf_host = tc.zig.autoconf_host.as_deref().unwrap_or("");
        let glibc = match &tc.zig.glibc_version {
            Some(v) => format!("Some({v:?})"),
            None => "None".to_string(),
        };
        let aliases: Vec<String> = tc.aliases.iter().map(|a| format!("{a:?}")).collect();
        writeln!(out, "    Platform {{").unwrap();
        writeln!(out, "        ruby_platform: {:?},", tc.ruby_platform).unwrap();
        writeln!(out, "        rust_target: {:?},", tc.rust_target).unwrap();
        writeln!(out, "        autoconf_host: {:?},", autoconf_host).unwrap();
        writeln!(out, "        glibc_version: {glibc},").unwrap();
        writeln!(out, "        zig_supported: true,").unwrap();
        writeln!(out, "        aliases: &[{}],", aliases.join(", ")).unwrap();
        writeln!(out, "    }},").unwrap();
    }
    writeln!(out, "];").unwrap();
    writeln!(out).unwrap();

    // Emit ZIG_CONFIGURE_OVERRIDES
    writeln!(
        out,
        "pub static ZIG_CONFIGURE_OVERRIDES: &[(&str, &str)] = &["
    )
    .unwrap();
    for (key, val) in &data.zig.configure_overrides {
        writeln!(out, "    ({key:?}, {val:?}),").unwrap();
    }
    writeln!(out, "];").unwrap();
    writeln!(out).unwrap();

    // Emit ZIG_CONFIG_H_FIXUPS
    writeln!(out, "pub static ZIG_CONFIG_H_FIXUPS: &[&str] = &[").unwrap();
    for fixup in &data.zig.config_h_fixups {
        writeln!(out, "    {:?},", fixup).unwrap();
    }
    writeln!(out, "];").unwrap();
}
