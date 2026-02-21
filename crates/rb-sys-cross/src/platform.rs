use anyhow::{bail, Result};

/// A supported cross-compilation platform.
#[derive(Debug, Clone)]
pub struct Platform {
    pub ruby_platform: &'static str,
    pub rust_target: &'static str,
    pub autoconf_host: &'static str,
    pub glibc_version: Option<&'static str>,
    pub zig_supported: bool,
    pub aliases: &'static [&'static str],
}

// Generated from data/toolchains.json via build.rs
include!(concat!(env!("OUT_DIR"), "/platforms_generated.rs"));

impl Platform {
    /// Find a platform by its Ruby platform name (or alias).
    pub fn find(name: &str) -> Result<&'static Platform> {
        for p in PLATFORMS {
            if p.ruby_platform == name || p.aliases.contains(&name) {
                return Ok(p);
            }
        }
        bail!(
            "unsupported platform: {name}\n\nSupported platforms:\n{}",
            Self::list_supported()
        )
    }

    /// Return all supported platforms.
    pub fn all() -> &'static [Platform] {
        PLATFORMS
    }

    /// List all supported platforms as a formatted string.
    pub fn list_supported() -> String {
        let mut out = String::new();
        for p in PLATFORMS {
            out.push_str(&format!(
                "  {:<24} → {}\n",
                p.ruby_platform, p.rust_target
            ));
        }
        out
    }

    /// The zig target suffix (e.g. "aarch64-unknown-linux-gnu.2.17")
    pub fn zigbuild_target(&self) -> String {
        match self.glibc_version {
            Some(ver) => format!("{}.{}", self.rust_target, ver),
            None => self.rust_target.to_string(),
        }
    }

    /// The zig cc target (autoconf-style with optional glibc version).
    /// e.g. "aarch64-linux-gnu.2.17" or "aarch64-linux-musl"
    pub fn zig_cc_target(&self) -> String {
        match self.glibc_version {
            Some(ver) => format!("{}.{}", self.autoconf_host, ver),
            None => self.autoconf_host.to_string(),
        }
    }

    /// The file extension for compiled shared libraries on this platform.
    pub fn shared_lib_ext(&self) -> &'static str {
        if self.rust_target.contains("windows") {
            "dll"
        } else if self.rust_target.contains("darwin") {
            "bundle"
        } else {
            "so"
        }
    }

    /// Return zig configure overrides (autoconf variable -> value).
    pub fn zig_configure_overrides() -> &'static [(&'static str, &'static str)] {
        ZIG_CONFIGURE_OVERRIDES
    }

    /// Return zig config.h fixup lines.
    pub fn zig_config_h_fixups() -> &'static [&'static str] {
        ZIG_CONFIG_H_FIXUPS
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ruby_platform)
    }
}
