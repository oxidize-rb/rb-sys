# ZigShim Trait Design - Complete Documentation

## Overview

This directory contains comprehensive documentation for implementing a trait-based abstraction for Zig toolchain wrappers in the rb-sys project.

## Documents

### 1. **QUICK_REFERENCE.md** ⚡
Start here for a quick overview of the problem, solution, and implementation pattern.
- Problem statement
- Core traits
- Implementation pattern
- File structure
- Benefits and checklist

### 2. **IMPLEMENTATION_GUIDE.md** 📚
Complete implementation guide with detailed code examples and migration strategy.
- Current architecture analysis
- Trait design with full code
- 6-phase implementation plan
- Benefits and migration strategy
- Testing strategy
- Risk mitigation

### 3. **MANIFEST.json** 📋
Metadata about the artifacts and project structure.

## Key Insights

### Problem
The rb-sys codebase has 4 separate Zig tool wrappers (cc, ld, ar, dlltool) with ~200 lines of duplicated execution logic following the same pattern:

```
validate → build command → add platform flags → filter args → execute
```

### Solution
Introduce `ZigShim` and `ShimArgs` traits using the template method pattern to:
- Eliminate code duplication
- Provide consistent execution lifecycle
- Enable easy extension for new tools
- Maintain type safety with associated types

### Core Traits

```rust
pub trait ShimArgs {
    fn zig_path(&self) -> &PathBuf;
    fn user_args(&self) -> &[String];
    fn target(&self) -> Option<&str> { None }
    fn sysroot(&self) -> Option<&PathBuf> { None }
}

pub trait ZigShim: Sized {
    type Args: ShimArgs;
    fn subcommand(&self) -> &str;
    fn target(&self) -> Option<&RustTarget> { None }
    fn validate(&self, _args: &Self::Args) -> Result<()> { Ok(()) }
    fn add_platform_flags(&self, _cmd: &mut Command, _args: &Self::Args) -> Result<()> { Ok(()) }
    fn filter_args(&self, args: &[String]) -> Vec<String>;
    fn run(&self, args: Self::Args) -> Result<()> { ... }  // Template method
}
```

## Implementation Phases

1. **Create Trait Definitions** - `tool.rs` with `ZigShim` and `ShimArgs`
2. **Implement Traits for Each Tool** - `tools/{cc,ld,ar,dlltool}.rs`
3. **Create Tools Module** - `tools/mod.rs` with exports
4. **Update Module Structure** - `zig/mod.rs` with new exports
5. **Update main.rs** - Use trait implementations in command handlers
6. **Deprecate Old Code** - Mark old functions as deprecated

## Benefits

| Benefit | Impact |
|---------|--------|
| **DRY** | Eliminates ~200 lines of duplicated code |
| **Extensibility** | Adding new tools requires only trait impl |
| **Clarity** | Tool-specific behavior isolated in impl blocks |
| **Consistency** | All tools follow same lifecycle pattern |
| **Testability** | Can mock trait for testing template method |
| **Type Safety** | Associated types ensure args match tool |

## Estimated Effort

- **Phase 1-2** (Traits + First Tool): 4-6 hours
- **Phase 3-4** (Remaining Tools + Integration): 4-6 hours
- **Phase 5-6** (Testing + Documentation): 4-6 hours
- **Total**: 2-3 days for full implementation and testing

## Current File Locations

- **Trait definitions**: `crates/rb-sys-cli/src/zig/tool.rs` (NEW)
- **Tool implementations**: `crates/rb-sys-cli/src/zig/tools/` (NEW)
- **Existing tools**: `crates/rb-sys-cli/src/zig/{cc,ld,ar,dlltool}.rs`
- **Main entry point**: `crates/rb-sys-cli/src/main.rs`

## Design Patterns

1. **Template Method Pattern** - `run()` defines algorithm skeleton
2. **Strategy Pattern** - Tool-specific behavior via trait methods
3. **Associated Types** - Type-safe parameterization of traits
4. **Zero-Cost Abstractions** - Traits compile to efficient code

## Next Steps

1. Review QUICK_REFERENCE.md for overview
2. Review IMPLEMENTATION_GUIDE.md for detailed plan
3. Get stakeholder approval on trait design
4. Implement Phase 1-2 (traits + first tool)
5. Add comprehensive unit tests
6. Update main.rs to use new trait
7. Run full test suite and integration tests
8. Update AGENTS.md documentation
9. Mark old functions as deprecated
10. Plan removal in next major version

## References

- **Rust Traits**: https://doc.rust-lang.org/book/ch17-00-oop.html
- **Template Method Pattern**: Gang of Four Design Patterns
- **Associated Types**: https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
- **Zero-Cost Abstractions**: https://doc.rust-lang.org/book/ch19-00-advanced-features.html

---

**Created**: 2025-12-09  
**Status**: Design Phase  
**Effort**: 2-3 days for full implementation  
**Risk Level**: Low (with proper testing and migration strategy)
