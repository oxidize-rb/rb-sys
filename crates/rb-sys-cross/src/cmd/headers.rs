use anyhow::Result;

use crate::headers::{build, download};
use crate::platform::Platform;

pub fn run_list() -> Result<()> {
    let cached = download::list_cached()?;

    if cached.is_empty() {
        println!("No cached header bundles found.");
        println!("Headers are downloaded automatically on first build.");
        return Ok(());
    }

    println!("Cached Ruby header bundles:\n");
    println!("  {:<24} RUBY VERSION", "PLATFORM");
    println!("  {}", "-".repeat(50));

    for (platform, version) in &cached {
        println!("  {platform:<24} {version}");
    }

    println!();
    Ok(())
}

pub fn run_build(platform_name: &str, ruby_version: &str) -> Result<()> {
    let plat = Platform::find(platform_name)?;
    let dest = build::build_ruby_headers(plat, ruby_version)?;
    println!("Headers built and cached at: {}", dest.display());
    Ok(())
}
