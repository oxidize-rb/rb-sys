use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use oci_distribution::client::{ClientConfig, ClientProtocol};
use oci_distribution::secrets::RegistryAuth;
use oci_distribution::{Client, Reference};
use std::fmt::Write;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tar::Archive;

use crate::generated_mappings::Toolchain;

const RUBY_PATH_PREFIXES: &[&str] = &[
    "usr/local/rake-compiler/rubies/",
    "usr/local/rake-compiler/ruby/",
];

/// Configuration for what to extract from the image
#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    /// The ruby platform string (e.g., "x86_64-linux-gnu")
    pub ruby_platform: String,
    /// The sysroot path prefixes to extract (e.g., ["usr/include", "usr/lib/x86_64-linux-gnu"])
    /// Note: stored without leading slash for tar path matching
    pub sysroot_prefixes: Vec<String>,
}

impl ExtractionConfig {
    pub fn new(toolchain: Toolchain) -> Self {
        let ruby_platform = toolchain.ruby_platform().to_string();
        let sysroot_prefixes = toolchain
            .sysroot_paths()
            .iter()
            .map(|p| p.trim_start_matches('/').to_string())
            .collect();

        Self {
            ruby_platform,
            sysroot_prefixes,
        }
    }
}

/// Categorization of extracted files
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileCategory {
    /// Ruby headers and rbconfig.rb
    Ruby,
    /// System sysroot files (headers and libraries)
    Sysroot,
}

/// Extract Ruby headers, libraries, and sysroot from a rake-compiler-dock image
///
/// # Arguments
/// * `toolchain` - The target toolchain to extract for
///
/// # Example
/// ```no_run
/// use rb_sys_cli::extractor::extract_for_toolchain;
/// use rb_sys_cli::generated_mappings::Toolchain;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     extract_for_toolchain(Toolchain::X8664Linux).await?;
///     Ok(())
/// }
/// ```
pub async fn extract_for_toolchain(toolchain: Toolchain) -> Result<()> {
    let config = ExtractionConfig::new(toolchain);
    let image_ref = toolchain.rake_compiler_image();

    println!(
        "🐳 Extracting for target: {} ({})",
        toolchain.ruby_platform(),
        toolchain.rust_target()
    );
    println!("   Image: {}", image_ref);

    if !config.sysroot_prefixes.is_empty() {
        println!("   Sysroot paths: {:?}", config.sysroot_prefixes);
    }

    // Parse the image reference
    let reference: Reference = image_ref
        .parse()
        .context("Failed to parse image reference")?;

    println!("   Registry: {}", reference.registry());
    println!("   Repository: {}", reference.repository());
    println!("   Tag: {}", reference.tag().unwrap_or("latest"));

    // Determine target platform from image tag
    let platform = if reference
        .tag()
        .map(|t| t.contains("x86_64") || t.contains("x64-mingw"))
        .unwrap_or(false)
    {
        Some("linux/amd64")
    } else if reference
        .tag()
        .map(|t| t.contains("aarch64") || t.contains("arm64"))
        .unwrap_or(false)
    {
        Some("linux/arm64")
    } else if reference.tag().map(|t| t.contains("arm-")).unwrap_or(false) {
        Some("linux/arm/v7")
    } else {
        None
    };

    if let Some(plat) = platform {
        println!("   Platform: {}", plat);
    }

    // Create OCI client with platform
    let client = create_oci_client(platform)?;

    // Pull the image manifest
    println!("📦 Fetching image manifest...");
    let auth = get_registry_auth();

    let image_data = client
        .pull(
            &reference,
            &auth,
            vec![
                oci_distribution::manifest::IMAGE_LAYER_MEDIA_TYPE,
                oci_distribution::manifest::IMAGE_LAYER_GZIP_MEDIA_TYPE,
                "application/vnd.docker.image.rootfs.diff.tar.gzip",
            ],
        )
        .await
        .context("Failed to pull image from registry")?;

    println!(
        "\n🔍 Scanning {} layer(s) for files...",
        image_data.layers.len()
    );

    // Create progress bar for layers
    let pb = ProgressBar::new(image_data.layers.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} layers ({msg})"
        )?
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("#>-"),
    );
    pb.set_message("Processing layers...");

    let mut total_ruby_files = 0;
    let mut total_sysroot_files = 0;
    let _start_time = Instant::now();

    for (idx, layer) in image_data.layers.iter().enumerate() {
        let size = layer.data.len() as u64;
        let size_mb = size / 1_048_576;

        pb.set_message(format!("Layer {}: {}MB", idx + 1, size_mb));

        // Clone the data to move into blocking thread
        let layer_data = layer.data.clone();
        let config_clone = config.clone();

        // Offload CPU/IO intensive work to a blocking thread
        let layer_start = Instant::now();
        let (ruby_count, sysroot_count) =
            tokio::task::spawn_blocking(move || process_layer_blob(&layer_data, &config_clone))
                .await
                .context("Failed to join blocking task")??;
        let layer_duration = layer_start.elapsed();

        total_ruby_files += ruby_count;
        total_sysroot_files += sysroot_count;

        pb.inc(1);

        let extracted_count = ruby_count + sysroot_count;
        if extracted_count > 0 {
            let throughput = size_mb as f64 / layer_duration.as_secs_f64().max(0.001);
            pb.println(format!(
                "   ✅ Layer {}/{}: {} files extracted ({} Ruby, {} sysroot) - {:.1}s, {:.1}MB/s",
                idx + 1,
                image_data.layers.len(),
                extracted_count,
                ruby_count,
                sysroot_count,
                layer_duration.as_secs_f64(),
                throughput
            ));
        } else {
            pb.println(format!(
                "   ⏭️  Layer {}/{}: No relevant files found - {:.1}s",
                idx + 1,
                image_data.layers.len(),
                layer_duration.as_secs_f64()
            ));
        }
    }

    pb.finish_with_message("All layers processed");

    let total = total_ruby_files + total_sysroot_files;
    if total > 0 {
        println!("\n✅ Successfully extracted {} file(s) to cache", total);
        println!("   Ruby files: {}", total_ruby_files);
        println!("   Sysroot files: {}", total_sysroot_files);
        println!("   Cache location: {}", get_cache_dir()?.display());

        // Serialize rbconfig.rb files to JSON
        println!("\n📝 Serializing rbconfig files...");
        let serialized_count = serialize_rbconfigs(&config.ruby_platform)?;
        if serialized_count > 0 {
            println!("   ✓ Serialized {} rbconfig file(s)", serialized_count);
        }
    } else {
        println!("\n⚠️  No files found in image layers");
        println!("   This might not be a rake-compiler-dock image");
    }

    Ok(())
}

/// Legacy function for backwards compatibility - extracts from a full image reference
pub async fn extract_headers(image_ref: &str) -> Result<()> {
    // Try to determine toolchain from image tag
    let reference: Reference = image_ref
        .parse()
        .context("Failed to parse image reference")?;

    let tag = reference.tag().unwrap_or("latest");

    // Try to extract ruby platform from tag (e.g., "1.10.0-mri-x86_64-linux" -> "x86_64-linux")
    let ruby_platform = tag
        .strip_prefix("1.10.0-mri-")
        .or_else(|| tag.strip_prefix("1.9.1-mri-"))
        .or_else(|| tag.strip_prefix("1.8.0-mri-"))
        .or_else(|| tag.strip_prefix("1.7.0-mri-"))
        .or_else(|| tag.strip_prefix("1.6.0-mri-"))
        .or_else(|| tag.strip_prefix("1.5.0-mri-"))
        .or_else(|| tag.strip_prefix("1.4.0-mri-"))
        .or_else(|| tag.strip_prefix("1.3.0-mri-"));

    if let Some(platform) = ruby_platform {
        if let Some(toolchain) = Toolchain::from_ruby_platform(platform) {
            return extract_for_toolchain(toolchain).await;
        }
    }

    // Fall back to old behavior without sysroot extraction
    println!("⚠️  Could not determine toolchain from image tag, extracting Ruby files only");
    extract_headers_legacy(image_ref).await
}

/// Legacy extraction without sysroot support
async fn extract_headers_legacy(image_ref: &str) -> Result<()> {
    println!("🐳 Extracting Ruby headers from image: {}", image_ref);

    let reference: Reference = image_ref
        .parse()
        .context("Failed to parse image reference")?;

    println!("   Registry: {}", reference.registry());
    println!("   Repository: {}", reference.repository());
    println!("   Tag: {}", reference.tag().unwrap_or("latest"));

    let platform = if reference
        .tag()
        .map(|t| t.contains("x86_64"))
        .unwrap_or(false)
    {
        Some("linux/amd64")
    } else if reference
        .tag()
        .map(|t| t.contains("aarch64"))
        .unwrap_or(false)
    {
        Some("linux/arm64")
    } else {
        None
    };

    if let Some(plat) = platform {
        println!("   Platform: {}", plat);
    }

    let client = create_oci_client(platform)?;

    println!("📦 Fetching image manifest...");
    let auth = get_registry_auth();

    let image_data = client
        .pull(
            &reference,
            &auth,
            vec![
                oci_distribution::manifest::IMAGE_LAYER_MEDIA_TYPE,
                oci_distribution::manifest::IMAGE_LAYER_GZIP_MEDIA_TYPE,
                "application/vnd.docker.image.rootfs.diff.tar.gzip",
            ],
        )
        .await
        .context("Failed to pull image from registry")?;

    println!(
        "\n🔍 Scanning {} layer(s) for Ruby files...",
        image_data.layers.len()
    );

    let mut total_files_extracted = 0;

    for (idx, layer) in image_data.layers.iter().enumerate() {
        let size = layer.data.len() as u64;

        println!(
            "   📥 Layer {}/{}: Processing ({}MB)",
            idx + 1,
            image_data.layers.len(),
            size / 1_048_576
        );

        let layer_data = layer.data.clone();

        let files_extracted =
            tokio::task::spawn_blocking(move || process_layer_blob_legacy(&layer_data))
                .await
                .context("Failed to join blocking task")??;

        total_files_extracted += files_extracted;

        if files_extracted > 0 {
            println!("      ✓ Extracted {} file(s)", files_extracted);
        }
    }

    if total_files_extracted > 0 {
        println!(
            "\n✅ Successfully extracted {} Ruby file(s) to cache",
            total_files_extracted
        );
        println!("   Cache location: {}", get_cache_dir()?.display());
    } else {
        println!("\n⚠️  No Ruby files found in image layers");
        println!("   This might not be a rake-compiler-dock image");
    }

    Ok(())
}

/// Create an OCI distribution client
fn create_oci_client(platform: Option<&str>) -> Result<Client> {
    let mut config = ClientConfig {
        protocol: ClientProtocol::Https,
        ..Default::default()
    };

    if let Some(plat) = platform {
        let plat_string = plat.to_string();
        config.platform_resolver = Some(Box::new(move |platforms| {
            let parts: Vec<&str> = plat_string.split('/').collect();
            if parts.len() >= 2 {
                let os = parts[0];
                let arch = parts[1];

                for entry in platforms {
                    if let Some(platform) = &entry.platform {
                        if platform.os == os && platform.architecture == arch {
                            return Some(entry.digest.clone());
                        }
                    }
                }
            }
            platforms.first().map(|e| e.digest.clone())
        }));
    }

    Ok(Client::new(config))
}

/// Get registry authentication from environment
fn get_registry_auth() -> RegistryAuth {
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        println!("   Using GitHub token for authentication");
        // For ghcr.io, use the token as password with any username
        RegistryAuth::Basic("token".to_string(), token)
    } else {
        RegistryAuth::Anonymous
    }
}

/// Categorize a file path based on the extraction config
fn categorize_path(path_str: &str, config: &ExtractionConfig) -> Option<FileCategory> {
    // Skip docker whiteout files
    if path_str.contains("/.wh.") {
        return None;
    }

    // Skip shared libraries everywhere (.so, .so.1, .so.1.2.3)
    if path_str.contains(".so") {
        return None;
    }

    // Check if it's a Ruby file
    for prefix in RUBY_PATH_PREFIXES {
        if path_str.starts_with(prefix) {
            // Headers - always extract
            if path_str.contains("/include/") {
                return Some(FileCategory::Ruby);
            }
            // rbconfig.rb - extract (will serialize later)
            if path_str.ends_with("rbconfig.rb") {
                return Some(FileCategory::Ruby);
            }
            // Static libraries only (.a files)
            if path_str.ends_with(".a") {
                return Some(FileCategory::Ruby);
            }
            // Skip everything else (bin/, enc/*.so, etc.)
            return None;
        }
    }

    // Check if it's a sysroot file
    for sysroot_prefix in &config.sysroot_prefixes {
        if path_str.starts_with(sysroot_prefix) {
            // Keep all headers
            if path_str.ends_with(".h") || path_str.ends_with(".def") {
                return Some(FileCategory::Sysroot);
            }
            // Static libraries
            if path_str.ends_with(".a") {
                return Some(FileCategory::Sysroot);
            }
            // Object files (crt*.o, etc.)
            if path_str.ends_with(".o") {
                return Some(FileCategory::Sysroot);
            }
            // Skip everything else
            return None;
        }
    }

    None
}

/// Process a single layer blob and extract matching files
fn process_layer_blob(blob: &[u8], config: &ExtractionConfig) -> Result<(usize, usize)> {
    let decoder = GzDecoder::new(blob);
    let mut archive = Archive::new(decoder);

    let mut ruby_files = 0;
    let mut sysroot_files = 0;
    let mut total_entries = 0;
    let mut skipped_entries = 0;

    let start_time = Instant::now();

    for entry_result in archive.entries().context("Failed to read tar entries")? {
        let mut entry = entry_result.context("Failed to read tar entry")?;
        total_entries += 1;

        let path = entry.path().context("Failed to get entry path")?;
        let path_str = path.to_string_lossy().to_string(); // Convert to owned String
        let path_owned = path.to_path_buf();

        let category = match categorize_path(&path_str, config) {
            Some(cat) => cat,
            None => {
                skipped_entries += 1;
                continue;
            }
        };

        match category {
            FileCategory::Ruby => {
                extract_ruby_entry(&mut entry, &path_owned, &config.ruby_platform)
                    .with_context(|| format!("Failed to extract Ruby file: {}", path_str))?;
                ruby_files += 1;
            }
            FileCategory::Sysroot => {
                extract_sysroot_entry(&mut entry, &path_owned, &config.ruby_platform)
                    .with_context(|| format!("Failed to extract sysroot file: {}", path_str))?;
                sysroot_files += 1;
            }
        }
    }

    let duration = start_time.elapsed();
    let extracted = ruby_files + sysroot_files;

    if extracted > 0 {
        eprintln!(
            "   📊 Layer stats: {} extracted, {} skipped, {} total entries ({:.1}s)",
            extracted,
            skipped_entries,
            total_entries,
            duration.as_secs_f64()
        );
    }

    Ok((ruby_files, sysroot_files))
}

/// Legacy layer processing (Ruby files only)
fn process_layer_blob_legacy(blob: &[u8]) -> Result<usize> {
    let decoder = GzDecoder::new(blob);
    let mut archive = Archive::new(decoder);

    let mut files_extracted = 0;

    for entry_result in archive.entries().context("Failed to read tar entries")? {
        let mut entry = entry_result.context("Failed to read tar entry")?;

        let path = entry.path().context("Failed to get entry path")?;
        let path_str = path.to_string_lossy();

        // Check if it's a Ruby path
        let mut is_ruby_file = false;
        for prefix in RUBY_PATH_PREFIXES {
            if path_str.starts_with(prefix) {
                if path_str.contains("/include/") || path_str.ends_with("rbconfig.rb") {
                    is_ruby_file = true;
                    break;
                }
                if path_str.contains("/lib/") {
                    let filename = Path::new(&*path_str)
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or("");
                    if is_library_file(filename) {
                        is_ruby_file = true;
                        break;
                    }
                }
            }
        }

        if !is_ruby_file {
            continue;
        }

        let path_owned = path.to_path_buf();

        // Extract to legacy location (without platform prefix)
        extract_entry_legacy(&mut entry, &path_owned)?;
        files_extracted += 1;
    }

    Ok(files_extracted)
}

/// Check if a file is a library file we want to extract
fn is_library_file(filename: &str) -> bool {
    filename.ends_with(".a")
        || filename.ends_with(".so")
        || filename.ends_with(".dylib")
        || filename.ends_with(".dll")
        || filename.ends_with(".lib")
        || filename.contains(".so.")
}

/// Strip the Ruby path prefix from a path
fn strip_ruby_path_prefix(path_str: &str) -> Option<&str> {
    for prefix in RUBY_PATH_PREFIXES {
        if let Some(stripped) = path_str.strip_prefix(prefix) {
            return Some(stripped);
        }
    }
    None
}

/// Extract a Ruby file to the cache directory
/// Cache structure: ~/.cache/rb-sys/rubies/{ruby_platform}/{ruby_version}/...
fn extract_ruby_entry<R: Read>(
    entry: &mut tar::Entry<R>,
    path: &Path,
    ruby_platform: &str,
) -> Result<()> {
    let path_str = path.to_string_lossy();

    // Strip the prefix: usr/local/rake-compiler/rubies/
    let relative_path =
        strip_ruby_path_prefix(&path_str).context("Path doesn't start with expected prefix")?;

    // relative_path is like: "ruby-3.4.5-x86_64-linux-gnu/include/ruby-3.4.0/ruby.h"
    // We want: {ruby_platform}/{relative_path}
    let cache_dir = get_cache_dir()?;
    let dest_path = cache_dir
        .join("rubies")
        .join(ruby_platform)
        .join(relative_path);

    write_entry_to_path(entry, &dest_path)
}

/// Extract a sysroot file to the cache directory
/// Cache structure: ~/.cache/rb-sys/rubies/{ruby_platform}/sysroot/{original_path}
fn extract_sysroot_entry<R: Read>(
    entry: &mut tar::Entry<R>,
    path: &Path,
    ruby_platform: &str,
) -> Result<()> {
    let path_str = path.to_string_lossy();

    let cache_dir = get_cache_dir()?;
    // Store under: rubies/{ruby_platform}/sysroot/{original_path}
    let dest_path = cache_dir
        .join("rubies")
        .join(ruby_platform)
        .join("sysroot")
        .join(&*path_str);

    write_entry_to_path(entry, &dest_path)
}

/// Legacy entry extraction (for backwards compatibility)
fn extract_entry_legacy<R: Read>(entry: &mut tar::Entry<R>, path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();

    let relative_path =
        strip_ruby_path_prefix(&path_str).context("Path doesn't start with expected prefix")?;

    let cache_dir = get_cache_dir()?;
    let dest_path = cache_dir.join("rubies").join(relative_path);

    write_entry_to_path(entry, &dest_path)
}

/// Write a tar entry to a destination path
fn write_entry_to_path<R: Read>(entry: &mut tar::Entry<R>, dest_path: &Path) -> Result<()> {
    // Skip if it's a directory entry
    if entry.header().entry_type().is_dir() {
        fs::create_dir_all(dest_path)
            .with_context(|| format!("Failed to create directory: {}", dest_path.display()))?;
        return Ok(());
    }

    // Create parent directories
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
    }

    // Extract the file
    let mut dest_file = fs::File::create(dest_path)
        .with_context(|| format!("Failed to create file: {}", dest_path.display()))?;

    std::io::copy(entry, &mut dest_file)
        .with_context(|| format!("Failed to write file: {}", dest_path.display()))?;

    // Preserve permissions on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(mode) = entry.header().mode() {
            let perms = fs::Permissions::from_mode(mode);
            let _ = fs::set_permissions(dest_path, perms);
        }
    }

    Ok(())
}

/// Get the cache directory for extracted Ruby files
pub fn get_cache_dir() -> Result<PathBuf> {
    let cache_dir = if let Ok(override_dir) = std::env::var("RB_SYS_CACHE_DIR") {
        PathBuf::from(override_dir)
    } else if let Some(cache_home) = std::env::var_os("XDG_CACHE_HOME") {
        PathBuf::from(cache_home).join("rb-sys")
    } else if let Some(home_dir) = dirs::home_dir() {
        home_dir.join(".cache").join("rb-sys")
    } else {
        anyhow::bail!("Could not determine cache directory (no HOME or XDG_CACHE_HOME)")
    };

    fs::create_dir_all(&cache_dir)
        .with_context(|| format!("Failed to create cache directory: {}", cache_dir.display()))?;

    Ok(cache_dir)
}

/// Get the sysroot path for a toolchain from the cache
#[allow(dead_code)]
pub fn get_cached_sysroot(toolchain: Toolchain) -> Option<PathBuf> {
    let cache_dir = get_cache_dir().ok()?;
    let ruby_platform = toolchain.ruby_platform();
    let sysroot_paths = toolchain.sysroot_paths();

    if sysroot_paths.is_empty() {
        return None;
    }

    // The sysroot is stored at: rubies/{ruby_platform}/sysroot/
    let sysroot_dir = cache_dir.join("rubies").join(ruby_platform).join("sysroot");

    if sysroot_dir.exists() {
        Some(sysroot_dir)
    } else {
        None
    }
}

/// List all extracted Ruby versions in the cache
pub fn list_cached_rubies() -> Result<Vec<String>> {
    let cache_dir = get_cache_dir()?;
    let rubies_dir = cache_dir.join("rubies");

    if !rubies_dir.exists() {
        return Ok(Vec::new());
    }

    let mut rubies = Vec::new();

    for entry in fs::read_dir(&rubies_dir)
        .with_context(|| format!("Failed to read directory: {}", rubies_dir.display()))?
    {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                rubies.push(name.to_string());
            }
        }
    }

    rubies.sort();
    Ok(rubies)
}

/// Clear the entire cache
pub fn clear_cache() -> Result<()> {
    let cache_dir = get_cache_dir()?;

    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir).with_context(|| {
            format!("Failed to remove cache directory: {}", cache_dir.display())
        })?;
        println!("✅ Cache cleared: {}", cache_dir.display());
    } else {
        println!("   Cache directory doesn't exist");
    }

    Ok(())
}

/// Serialize all rbconfig.rb files to JSON and delete the originals
pub fn serialize_rbconfigs(ruby_platform: &str) -> Result<usize> {
    use walkdir::WalkDir;

    let cache_dir = get_cache_dir()?;
    let platform_dir = cache_dir.join("rubies").join(ruby_platform);

    if !platform_dir.exists() {
        return Ok(0);
    }

    let mut count = 0;

    // Walk the directory tree looking for rbconfig.rb files
    for entry in WalkDir::new(&platform_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.file_name() == Some("rbconfig.rb".as_ref()) {
            // Parse the rbconfig.rb
            match crate::rbconfig_parser::RbConfigParser::from_file(path) {
                Ok(parser) => {
                    // Compute prefix from path
                    let prefix = crate::rbconfig_parser::RbConfigParser::compute_prefix(path)
                        .unwrap_or_default();

                    // Serialize to JSON
                    let serialized = parser.to_serialized(&prefix);
                    let json_path = path.with_file_name("rbconfig.json");

                    let json = serde_json::to_string_pretty(&serialized)?;
                    fs::write(&json_path, json)?;

                    // Delete the original .rb file
                    fs::remove_file(path)?;

                    count += 1;
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                }
            }
        }
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_library_file() {
        assert!(is_library_file("libruby.a"));
        assert!(is_library_file("libruby.so"));
        assert!(is_library_file("libruby.so.3.2"));
        assert!(is_library_file("ruby.dylib"));
        assert!(is_library_file("ruby.dll"));
        assert!(is_library_file("ruby.lib"));

        assert!(!is_library_file("ruby"));
        assert!(!is_library_file("ruby.h"));
        assert!(!is_library_file("ruby.txt"));
        assert!(!is_library_file(""));
    }

    #[test]
    fn test_get_cache_dir() {
        let cache_dir = get_cache_dir().unwrap();
        assert!(cache_dir.ends_with("rb-sys") || cache_dir.ends_with("RB_SYS_CACHE_DIR"));
    }

    #[test]
    fn test_categorize_path_ruby() {
        let config = ExtractionConfig {
            ruby_platform: "x86_64-linux".to_string(),
            sysroot_prefixes: vec![
                "usr/include".to_string(),
                "usr/lib/x86_64-linux-gnu".to_string(),
            ],
        };

        // Ruby include files
        assert_eq!(
            categorize_path(
                "usr/local/rake-compiler/rubies/ruby-3.4.5-x86_64-linux-gnu/include/ruby.h",
                &config
            ),
            Some(FileCategory::Ruby)
        );

        // rbconfig.rb
        assert_eq!(
            categorize_path(
                "usr/local/rake-compiler/rubies/ruby-3.4.5-x86_64-linux-gnu/lib/ruby/3.4.0/x86_64-linux-gnu/rbconfig.rb",
                &config
            ),
            Some(FileCategory::Ruby)
        );

        // Random Ruby file (should be None)
        assert_eq!(
            categorize_path(
                "usr/local/rake-compiler/rubies/ruby-3.4.5-x86_64-linux-gnu/bin/ruby",
                &config
            ),
            None
        );
    }

    #[test]
    fn test_categorize_path_sysroot() {
        let config = ExtractionConfig {
            ruby_platform: "x86_64-linux".to_string(),
            sysroot_prefixes: vec![
                "usr/include".to_string(),
                "usr/lib/x86_64-linux-gnu".to_string(),
            ],
        };

        // Sysroot include files
        assert_eq!(
            categorize_path("usr/include/stdio.h", &config),
            Some(FileCategory::Sysroot)
        );

        // Sysroot lib files
        assert_eq!(
            categorize_path("usr/lib/x86_64-linux-gnu/libc.a", &config),
            Some(FileCategory::Sysroot)
        );

        // Non-sysroot path
        assert_eq!(categorize_path("usr/bin/something", &config), None);
    }
}
