use anyhow::Result;

use crate::platform::Platform;

pub fn run() -> Result<()> {
    println!("Supported cross-compilation platforms:\n");
    println!(
        "  {:<24} {:<40} GLIBC",
        "RUBY PLATFORM", "RUST TARGET"
    );
    println!("  {}", "-".repeat(80));

    for p in Platform::all() {
        println!(
            "  {:<24} {:<40} {}",
            p.ruby_platform,
            p.rust_target,
            p.glibc_version.unwrap_or("—"),
        );
    }

    println!();
    Ok(())
}
