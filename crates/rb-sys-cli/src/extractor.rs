use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use oci_distribution::client::{ClientConfig, ClientProtocol};
use oci_distribution::secrets::RegistryAuth;
use oci_distribution::{Client, Reference};
use regex::Regex;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use tar::Archive;

const LAYER_SIZE_THRESHOLD: u64 = 1_000_000; // 1MB minimum layer size

// Compiled once, on first access
static RUBY_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^usr/local/rake-compiler/(rubies|ruby)/[^/]+/[^/]+/(include/.*|lib/ruby/[^/]+/[^/]+/rbconfig\.rb)$"
    )
    .expect("Invalid regex pattern")
});

fn strip_ruby_path_prefix(path_str: &str) -> Option<&str> {
    const RUBY_PATH_PREFIXES: &[&str] = &[
        "usr/local/rake-compiler/rubies/",
        "usr/local/rake-compiler/ruby/",
    ];
    
    for prefix in RUBY_PATH_PREFIXES {
        if let Some(stripped) = path_str.strip_prefix(prefix) {
            return Some(stripped);
        }
    }
    None
}

/// Extract Ruby headers and libraries from a Docker image OCI registry
///
/// This function streams image layers directly from the registry without Docker,
/// extracting only the Ruby headers (include/) and libraries (lib/) to a local cache.
///
/// # Arguments
/// * `image_ref` - OCI image reference (e.g., "ghcr.io/rake-compiler/rake-compiler-dock-image:1.3.0-mri-x86_64-linux")
///
/// # Example
/// ```no_run
/// use rb_sys_cli::extractor::extract_headers;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     extract_headers("ghcr.io/rake-compiler/rake-compiler-dock-image:1.3.0-mri-x86_64-linux").await?;
///     Ok(())
/// }
/// ```
pub async fn extract_headers(image_ref: &str) -> Result<()> {
    println!("🐳 Extracting Ruby headers from image: {}", image_ref);

    // Parse the image reference
    let reference: Reference = image_ref
        .parse()
        .context("Failed to parse image reference")?;

    println!("   Registry: {}", reference.registry());
    println!("   Repository: {}", reference.repository());
    println!("   Tag: {}", reference.tag().unwrap_or("latest"));

    // Determine target platform from image tag
    // For x86_64 images, we need to specify linux/amd64 platform
    let platform = if reference.tag().map(|t| t.contains("x86_64")).unwrap_or(false) {
        Some("linux/amd64")
    } else if reference.tag().map(|t| t.contains("aarch64")).unwrap_or(false) {
        Some("linux/arm64")
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
    let auth = RegistryAuth::Anonymous;
    
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

    println!("\n🔍 Scanning {} layer(s) for Ruby files...", image_data.layers.len());

    let mut total_files_extracted = 0;

    for (idx, layer) in image_data.layers.iter().enumerate() {
        let size = layer.data.len() as u64;
        
        if size < LAYER_SIZE_THRESHOLD {
            println!(
                "   ⏭️  Layer {}/{}: Skipping small layer ({}KB)",
                idx + 1,
                image_data.layers.len(),
                size / 1024
            );
            continue;
        }

        println!(
            "   📥 Layer {}/{}: Processing ({}MB)",
            idx + 1,
            image_data.layers.len(),
            size / 1_048_576
        );

        // Clone the data to move into blocking thread
        let layer_data = layer.data.clone();
        
        // Offload CPU/IO intensive work to a blocking thread
        let files_extracted = tokio::task::spawn_blocking(move || {
            process_layer_blob(&layer_data)
        })
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
    
    // Set platform-specific configuration if provided
    if let Some(plat) = platform {
        let plat_string = plat.to_string(); // Convert to owned String for 'static lifetime
        config.platform_resolver = Some(Box::new(move |platforms| {
            // Parse the requested platform (e.g., "linux/amd64")
            let parts: Vec<&str> = plat_string.split('/').collect();
            if parts.len() == 2 {
                let os = parts[0];
                let arch = parts[1];
                
                // Find matching platform in available platforms
                for entry in platforms {
                    if let Some(platform) = &entry.platform {
                        if platform.os == os && platform.architecture == arch {
                            return Some(entry.digest.clone());
                        }
                    }
                }
            }
            // Default to first platform if no match
            platforms.first().map(|e| e.digest.clone())
        }));
    }

    Ok(Client::new(config))
}

/// Process a single layer blob and extract matching files
fn process_layer_blob(blob: &[u8]) -> Result<usize> {
    // Create a GzDecoder to decompress the layer
    let decoder = GzDecoder::new(blob);

    // Create a tar archive from the decompressed stream
    let mut archive = Archive::new(decoder);

    let mut files_extracted = 0;

    // Iterate through all entries in the tar archive
    for entry_result in archive.entries().context("Failed to read tar entries")? {
        let mut entry = entry_result.context("Failed to read tar entry")?;

        let path = entry.path().context("Failed to get entry path")?;
        let path_str = path.to_string_lossy();

        // Check if this entry matches our criteria using the static regex
        if !RUBY_PATH_REGEX.is_match(&path_str) {
            continue;
        }

        // Additional filtering for lib/ - only extract actual libraries or rbconfig.rb
        if path_str.contains("/lib/") {
            let file_name = path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("");

            // Extract static/shared libraries, archives, or rbconfig.rb
            if !is_library_file(file_name) && file_name != "rbconfig.rb" {
                continue;
            }
        }

        // Convert path to owned PathBuf to avoid borrowing issues
        let path_owned = path.to_path_buf();

        // Extract the file
        extract_entry(&mut entry, &path_owned)?;
        files_extracted += 1;
    }

    Ok(files_extracted)
}

/// Check if a file is a library file we want to extract
fn is_library_file(filename: &str) -> bool {
    filename.ends_with(".a")      // Static libraries (Unix)
        || filename.ends_with(".so")    // Shared libraries (Linux)
        || filename.ends_with(".dylib") // Shared libraries (macOS)
        || filename.ends_with(".dll")   // Shared libraries (Windows)
        || filename.ends_with(".lib")   // Import libraries (Windows)
}

/// Extract a single tar entry to the cache directory
fn extract_entry<R: Read>(entry: &mut tar::Entry<R>, path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();

    // Strip the prefix: usr/local/rake-compiler/rubies/ or /ruby/
    let relative_path = strip_ruby_path_prefix(&path_str)
        .context("Path doesn't start with expected prefix")?;

    // Get the cache directory
    let cache_dir = get_cache_dir()?;
    let dest_path = cache_dir.join("rubies").join(relative_path);

    // Skip if it's a directory entry (tar archives include these)
    if entry.header().entry_type().is_dir() {
        fs::create_dir_all(&dest_path)
            .with_context(|| format!("Failed to create directory: {}", dest_path.display()))?;
        return Ok(());
    }

    // Create parent directories
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
    }

    // Extract the file
    let mut dest_file = fs::File::create(&dest_path)
        .with_context(|| format!("Failed to create file: {}", dest_path.display()))?;

    std::io::copy(entry, &mut dest_file)
        .with_context(|| format!("Failed to write file: {}", dest_path.display()))?;

    // Preserve permissions on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(mode) = entry.header().mode() {
            let perms = fs::Permissions::from_mode(mode);
            let _ = fs::set_permissions(&dest_path, perms);
        }
    }

    Ok(())
}

/// Get the cache directory for extracted Ruby files
pub fn get_cache_dir() -> Result<PathBuf> {
    let cache_dir = if let Ok(override_dir) = std::env::var("GEM_FORGE_CACHE_DIR") {
        PathBuf::from(override_dir)
    } else if let Some(cache_home) = std::env::var_os("XDG_CACHE_HOME") {
        PathBuf::from(cache_home).join("gem-forge")
    } else if let Some(home_dir) = dirs::home_dir() {
        home_dir.join(".cache").join("gem-forge")
    } else {
        anyhow::bail!("Could not determine cache directory (no HOME or XDG_CACHE_HOME)")
    };

    fs::create_dir_all(&cache_dir)
        .with_context(|| format!("Failed to create cache directory: {}", cache_dir.display()))?;

    Ok(cache_dir)
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
        fs::remove_dir_all(&cache_dir)
            .with_context(|| format!("Failed to remove cache directory: {}", cache_dir.display()))?;
        println!("✅ Cache cleared: {}", cache_dir.display());
    } else {
        println!("   Cache directory doesn't exist");
    }

    Ok(())
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
        assert!(cache_dir.ends_with("gem-forge") || cache_dir.ends_with("GEM_FORGE_CACHE_DIR"));
    }

    #[test]
    fn test_ruby_path_regex() {
        let regex = Regex::new(r"^usr/local/rake-compiler/rubies/[^/]+/(include|lib)/").unwrap();

        assert!(regex.is_match("usr/local/rake-compiler/rubies/ruby-3.2.2-x86_64-linux/include/ruby.h"));
        assert!(regex.is_match("usr/local/rake-compiler/rubies/ruby-3.2.2-x86_64-linux/lib/libruby.a"));
        assert!(regex.is_match("usr/local/rake-compiler/rubies/ruby-3.1.0-aarch64-linux/include/ruby/ruby.h"));

        assert!(!regex.is_match("usr/local/rake-compiler/rubies/ruby-3.2.2-x86_64-linux/bin/ruby"));
        assert!(!regex.is_match("usr/local/rake-compiler/rubies/ruby-3.2.2-x86_64-linux/share/man/ruby.1"));
        assert!(!regex.is_match("usr/bin/ruby"));
    }
}
