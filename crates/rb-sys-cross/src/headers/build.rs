use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use anyhow::{bail, Context, Result};
use regex::Regex;

use crate::platform::Platform;
use crate::util;

use super::cache;

fn re_config_str() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"CONFIG\["([^"]+)"\]\s*=\s*"([^"]*)""#).unwrap())
}

fn re_config_num() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"CONFIG\["([^"]+)"\]\s*=\s*(\d+)"#).unwrap())
}

/// Build Ruby headers from source using zig cc for cross-compilation.
/// Returns the path to the cached header directory.
pub fn build_ruby_headers(platform: &Platform, ruby_version: &str) -> Result<PathBuf> {
    // Check prerequisites
    check_prerequisites()?;

    let ruby_major_minor = ruby_major_minor(ruby_version);
    let zig_cc_target = platform.zig_cc_target();

    let cache_base = cache::cache_dir()?.parent().unwrap().to_path_buf();
    let workdir = cache_base.join("build");
    let wrapper_dir = workdir.join("zig-wrappers");

    eprintln!(
        "=== Cross-compiling Ruby {ruby_version} for {} ===",
        platform.ruby_platform
    );
    eprintln!("  Zig target: {zig_cc_target}");
    eprintln!("  Autoconf host: {}", platform.autoconf_host);
    eprintln!();

    // Create zig wrapper scripts
    create_zig_wrappers(&wrapper_dir, &zig_cc_target)?;

    // Download Ruby source
    let ruby_src = download_ruby_source(&workdir, ruby_version, &ruby_major_minor)?;

    // Configure
    let build_dir = workdir.join(format!("build-{}-{ruby_version}", platform.ruby_platform));
    if build_dir.exists() {
        fs::remove_dir_all(&build_dir)?;
    }
    fs::create_dir_all(&build_dir)?;

    configure_ruby(&ruby_src, &build_dir, platform, &wrapper_dir)?;

    // Patch config.h with fixups
    patch_config_h(&build_dir)?;

    // Build
    build_ruby(&build_dir)?;

    // Collect artifacts
    let dest = cache::header_dir(platform.ruby_platform, &ruby_major_minor)?;
    collect_artifacts(&ruby_src, &build_dir, &dest, platform)?;

    // Convert rbconfig.rb → rbconfig.json
    convert_rbconfig(&build_dir, &dest)?;

    eprintln!();
    eprintln!("=== Done ===");
    eprintln!("Artifacts cached in: {}", dest.display());

    Ok(dest)
}

fn check_prerequisites() -> Result<()> {
    if which::which("zig").is_err() {
        bail!("zig not found on PATH. Install zig first: https://ziglang.org/download/");
    }
    if which::which("ruby").is_err() {
        bail!("ruby not found on PATH (needed as baseruby)");
    }

    let zig_out = Command::new("zig").arg("version").output()?;
    eprintln!(
        "Using zig: {}",
        String::from_utf8_lossy(&zig_out.stdout).trim()
    );

    let ruby_out = Command::new("ruby").arg("--version").output()?;
    eprintln!(
        "Using baseruby: {}",
        String::from_utf8_lossy(&ruby_out.stdout).trim()
    );
    eprintln!();

    Ok(())
}

fn ruby_major_minor(version: &str) -> String {
    let parts: Vec<&str> = version.splitn(3, '.').collect();
    if parts.len() >= 2 {
        format!("{}.{}", parts[0], parts[1])
    } else {
        version.to_string()
    }
}

fn create_zig_wrappers(wrapper_dir: &Path, zig_target: &str) -> Result<()> {
    fs::create_dir_all(wrapper_dir)?;

    let wrappers = [
        (
            "zigcc",
            format!("#!/bin/sh\nexec zig cc --target={zig_target} \"$@\"\n"),
        ),
        (
            "zigcxx",
            format!("#!/bin/sh\nexec zig c++ --target={zig_target} \"$@\"\n"),
        ),
        ("zigar", "#!/bin/sh\nexec zig ar \"$@\"\n".to_string()),
        (
            "zigranlib",
            "#!/bin/sh\nexec zig ranlib \"$@\"\n".to_string(),
        ),
    ];

    for (name, content) in &wrappers {
        let path = wrapper_dir.join(name);
        fs::write(&path, content)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
        }
    }

    Ok(())
}

fn download_ruby_source(workdir: &Path, version: &str, major_minor: &str) -> Result<PathBuf> {
    fs::create_dir_all(workdir)?;
    let ruby_src = workdir.join(format!("ruby-{version}"));

    if ruby_src.exists() {
        return Ok(ruby_src);
    }

    let tarball = workdir.join(format!("ruby-{version}.tar.gz"));
    if !tarball.exists() {
        let url =
            format!("https://cache.ruby-lang.org/pub/ruby/{major_minor}/ruby-{version}.tar.gz");
        eprintln!("Downloading Ruby {version}...");

        let response = reqwest::blocking::get(&url).with_context(|| format!("fetching {url}"))?;

        if !response.status().is_success() {
            bail!(
                "failed to download Ruby source from {url}: HTTP {}",
                response.status()
            );
        }

        let bytes = response.bytes()?;
        fs::write(&tarball, &bytes)?;
    }

    eprintln!("Extracting...");
    let tar_gz = fs::File::open(&tarball)?;
    let gz = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(gz);
    archive
        .unpack(workdir)
        .with_context(|| "extracting Ruby source tarball")?;

    Ok(ruby_src)
}

fn configure_ruby(
    ruby_src: &Path,
    build_dir: &Path,
    platform: &Platform,
    wrapper_dir: &Path,
) -> Result<()> {
    eprintln!("Configuring Ruby for {}...", platform.ruby_platform);

    // Determine build triple
    let config_guess = ruby_src.join("tool").join("config.guess");
    let build_triple = if config_guess.exists() {
        let out = Command::new(&config_guess)
            .output()
            .context("running config.guess")?;
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    } else {
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;
        format!("{arch}-{os}")
    };

    let configure = ruby_src.join("configure");
    let mut cmd = Command::new(&configure);
    cmd.current_dir(build_dir)
        .arg(format!("--host={}", platform.autoconf_host))
        .arg(format!("--build={build_triple}"))
        .arg("--prefix=/usr/local")
        .arg("--disable-install-doc")
        .arg("--disable-install-rdoc")
        .arg("--enable-install-static-library")
        .arg("--disable-shared")
        .arg("--with-static-linked-ext")
        .arg("--without-gmp")
        .arg("--without-gdbm")
        .arg("--without-dbm")
        .arg("--without-readline")
        .arg("--without-openssl")
        .arg("--without-fiddle")
        .env("CC", wrapper_dir.join("zigcc"))
        .env("CXX", wrapper_dir.join("zigcxx"))
        .env("AR", wrapper_dir.join("zigar"))
        .env("RANLIB", wrapper_dir.join("zigranlib"))
        .env("LD", wrapper_dir.join("zigcc"))
        .env("CFLAGS", "-O2 -fPIC")
        .env("LDFLAGS", "");

    // Add configure overrides from generated data
    for (key, val) in Platform::zig_configure_overrides() {
        cmd.arg(format!("{key}={val}"));
    }

    let status = cmd
        .status()
        .with_context(|| format!("running {}", configure.display()))?;

    if !status.success() {
        bail!("configure failed with exit code: {status}");
    }

    Ok(())
}

fn patch_config_h(build_dir: &Path) -> Result<()> {
    // Find config.h in build dir
    let config_h = find_config_h(build_dir)?;

    let Some(config_h) = config_h else {
        eprintln!("Warning: config.h not found, skipping fixup");
        return Ok(());
    };

    eprintln!(
        "Patching {} with cross-compilation fixups...",
        config_h.display()
    );

    let content = fs::read_to_string(&config_h)?;

    // Build fixup block
    let mut fixup_lines = String::new();
    fixup_lines.push_str("\n/* rb-sys-cross: fixups for cross-compilation with zig cc */\n");
    for fixup in Platform::zig_config_h_fixups() {
        // Extract the macro name for the #ifndef guard
        let macro_name = extract_define_name(fixup);
        if let Some(name) = macro_name {
            fixup_lines.push_str(&format!("#ifndef {name}\n{fixup}\n#endif\n"));
        } else {
            fixup_lines.push_str(&format!("{fixup}\n"));
        }
    }

    // Insert before the final #endif
    let patched = if let Some(pos) = content.rfind("#endif") {
        let (before, after) = content.split_at(pos);
        format!("{before}{fixup_lines}{after}")
    } else {
        format!("{content}{fixup_lines}")
    };

    fs::write(&config_h, patched)?;

    Ok(())
}

fn extract_define_name(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("#define ") {
        // Get the first token (macro name, possibly with parens)
        let name_end = rest.find([' ', '(']).unwrap_or(rest.len());
        Some(&rest[..name_end])
    } else {
        None
    }
}

fn find_config_h(build_dir: &Path) -> Result<Option<PathBuf>> {
    let ext_include = build_dir.join(".ext").join("include");
    if ext_include.exists() {
        for entry in fs::read_dir(&ext_include)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let candidate = entry.path().join("ruby").join("config.h");
                if candidate.exists() {
                    return Ok(Some(candidate));
                }
            }
        }
    }
    Ok(None)
}

fn build_ruby(build_dir: &Path) -> Result<()> {
    eprintln!("Building...");

    let jobs = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let status = Command::new("make")
        .current_dir(build_dir)
        .arg(format!("-j{jobs}"))
        .arg("libruby-static.a")
        .arg("rbconfig.rb")
        .status()
        .context("running make")?;

    if !status.success() {
        bail!("make failed with exit code: {status}");
    }

    Ok(())
}

fn collect_artifacts(
    ruby_src: &Path,
    build_dir: &Path,
    dest: &Path,
    platform: &Platform,
) -> Result<()> {
    if dest.exists() {
        fs::remove_dir_all(dest)?;
    }

    let include_dest = dest.join("include");
    let lib_dest = dest.join("lib");
    fs::create_dir_all(&include_dest)?;
    fs::create_dir_all(&lib_dest)?;

    // Copy common headers from source
    util::copy_dir_recursive(&ruby_src.join("include"), &include_dest)?;

    // Copy architecture-specific headers from build dir
    let ext_include = build_dir.join(".ext").join("include");
    if ext_include.exists() {
        for entry in fs::read_dir(&ext_include)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                util::copy_dir_recursive(&entry.path(), &include_dest)?;
            }
        }
    } else {
        // Fallback: config.h might be in the build root
        let fallback = build_dir.join("include").join("ruby").join("config.h");
        if fallback.exists() {
            let arch_dir = include_dest.join(platform.ruby_platform).join("ruby");
            fs::create_dir_all(&arch_dir)?;
            fs::copy(&fallback, arch_dir.join("config.h"))?;
        }
    }

    // Static library
    let static_lib = build_dir.join("libruby-static.a");
    if static_lib.exists() {
        fs::copy(&static_lib, lib_dest.join("libruby-static.a"))?;
        eprintln!("Copied libruby-static.a");
    }

    // rbconfig.rb
    let rbconfig = build_dir.join("rbconfig.rb");
    if rbconfig.exists() {
        fs::copy(&rbconfig, dest.join("rbconfig.rb"))?;
        eprintln!("Copied rbconfig.rb");
    }

    Ok(())
}

fn convert_rbconfig(build_dir: &Path, dest: &Path) -> Result<()> {
    let rbconfig_rb = build_dir.join("rbconfig.rb");
    if !rbconfig_rb.exists() {
        eprintln!("Warning: rbconfig.rb not found, skipping JSON conversion");
        return Ok(());
    }

    let content = fs::read_to_string(&rbconfig_rb)?;
    let mut config = serde_json::Map::new();

    // Match CONFIG["key"] = "value"
    for cap in re_config_str().captures_iter(&content) {
        config.insert(
            cap[1].to_string(),
            serde_json::Value::String(cap[2].to_string()),
        );
    }

    // Match CONFIG["key"] = 123 (numeric)
    for cap in re_config_num().captures_iter(&content) {
        config.insert(
            cap[1].to_string(),
            serde_json::Value::String(cap[2].to_string()),
        );
    }

    let json = serde_json::to_string_pretty(&config)?;
    let rbconfig_json = dest.join("rbconfig.json");
    let mut f = fs::File::create(&rbconfig_json)?;
    f.write_all(json.as_bytes())?;
    eprintln!("Generated rbconfig.json");

    Ok(())
}
