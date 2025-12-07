use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use futures::stream::{self, StreamExt, TryStreamExt};
use oci_distribution::client::{ClientConfig, ClientProtocol};
use oci_distribution::secrets::RegistryAuth;
use oci_distribution::{Client, Reference};
use std::fs;
use std::io::Read;
use std::path::Path;
use tar::Archive;

const RUBY_PATH_PREFIXES: &[&str] = &[
    "usr/local/rake-compiler/rubies/",
    "usr/local/rake-compiler/ruby/",
];

/// Extract a platform from an OCI image to the cache directory
pub async fn extract_platform(
    image_ref: &str,
    ruby_platform: &str,
    sysroot_prefixes: &[String],
    dest_dir: &Path,
) -> Result<Vec<String>> {
    let reference: Reference = image_ref
        .parse()
        .with_context(|| format!("Failed to parse image reference: {image_ref}"))?;

    // Determine platform from ruby_platform string
    let platform = determine_platform_from_ruby_platform(ruby_platform);
    let client = create_oci_client(platform)?;
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
        .with_context(|| format!("Failed to pull image: {image_ref}"))?;

    // Process layers in parallel on blocking pool (max 4 concurrent per platform)
    let ruby_platform_owned = ruby_platform.to_string();
    let sysroot_prefixes_owned: Vec<String> = sysroot_prefixes.to_vec();
    let dest_dir_owned = dest_dir.to_path_buf();

    let layer_results: Vec<Vec<String>> = stream::iter(image_data.layers)
        .map(|layer| {
            let ruby_platform = ruby_platform_owned.clone();
            let sysroot_prefixes = sysroot_prefixes_owned.clone();
            let dest_dir = dest_dir_owned.clone();

            async move {
                tokio::task::spawn_blocking(move || {
                    process_layer_blob(&layer.data, &ruby_platform, &sysroot_prefixes, &dest_dir)
                })
                .await
                .context("Layer extraction task panicked")?
            }
        })
        .buffer_unordered(4)
        .try_collect()
        .await?;

    // Flatten and deduplicate versions
    let mut ruby_versions: Vec<String> = layer_results.into_iter().flatten().collect();
    ruby_versions.sort();
    ruby_versions.dedup();

    Ok(ruby_versions)
}

fn determine_platform_from_ruby_platform(ruby_platform: &str) -> Option<&'static str> {
    if ruby_platform.contains("x86_64") || ruby_platform.contains("x64-mingw") {
        Some("linux/amd64")
    } else if ruby_platform.contains("aarch64") || ruby_platform.contains("arm64") {
        Some("linux/arm64")
    } else if ruby_platform.contains("arm-") {
        Some("linux/arm/v7")
    } else if ruby_platform.contains("x86-") {
        Some("linux/386")
    } else {
        None
    }
}

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

fn get_registry_auth() -> RegistryAuth {
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        RegistryAuth::Basic("token".to_string(), token)
    } else {
        RegistryAuth::Anonymous
    }
}

fn process_layer_blob(
    blob: &[u8],
    ruby_platform: &str,
    sysroot_prefixes: &[String],
    dest_dir: &Path,
) -> Result<Vec<String>> {
    let decoder = GzDecoder::new(blob);
    let mut archive = Archive::new(decoder);

    let mut ruby_versions = Vec::new();

    for entry_result in archive.entries().context("Failed to read tar entries")? {
        let mut entry = entry_result.context("Failed to read tar entry")?;
        let path = entry.path().context("Failed to get entry path")?.into_owned();
        let path_str = path.to_string_lossy().to_string();

        // Skip docker whiteout files
        if path_str.contains("/.wh.") {
            continue;
        }

        // Skip shared libraries
        if path_str.contains(".so") {
            continue;
        }

        // Check if it's a Ruby file
        let mut is_ruby = false;
        for prefix in RUBY_PATH_PREFIXES {
            if path_str.starts_with(prefix) {
                // Extract Ruby version from path
                if let Some(version) = extract_ruby_version(&path_str) {
                    if !ruby_versions.contains(&version) {
                        ruby_versions.push(version);
                    }
                }

                // Headers - always extract
                if path_str.contains("/include/") {
                    is_ruby = true;
                    break;
                }
                // rbconfig.rb - extract
                if path_str.ends_with("rbconfig.rb") {
                    is_ruby = true;
                    break;
                }
                // Static libraries only (.a files)
                if path_str.ends_with(".a") {
                    is_ruby = true;
                    break;
                }
            }
        }

        if is_ruby {
            extract_ruby_entry(&mut entry, &path, ruby_platform, dest_dir)?;
            continue;
        }

        // Check if it's a sysroot file
        // Note: tar paths don't have leading slash, so strip it from sysroot_prefixes
        for sysroot_prefix in sysroot_prefixes {
            let prefix_to_match = sysroot_prefix.strip_prefix('/').unwrap_or(sysroot_prefix);
            if path_str.starts_with(prefix_to_match) {
                // Keep all headers
                if path_str.ends_with(".h") || path_str.ends_with(".def") {
                    extract_sysroot_entry(&mut entry, &path, ruby_platform, dest_dir)?;
                    break;
                }
                // Static libraries
                if path_str.ends_with(".a") {
                    extract_sysroot_entry(&mut entry, &path, ruby_platform, dest_dir)?;
                    break;
                }
                // Object files (crt*.o, etc.)
                if path_str.ends_with(".o") {
                    extract_sysroot_entry(&mut entry, &path, ruby_platform, dest_dir)?;
                    break;
                }
            }
        }
    }

    Ok(ruby_versions)
}

fn extract_ruby_version(path: &str) -> Option<String> {
    // Path like: usr/local/rake-compiler/rubies/ruby-3.4.1-x86_64-linux/...
    for prefix in RUBY_PATH_PREFIXES {
        if let Some(rest) = path.strip_prefix(prefix) {
            if let Some(slash_pos) = rest.find('/') {
                let ruby_dir = &rest[..slash_pos];
                // ruby_dir is like "ruby-3.4.1-x86_64-linux"
                if let Some(version_start) = ruby_dir.strip_prefix("ruby-") {
                    // Find the version part (before the platform)
                    if let Some(dash_pos) = version_start.rfind('-') {
                        if let Some(second_dash) = version_start[..dash_pos].rfind('-') {
                            return Some(version_start[..second_dash].to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

fn strip_ruby_path_prefix(path: &str) -> Option<String> {
    for prefix in RUBY_PATH_PREFIXES {
        if let Some(rest) = path.strip_prefix(prefix) {
            return Some(rest.to_string());
        }
    }
    None
}

fn extract_ruby_entry<R: Read>(
    entry: &mut tar::Entry<R>,
    path: &Path,
    ruby_platform: &str,
    dest_dir: &Path,
) -> Result<()> {
    let path_str = path.to_string_lossy();
    let relative_path = strip_ruby_path_prefix(&path_str)
        .context("Path doesn't start with expected prefix")?;

    let dest_path = dest_dir
        .join(ruby_platform)
        .join("rubies")
        .join(relative_path);

    write_entry_to_path(entry, &dest_path)
}

fn extract_sysroot_entry<R: Read>(
    entry: &mut tar::Entry<R>,
    path: &Path,
    ruby_platform: &str,
    dest_dir: &Path,
) -> Result<()> {
    let path_str = path.to_string_lossy();

    let dest_path = dest_dir
        .join(ruby_platform)
        .join("sysroot")
        .join(&*path_str);

    write_entry_to_path(entry, &dest_path)
}

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
