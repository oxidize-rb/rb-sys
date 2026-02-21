use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tar::{Builder, Header};

use super::gemspec::{CrossConfig, GemMetadata};
use crate::util;

/// A compiled artifact for a specific Ruby version.
pub struct VersionedArtifact {
    pub ruby_version: String,
    pub artifact_path: PathBuf,
}

/// Options for packing a native gem.
pub struct GemPackOptions<'a> {
    pub meta: &'a GemMetadata,
    pub ruby_platform: &'a str,
    pub artifacts: &'a [VersionedArtifact],
    pub lib_ext: &'a str,
    pub extra_files: &'a [(PathBuf, Vec<u8>)],
    pub config: &'a CrossConfig,
    pub output_dir: &'a Path,
}

/// Pack a native gem from compiled artifacts for multiple Ruby versions.
///
/// A `.gem` file is a POSIX tar containing:
/// - `metadata.gz` (gzipped YAML gemspec)
/// - `data.tar.gz` (gzipped tar of gem files)
/// - `checksums.yaml.gz` (gzipped YAML with SHA256 checksums)
///
/// The .so files are placed in versioned directories like:
///   lib/<gem_name>/3.3/<crate_name>.so
///   lib/<gem_name>/3.4/<crate_name>.so
pub fn pack_native_gem(opts: &GemPackOptions) -> Result<PathBuf> {
    let gem_name = opts
        .config
        .gem
        .name
        .as_deref()
        .unwrap_or(&opts.meta.name);

    std::fs::create_dir_all(opts.output_dir)
        .with_context(|| format!("creating output dir {}", opts.output_dir.display()))?;

    // Build the file list for the gemspec
    let ruby_versions: Vec<String> = opts.artifacts.iter().map(|a| a.ruby_version.clone()).collect();
    let file_list = build_file_list(
        gem_name,
        &opts.meta.crate_name,
        opts.lib_ext,
        &ruby_versions,
        opts.extra_files,
    );

    // Generate metadata.gz
    let gemspec_yaml = super::gemspec::generate_gemspec_yaml(
        opts.meta,
        opts.ruby_platform,
        &ruby_versions,
        &file_list,
        opts.config,
    );
    let metadata_gz = util::gz_compress(gemspec_yaml.as_bytes())?;

    // Build data.tar.gz
    let data_tar_gz = build_data_tar(
        gem_name,
        &opts.meta.crate_name,
        opts.artifacts,
        opts.lib_ext,
        opts.extra_files,
    )?;

    // Build checksums.yaml.gz
    let checksums_yaml = build_checksums_yaml(&metadata_gz, &data_tar_gz);
    let checksums_gz = util::gz_compress(checksums_yaml.as_bytes())?;

    // Assemble the outer .gem tar
    let gem_filename = format!(
        "{gem_name}-{}-{}.gem",
        opts.meta.version, opts.ruby_platform
    );
    let gem_path = opts.output_dir.join(&gem_filename);
    let gem_file = std::fs::File::create(&gem_path)
        .with_context(|| format!("creating {}", gem_path.display()))?;

    let mut outer = Builder::new(gem_file);
    append_bytes(&mut outer, "metadata.gz", &metadata_gz)?;
    append_bytes(&mut outer, "data.tar.gz", &data_tar_gz)?;
    append_bytes(&mut outer, "checksums.yaml.gz", &checksums_gz)?;
    outer.finish().context("finishing gem tar")?;

    eprintln!("Packed: {}", gem_path.display());
    Ok(gem_path)
}

/// Generate the version-aware extension loader Ruby file.
/// This goes at lib/<gem_name>/extension.rb and loads the correct versioned .so.
fn extension_loader_rb(gem_name: &str, crate_name: &str) -> String {
    // NOTE: Uses HEREDOC-style construction to avoid issues with Ruby's #{} interpolation
    // inside Rust raw strings.
    let mut s = String::new();
    s.push_str("# frozen_string_literal: true\n\n");
    s.push_str("begin\n");
    s.push_str(&format!(
        "  # Native precompiled gems package shared libraries in lib/{gem_name}/<ruby_version>/\n"
    ));
    s.push_str("  ruby_version = /\\d+\\.\\d+/.match(RUBY_VERSION)\n");
    // Ruby string interpolation: "#{ruby_version}/crate_name"
    s.push_str(&format!(
        "  require_relative \"#{{ruby_version}}/{crate_name}\"\n"
    ));
    s.push_str("rescue LoadError\n");
    s.push_str("  # Fall back to extension compiled upon installation\n");
    s.push_str(&format!("  require \"{gem_name}/{crate_name}\"\n"));
    s.push_str("end\n");
    s
}

/// Build the list of files that go in the gemspec.
fn build_file_list(
    gem_name: &str,
    crate_name: &str,
    lib_ext: &str,
    ruby_versions: &[String],
    extra_files: &[(PathBuf, Vec<u8>)],
) -> Vec<String> {
    let mut files = Vec::new();

    // Top-level entrypoint and extension loader
    files.push(format!("lib/{gem_name}.rb"));
    files.push(format!("lib/{gem_name}/extension.rb"));

    // Versioned .so files
    for ver in ruby_versions {
        files.push(format!("lib/{gem_name}/{ver}/{crate_name}.{lib_ext}"));
    }

    // Extra Ruby files
    for (path, _) in extra_files {
        files.push(path.to_string_lossy().to_string());
    }

    files.sort();
    files
}

/// Build the data.tar.gz containing compiled libraries and extra files.
fn build_data_tar(
    gem_name: &str,
    crate_name: &str,
    artifacts: &[VersionedArtifact],
    lib_ext: &str,
    extra_files: &[(PathBuf, Vec<u8>)],
) -> Result<Vec<u8>> {
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    {
        let mut tar = Builder::new(&mut gz);

        // Add the extension loader
        let loader = extension_loader_rb(gem_name, crate_name);
        append_bytes(
            &mut tar,
            &format!("lib/{gem_name}/extension.rb"),
            loader.as_bytes(),
        )?;

        // Add versioned .so files: lib/<gem_name>/<ruby_version>/<crate_name>.<ext>
        for artifact in artifacts {
            let lib_path = format!(
                "lib/{gem_name}/{}/{crate_name}.{lib_ext}",
                artifact.ruby_version
            );
            let artifact_bytes = std::fs::read(&artifact.artifact_path).with_context(|| {
                format!("reading artifact {}", artifact.artifact_path.display())
            })?;
            append_bytes(&mut tar, &lib_path, &artifact_bytes)?;
        }

        // Add the top-level lib/<gem_name>.rb entrypoint.
        // If the user provided one in extra_files, prepend the extension loader require.
        // Otherwise, generate one that just loads the extension.
        let entrypoint_path = format!("lib/{gem_name}.rb");
        let extension_require = format!("require \"{gem_name}/extension\"\n");
        let user_entrypoint = extra_files
            .iter()
            .find(|(p, _)| p.to_string_lossy() == entrypoint_path);
        let entrypoint_contents = match user_entrypoint {
            Some((_, contents)) => {
                // Prepend the extension loader require to the user's file
                let mut combined = extension_require.into_bytes();
                combined.extend_from_slice(contents);
                combined
            }
            None => {
                // Generate a minimal entrypoint
                extension_require.into_bytes()
            }
        };
        append_bytes(&mut tar, &entrypoint_path, &entrypoint_contents)?;

        // Add any extra Ruby files (skip the entrypoint since we already handled it)
        for (path, contents) in extra_files {
            if path.to_string_lossy() == entrypoint_path {
                continue;
            }
            append_bytes(&mut tar, &path.to_string_lossy(), contents)?;
        }

        tar.finish().context("finishing data tar")?;
    }
    gz.finish().context("finishing data gzip")
}

fn build_checksums_yaml(metadata_gz: &[u8], data_tar_gz: &[u8]) -> String {
    let meta_sha = util::hex_sha256(metadata_gz);
    let data_sha = util::hex_sha256(data_tar_gz);

    format!("---\nSHA256:\n  metadata.gz: {meta_sha}\n  data.tar.gz: {data_sha}\n")
}

fn append_bytes<W: Write>(tar: &mut Builder<W>, path: &str, data: &[u8]) -> Result<()> {
    let mut header = Header::new_gnu();
    header.set_path(path).context("setting tar path")?;
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    header.set_mtime(0); // Reproducible builds
    header.set_cksum();
    tar.append(&header, data).context("appending to tar")?;
    Ok(())
}
