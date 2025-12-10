# ZigShim Trait Design - Implementation Guide

## Executive Summary

This document outlines the implementation of a trait-based abstraction for Zig toolchain wrappers (cc, ld, ar, dlltool) in the rb-sys project. The design consolidates common execution patterns while allowing tool-specific customization through a template method pattern.

**Current State**: The codebase has separate implementations for each tool (cc.rs, ld.rs, ar.rs, dlltool.rs) with significant code duplication in the execution lifecycle.

**Goal**: Introduce `ZigShim` and `ShimArgs` traits to eliminate duplication and provide a consistent, extensible pattern.

---

## Current Architecture Analysis

### Existing Files
- `crates/rb-sys-cli/src/zig/cc.rs` (221 lines) - C/C++ compiler wrapper
- `crates/rb-sys-cli/src/zig/ld.rs` (310 lines) - Linker wrapper
- `crates/rb-sys-cli/src/zig/ar.rs` (87 lines) - Archiver wrapper
- `crates/rb-sys-cli/src/zig/dlltool.rs` (TBD) - dlltool wrapper
- `crates/rb-sys-cli/src/zig/args.rs` (32K lines) - Argument filtering logic
- `crates/rb-sys-cli/src/zig/shim.rs` (348 lines) - Bash shim generation (NOT the trait)

### Current Execution Pattern (Duplicated)

Each tool follows this pattern:

```rust
pub fn run_cc(args: ZigCcArgs, is_cxx: bool) -> Result<()> {
    // 1. Parse target
    let target = RustTarget::parse(&args.target)?;
    
    // 2. Validate requirements (sysroot, SDKROOT)
    validate_requirements(&target, &args)?;
    
    // 3. Build base command
    let mut cmd = Command::new(&args.zig_path);
    cmd.arg(subcommand);
    
    // 4. Add platform-specific flags
    add_platform_args(&mut cmd, &target, &args)?;
    
    // 5. Filter user arguments
    let filtered_args = filter.filter_cc_args(&args.args);
    for arg in filtered_args {
        cmd.arg(arg);
    }
    
    // 6. Execute
    let status = cmd.status()?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}
```

**Duplication Points**:
- Steps 1, 3, 6 are nearly identical across all tools
- Validation logic is similar but tool-specific
- Platform flag addition varies by tool
- Argument filtering is tool-specific

---

## Trait Design

### Core Traits

#### `ShimArgs` Trait
```rust
pub trait ShimArgs {
    fn zig_path(&self) -> &PathBuf;
    fn user_args(&self) -> &[String];
    fn target(&self) -> Option<&str> { None }
    fn sysroot(&self) -> Option<&PathBuf> { None }
}
```

**Purpose**: Abstract over different argument types (ZigCcArgs, ZigLdArgs, ZigArArgs).

**Implementations**:
- `ZigCcArgs` - has target, sysroot
- `ZigLdArgs` - has target, sysroot
- `ZigArArgs` - no target, no sysroot
- `ZigDlltoolArgs` - has target, no sysroot

#### `ZigShim` Trait
```rust
pub trait ZigShim: Sized {
    type Args: ShimArgs;
    
    fn subcommand(&self) -> &str;
    fn target(&self) -> Option<&RustTarget> { None }
    fn validate(&self, _args: &Self::Args) -> Result<()> { Ok(()) }
    fn add_platform_flags(&self, _cmd: &mut Command, _args: &Self::Args) -> Result<()> { Ok(()) }
    fn filter_args(&self, args: &[String]) -> Vec<String>;
    fn build_command(&self, args: &Self::Args) -> Command { ... }
    fn run(&self, args: Self::Args) -> Result<()> { ... }
}
```

**Template Method Pattern**: The `run()` method orchestrates the entire lifecycle:
1. Validate requirements
2. Build base command
3. Add platform-specific flags
4. Filter user arguments
5. Execute and handle exit status

---

## Implementation Plan

### Phase 1: Create Trait Definitions (New File)

**File**: `crates/rb-sys-cli/src/zig/tool.rs`

```rust
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
    fn target(&self) -> Option<&str> { None }
    fn sysroot(&self) -> Option<&PathBuf> { None }
}

/// Common interface for Zig toolchain shims.
pub trait ZigShim: Sized {
    type Args: ShimArgs;
    
    fn subcommand(&self) -> &str;
    fn target(&self) -> Option<&crate::zig::target::RustTarget> { None }
    fn validate(&self, _args: &Self::Args) -> Result<()> { Ok(()) }
    fn add_platform_flags(&self, _cmd: &mut Command, _args: &Self::Args) -> Result<()> { Ok(()) }
    fn filter_args(&self, args: &[String]) -> Vec<String>;
    
    fn build_command(&self, args: &Self::Args) -> Command {
        let mut cmd = Command::new(args.zig_path());
        cmd.arg(self.subcommand());
        cmd
    }
    
    fn run(&self, args: Self::Args) -> Result<()> {
        debug!(tool = self.subcommand(), target = ?self.target().map(|t| &t.raw));
        self.validate(&args)?;
        
        let mut cmd = self.build_command(&args);
        self.add_platform_flags(&mut cmd, &args)?;
        
        let user_args = args.user_args();
        let filtered_args = self.filter_args(user_args);
        debug!(original_args = ?user_args, filtered_args = ?filtered_args);
        
        for arg in filtered_args {
            cmd.arg(arg);
        }
        
        info!(command = ?cmd, "Executing Zig command");
        let status = cmd.status()
            .context(format!("Failed to execute zig {}", self.subcommand()))?;
        
        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
        
        Ok(())
    }
}
```

### Phase 2: Implement Traits for Each Tool

#### 2a. ZigCc Implementation

**File**: `crates/rb-sys-cli/src/zig/tools/cc.rs` (new)

```rust
use super::super::tool::{ShimArgs, ZigShim};
use super::super::target::RustTarget;
use super::super::args::ArgFilter;
use super::super::cpu::cpu_flag;
use super::super::target::Os;
use crate::platform::{LinuxConfig, MacOSConfig, WindowsConfig};
use anyhow::{bail, Result};
use std::path::PathBuf;
use std::process::Command;

pub struct ZigCc {
    pub target: RustTarget,
    pub is_cxx: bool,
}

impl ZigShim for ZigCc {
    type Args = ZigCcArgs;
    
    fn subcommand(&self) -> &str {
        if self.is_cxx { "c++" } else { "cc" }
    }
    
    fn target(&self) -> Option<&RustTarget> {
        Some(&self.target)
    }
    
    fn validate(&self, args: &ZigCcArgs) -> Result<()> {
        if self.target.requires_sysroot() {
            match &args.sysroot {
                Some(sysroot) => {
                    let config = LinuxConfig::new(&self.target, sysroot.clone());
                    if let Err(e) = config.validate() {
                        bail!("{}\n\nTo extract the sysroot, run:\n  cargo gem extract --target {}", 
                              e, self.target.raw);
                    }
                }
                None => {
                    bail!("Sysroot is required for Linux target: {}\n\nTo extract the sysroot, run:\n  cargo gem extract --target {}", 
                          self.target.raw, self.target.raw);
                }
            }
        }
        
        if self.target.requires_sdkroot() {
            let config = MacOSConfig::from_env_or_embedded(None)?;
            if let Err(e) = config.validate() {
                bail!("{e}");
            }
        }
        
        Ok(())
    }
    
    fn add_platform_flags(&self, cmd: &mut Command, args: &ZigCcArgs) -> Result<()> {
        let zig_target = self.target.to_zig_target();
        cmd.arg("-target").arg(&zig_target);
        
        if let Some(cpu) = cpu_flag(&self.target) {
            cmd.arg(format!("-mcpu={cpu}"));
        }
        
        cmd.arg("-g");
        cmd.arg("-fno-sanitize=all");
        
        match self.target.os {
            Os::Linux => {
                let sysroot = args.sysroot.as_ref().unwrap();
                let config = LinuxConfig::new(&self.target, sysroot.clone());
                for arg in config.cc_args() {
                    cmd.arg(arg);
                }
                if config.is_musl {
                    for arg in LinuxConfig::musl_defines() {
                        cmd.arg(arg);
                    }
                }
            }
            Os::Darwin => {
                let config = MacOSConfig::from_env_or_embedded(None)?;
                for arg in config.cc_args() {
                    cmd.arg(arg);
                }
            }
            Os::Windows => {
                for arg in WindowsConfig::cc_args() {
                    cmd.arg(arg);
                }
            }
        }
        
        Ok(())
    }
    
    fn filter_args(&self, args: &[String]) -> Vec<String> {
        let filter = ArgFilter::new(&self.target);
        filter.filter_cc_args(args)
    }
}

impl ShimArgs for ZigCcArgs {
    fn zig_path(&self) -> &PathBuf { &self.zig_path }
    fn user_args(&self) -> &[String] { &self.args }
    fn target(&self) -> Option<&str> { Some(&self.target) }
    fn sysroot(&self) -> Option<&PathBuf> { self.sysroot.as_ref() }
}

// Keep existing ZigCcArgs struct unchanged
#[derive(Args, Debug, Clone)]
pub struct ZigCcArgs {
    #[arg(long)]
    pub target: String,
    #[arg(long)]
    pub zig_path: PathBuf,
    #[arg(long)]
    pub sysroot: Option<PathBuf>,
    #[arg(last = true)]
    pub args: Vec<String>,
}
```

#### 2b. ZigLd Implementation

**File**: `crates/rb-sys-cli/src/zig/tools/ld.rs` (new)

```rust
use super::super::tool::{ShimArgs, ZigShim};
use super::super::target::{RustTarget, Os, Arch};
use super::super::args::{ArgFilter, LinkMode};
use crate::platform::LinuxConfig;
use anyhow::{bail, Result};
use std::path::PathBuf;
use std::process::Command;

pub struct ZigLd {
    pub target: RustTarget,
    pub link_mode: LinkMode,
}

impl ZigShim for ZigLd {
    type Args = ZigLdArgs;
    
    fn subcommand(&self) -> &str {
        match self.target.os {
            Os::Darwin => "ld64.lld",
            Os::Windows => "cc",
            _ => "ld.lld",
        }
    }
    
    fn target(&self) -> Option<&RustTarget> {
        Some(&self.target)
    }
    
    fn validate(&self, args: &ZigLdArgs) -> Result<()> {
        if self.target.requires_sysroot() {
            match &args.sysroot {
                Some(sysroot) => {
                    let config = LinuxConfig::new(&self.target, sysroot.clone());
                    if let Err(e) = config.validate() {
                        bail!("{}\n\nTo extract the sysroot, run:\n  cargo gem extract --target {}", 
                              e, self.target.raw);
                    }
                }
                None => {
                    bail!("Sysroot is required for Linux target: {}\n\nTo extract the sysroot, run:\n  cargo gem extract --target {}", 
                          self.target.raw, self.target.raw);
                }
            }
        }
        Ok(())
    }
    
    fn add_platform_flags(&self, cmd: &mut Command, args: &ZigLdArgs) -> Result<()> {
        match self.target.os {
            Os::Windows => {
                cmd.arg("-target").arg(self.target.to_zig_target());
                cmd.arg("-fno-sanitize=all");
            }
            _ => {
                if let Some(emulation) = linker_emulation(&self.target) {
                    cmd.arg("-m").arg(emulation);
                }
                
                match self.target.os {
                    Os::Linux => {
                        let sysroot = args.sysroot.as_ref().unwrap();
                        cmd.arg(format!("--sysroot={}", sysroot.display()));
                    }
                    Os::Darwin => {
                        if let Ok(sdkroot) = std::env::var("SDKROOT") {
                            cmd.arg("-syslibroot").arg(&sdkroot);
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
    
    fn filter_args(&self, args: &[String]) -> Vec<String> {
        let filter = ArgFilter::with_link_mode(&self.target, self.link_mode);
        filter.filter_link_args(args)
    }
}

impl ShimArgs for ZigLdArgs {
    fn zig_path(&self) -> &PathBuf { &self.zig_path }
    fn user_args(&self) -> &[String] { &self.args }
    fn target(&self) -> Option<&str> { Some(&self.target) }
    fn sysroot(&self) -> Option<&PathBuf> { self.sysroot.as_ref() }
}

fn linker_emulation(target: &RustTarget) -> Option<&'static str> {
    match target.os {
        Os::Darwin => None,
        Os::Windows => None,
        Os::Linux => Some(match target.arch {
            Arch::X86_64 => "elf_x86_64",
            Arch::Aarch64 => "aarch64linux",
            Arch::Arm => "armelf_linux_eabi",
        }),
    }
}

#[derive(Args, Debug, Clone)]
pub struct ZigLdArgs {
    #[arg(long)]
    pub target: String,
    #[arg(long)]
    pub zig_path: PathBuf,
    #[arg(long)]
    pub sysroot: Option<PathBuf>,
    #[arg(last = true)]
    pub args: Vec<String>,
}
```

#### 2c. ZigAr Implementation

**File**: `crates/rb-sys-cli/src/zig/tools/ar.rs` (new)

```rust
use super::super::tool::{ShimArgs, ZigShim};
use super::super::args::filter_ar_args;
use anyhow::Result;
use std::path::PathBuf;

pub struct ZigAr;

impl ZigShim for ZigAr {
    type Args = ZigArArgs;
    
    fn subcommand(&self) -> &str {
        "ar"
    }
    
    fn filter_args(&self, args: &[String]) -> Vec<String> {
        filter_ar_args(args)
    }
}

impl ShimArgs for ZigArArgs {
    fn zig_path(&self) -> &PathBuf { &self.zig_path }
    fn user_args(&self) -> &[String] { &self.args }
}

#[derive(Args, Debug, Clone)]
pub struct ZigArArgs {
    #[arg(long)]
    pub zig_path: PathBuf,
    #[arg(last = true)]
    pub args: Vec<String>,
}
```

#### 2d. ZigDlltool Implementation

**File**: `crates/rb-sys-cli/src/zig/tools/dlltool.rs` (new)

```rust
use super::super::tool::{ShimArgs, ZigShim};
use super::super::target::RustTarget;
use anyhow::Result;
use std::path::PathBuf;

pub struct ZigDlltool {
    pub target: RustTarget,
}

impl ZigShim for ZigDlltool {
    type Args = ZigDlltoolArgs;
    
    fn subcommand(&self) -> &str {
        "dlltool"
    }
    
    fn target(&self) -> Option<&RustTarget> {
        Some(&self.target)
    }
    
    fn filter_args(&self, args: &[String]) -> Vec<String> {
        // dlltool args don't need filtering
        args.to_vec()
    }
}

impl ShimArgs for ZigDlltoolArgs {
    fn zig_path(&self) -> &PathBuf { &self.zig_path }
    fn user_args(&self) -> &[String] { &self.args }
    fn target(&self) -> Option<&str> { Some(&self.target) }
}

#[derive(Args, Debug, Clone)]
pub struct ZigDlltoolArgs {
    #[arg(long)]
    pub target: String,
    #[arg(long)]
    pub zig_path: PathBuf,
    #[arg(last = true)]
    pub args: Vec<String>,
}
```

### Phase 3: Create Tools Module

**File**: `crates/rb-sys-cli/src/zig/tools/mod.rs` (new)

```rust
//! Tool-specific implementations of the ZigShim trait.

pub mod ar;
pub mod cc;
pub mod dlltool;
pub mod ld;

pub use ar::{ZigAr, ZigArArgs};
pub use cc::{ZigCc, ZigCcArgs};
pub use dlltool::{ZigDlltool, ZigDlltoolArgs};
pub use ld::{ZigLd, ZigLdArgs};
```

### Phase 4: Update Module Structure

**File**: `crates/rb-sys-cli/src/zig/mod.rs`

```rust
pub mod ar;
pub mod args;
pub mod cc;
pub mod cpu;
pub mod dlltool;
pub mod env;
pub mod ld;
pub mod libc;
pub mod manager;
pub mod shim;
pub mod target;
pub mod tool;      // NEW: Trait definitions
pub mod tools;     // NEW: Tool implementations

// Re-exports
pub use libc::get_zig_libc_includes;
pub use manager::resolve_zig_path;
pub use shim::generate_shims;
pub use target::RustTarget;
pub use tool::{ShimArgs, ZigShim};
pub use tools::{ZigAr, ZigArArgs, ZigCc, ZigCcArgs, ZigDlltool, ZigDlltoolArgs, ZigLd, ZigLdArgs};
```

### Phase 5: Update main.rs

**File**: `crates/rb-sys-cli/src/main.rs`

```rust
// Update command handling to use the trait
Commands::ZigCc(args) => {
    let target = zig::target::RustTarget::parse(&args.target)?;
    let shim = zig::tools::ZigCc { target, is_cxx: false };
    shim.run(args)?;
}
Commands::ZigCxx(args) => {
    let target = zig::target::RustTarget::parse(&args.target)?;
    let shim = zig::tools::ZigCc { target, is_cxx: true };
    shim.run(args)?;
}
Commands::ZigAr(args) => {
    let shim = zig::tools::ZigAr;
    shim.run(args)?;
}
Commands::ZigLd(args) => {
    let target = zig::target::RustTarget::parse(&args.target)?;
    // Determine link mode from context (may need adjustment)
    let link_mode = zig::args::LinkMode::Direct;
    let shim = zig::tools::ZigLd { target, link_mode };
    shim.run(args)?;
}
Commands::ZigDlltool(args) => {
    let target = zig::target::RustTarget::parse(&args.target)?;
    let shim = zig::tools::ZigDlltool { target };
    shim.run(args)?;
}
```

### Phase 6: Deprecate Old Functions (Optional)

Keep the old `run_cc()`, `run_ld()`, `run_ar()` functions as thin wrappers for backward compatibility during transition:

```rust
// In cc.rs
pub fn run_cc(args: ZigCcArgs, is_cxx: bool) -> Result<()> {
    let target = RustTarget::parse(&args.target)?;
    let shim = ZigCc { target, is_cxx };
    shim.run(args)
}
```

---

## Benefits

1. **DRY Principle**: Eliminates ~200 lines of duplicated execution logic
2. **Extensibility**: Adding new tools (ranlib, strip, etc.) requires only implementing the trait
3. **Clarity**: Each tool's specific behavior is isolated in its impl block
4. **Consistency**: All tools follow the same lifecycle pattern
5. **Testability**: Can mock the trait for testing the template method
6. **Type Safety**: Associated types ensure args match the tool

---

## Migration Strategy

### Step 1: Create Traits (No Breaking Changes)
- Add `tool.rs` with trait definitions
- Add `tools/` module with implementations
- Update `mod.rs` to export new types

### Step 2: Update main.rs
- Change command handlers to use trait implementations
- Keep old functions as wrappers initially

### Step 3: Deprecate Old Code
- Mark old `run_cc()`, `run_ld()`, `run_ar()` as deprecated
- Update documentation to use new trait-based approach

### Step 4: Remove Old Code (Major Version)
- Delete old implementations after deprecation period
- Clean up module structure

---

## Testing Strategy

### Unit Tests
- Test each trait implementation independently
- Mock `ShimArgs` for testing trait behavior
- Verify filtering logic still works

### Integration Tests
- Test full execution flow with real Zig commands
- Verify platform-specific flags are added correctly
- Test error handling and validation

### Regression Tests
- Ensure existing tests still pass
- Verify argument filtering produces same results

---

## File Structure After Implementation

```
crates/rb-sys-cli/src/zig/
├── mod.rs                 (updated)
├── tool.rs               (NEW - trait definitions)
├── tools/                (NEW - tool implementations)
│   ├── mod.rs
│   ├── cc.rs
│   ├── ld.rs
│   ├── ar.rs
│   └── dlltool.rs
├── cc.rs                 (keep as wrapper or deprecate)
├── ld.rs                 (keep as wrapper or deprecate)
├── ar.rs                 (keep as wrapper or deprecate)
├── dlltool.rs            (keep as wrapper or deprecate)
├── args.rs               (unchanged)
├── target.rs             (unchanged)
├── cpu.rs                (unchanged)
├── env.rs                (unchanged)
├── libc.rs               (unchanged)
├── manager.rs            (unchanged)
└── shim.rs               (unchanged)
```

---

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Breaking changes | Keep old functions as wrappers during transition |
| Trait complexity | Start with simple trait, add methods incrementally |
| Testing gaps | Add comprehensive unit and integration tests |
| Performance | Trait methods are zero-cost abstractions |
| Documentation | Update AGENTS.md and inline docs |

---

## Next Steps

1. **Review & Feedback**: Get stakeholder approval on trait design
2. **Implement Phase 1-2**: Create traits and first tool implementation
3. **Test**: Add unit tests for trait implementations
4. **Integrate**: Update main.rs to use new trait
5. **Verify**: Run full test suite and integration tests
6. **Document**: Update AGENTS.md with new patterns
7. **Deprecate**: Mark old functions as deprecated
8. **Cleanup**: Remove old code in next major version

---

## References

- **Template Method Pattern**: Gang of Four design pattern for defining algorithm skeleton
- **Trait Objects**: Rust's mechanism for polymorphism
- **Associated Types**: Type-safe way to parameterize traits
- **Zero-Cost Abstractions**: Rust's guarantee that traits compile to efficient code

