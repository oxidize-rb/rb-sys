//! Trait-based abstraction for Zig toolchain wrappers.
//!
//! Provides a template method pattern for wrapping Zig commands.
//! Implementors customize specific phases while the trait provides
//! the overall execution flow.

use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info};

/// Common interface for shim CLI arguments.
pub trait ShimArgs {
    fn zig_path(&self) -> &PathBuf;
    fn user_args(&self) -> &[String];
    fn target(&self) -> Option<&str> {
        None
    }
    fn sysroot(&self) -> Option<&PathBuf> {
        None
    }
}

/// Common interface for Zig toolchain shims.
///
/// Implements the template method pattern for tool execution.
/// Subclasses override specific methods to customize behavior.
pub trait ZigShim: Sized {
    type Args: ShimArgs;

    /// Returns the Zig subcommand name (e.g., "cc", "ar", "ld.lld")
    fn subcommand(&self) -> &str;

    /// Returns the parsed target if available
    fn target(&self) -> Option<&crate::zig::target::RustTarget> {
        None
    }

    /// Validates platform requirements (sysroot, SDKROOT, etc.)
    fn validate(&self, _args: &Self::Args) -> Result<()> {
        Ok(())
    }

    /// Adds platform-specific flags to the command
    fn add_platform_flags(&self, _cmd: &mut Command, _args: &Self::Args) -> Result<()> {
        Ok(())
    }

    /// Filters user arguments for tool compatibility
    fn filter_args(&self, args: &[String]) -> Vec<String>;

    /// Builds the base command with zig path and subcommand
    fn build_command(&self, args: &Self::Args) -> Command {
        let mut cmd = Command::new(args.zig_path());
        cmd.arg(self.subcommand());
        cmd
    }

    /// Template method: orchestrates the entire execution lifecycle
    ///
    /// Steps:
    /// 1. Validate requirements
    /// 2. Build base command
    /// 3. Add platform-specific flags
    /// 4. Filter user arguments
    /// 5. Execute and handle exit status
    fn run(&self, args: Self::Args) -> Result<()> {
        debug!(
            tool = self.subcommand(),
            target = ?self.target().map(|t| &t.raw)
        );

        // Step 1: Validate
        self.validate(&args)?;

        // Step 2: Build command
        let mut cmd = self.build_command(&args);

        // Step 3: Add platform flags
        self.add_platform_flags(&mut cmd, &args)?;

        // Step 4: Filter and add user arguments
        let user_args = args.user_args();
        let filtered_args = self.filter_args(user_args);
        debug!(
            original_args = ?user_args,
            filtered_args = ?filtered_args
        );

        for arg in filtered_args {
            cmd.arg(arg);
        }

        // Step 5: Execute
        info!(command = ?cmd, "Executing Zig command");
        let status = cmd
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to execute zig {}: {}", self.subcommand(), e))?;

        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }

        Ok(())
    }
}
