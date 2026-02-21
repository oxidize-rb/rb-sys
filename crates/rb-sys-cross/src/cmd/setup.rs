use anyhow::Result;

use crate::platform::Platform;
use crate::toolchain;

pub fn run(platforms: &[String]) -> Result<()> {
    let targets: Vec<&Platform> = if platforms.is_empty() {
        Platform::all().iter().collect()
    } else {
        platforms
            .iter()
            .map(|p| Platform::find(p))
            .collect::<Result<Vec<_>>>()?
    };

    eprintln!("Setting up toolchains for cross-compilation...\n");
    toolchain::setup_all(&targets)?;
    eprintln!("\nAll toolchains ready.");
    Ok(())
}
