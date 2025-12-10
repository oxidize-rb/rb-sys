# ZigShim Trait Design - Quick Reference

## Problem Statement

The rb-sys codebase has 4 separate Zig tool wrappers (cc, ld, ar, dlltool) with ~200 lines of duplicated execution logic:

```
validate → build command → add platform flags → filter args → execute
```

This pattern is repeated in each tool with minor variations.

## Solution: Trait-Based Abstraction

### Core Traits

```rust
// Abstracts over different argument types
pub trait ShimArgs {
    fn zig_path(&self) -> &PathBuf;
    fn user_args(&self) -> &[String];
    fn target(&self) -> Option<&str> { None }
    fn sysroot(&self) -> Option<&PathBuf> { None }
}

// Template method pattern for tool execution
pub trait ZigShim: Sized {
    type Args: ShimArgs;
    
    fn subcommand(&self) -> &str;
    fn target(&self) -> Option<&RustTarget> { None }
    fn validate(&self, _args: &Self::Args) -> Result<()> { Ok(()) }
    fn add_platform_flags(&self, _cmd: &mut Command, _args: &Self::Args) -> Result<()> { Ok(()) }
    fn filter_args(&self, args: &[String]) -> Vec<String>;
    
    // Template method - orchestrates the entire lifecycle
    fn run(&self, args: Self::Args) -> Result<()> { ... }
}
```

## Implementation Pattern

### For Each Tool

```rust
// 1. Define the tool struct
pub struct ZigCc {
    pub target: RustTarget,
    pub is_cxx: bool,
}

// 2. Implement ZigShim trait
impl ZigShim for ZigCc {
    type Args = ZigCcArgs;
    
    fn subcommand(&self) -> &str {
        if self.is_cxx { "c++" } else { "cc" }
    }
    
    fn validate(&self, args: &ZigCcArgs) -> Result<()> {
        // Tool-specific validation
    }
    
    fn add_platform_flags(&self, cmd: &mut Command, args: &ZigCcArgs) -> Result<()> {
        // Tool-specific platform flags
    }
    
    fn filter_args(&self, args: &[String]) -> Vec<String> {
        // Tool-specific argument filtering
    }
}

// 3. Implement ShimArgs for the args type
impl ShimArgs for ZigCcArgs {
    fn zig_path(&self) -> &PathBuf { &self.zig_path }
    fn user_args(&self) -> &[String] { &self.args }
    fn target(&self) -> Option<&str> { Some(&self.target) }
    fn sysroot(&self) -> Option<&PathBuf> { self.sysroot.as_ref() }
}
```

## Usage in main.rs

```rust
// Before (duplicated logic in each handler)
Commands::ZigCc(args) => {
    zig::cc::run_cc(args, false)?;
}

// After (unified trait-based approach)
Commands::ZigCc(args) => {
    let target = zig::target::RustTarget::parse(&args.target)?;
    let shim = zig::tools::ZigCc { target, is_cxx: false };
    shim.run(args)?;
}
```

## File Structure

```
crates/rb-sys-cli/src/zig/
├── tool.rs              (NEW - trait definitions)
├── tools/               (NEW - tool implementations)
│   ├── mod.rs
│   ├── cc.rs
│   ├── ld.rs
│   ├── ar.rs
│   └── dlltool.rs
├── cc.rs                (keep as wrapper or deprecate)
├── ld.rs                (keep as wrapper or deprecate)
├── ar.rs                (keep as wrapper or deprecate)
└── dlltool.rs           (keep as wrapper or deprecate)
```

## Key Benefits

| Benefit | Impact |
|---------|--------|
| **DRY** | Eliminates ~200 lines of duplicated code |
| **Extensibility** | Adding new tools requires only trait impl |
| **Clarity** | Tool-specific behavior isolated in impl blocks |
| **Consistency** | All tools follow same lifecycle pattern |
| **Testability** | Can mock trait for testing template method |
| **Type Safety** | Associated types ensure args match tool |

## Implementation Checklist

- [ ] Create `tool.rs` with trait definitions
- [ ] Create `tools/` module structure
- [ ] Implement `ZigCc` in `tools/cc.rs`
- [ ] Implement `ZigLd` in `tools/ld.rs`
- [ ] Implement `ZigAr` in `tools/ar.rs`
- [ ] Implement `ZigDlltool` in `tools/dlltool.rs`
- [ ] Update `zig/mod.rs` to export new types
- [ ] Update `main.rs` to use trait implementations
- [ ] Add unit tests for each trait impl
- [ ] Run full test suite
- [ ] Update AGENTS.md documentation
- [ ] Mark old functions as deprecated
- [ ] Plan removal in next major version

## Estimated Effort

- **Phase 1-2** (Traits + First Tool): 4-6 hours
- **Phase 3-4** (Remaining Tools + Integration): 4-6 hours
- **Phase 5-6** (Testing + Documentation): 4-6 hours
- **Total**: 2-3 days for full implementation and testing

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Breaking changes | Keep old functions as wrappers during transition |
| Testing gaps | Add comprehensive unit and integration tests |
| Performance | Trait methods are zero-cost abstractions |
| Documentation | Update AGENTS.md and inline docs |

## Design Patterns Used

1. **Template Method Pattern**: `run()` method defines algorithm skeleton
2. **Strategy Pattern**: Tool-specific behavior via trait methods
3. **Associated Types**: Type-safe parameterization of traits
4. **Zero-Cost Abstractions**: Traits compile to efficient code

## References

- Full implementation guide: `IMPLEMENTATION_GUIDE.md`
- Current code: `crates/rb-sys-cli/src/zig/{cc,ld,ar,dlltool}.rs`
- Trait documentation: Rust Book Chapter 17 (Traits)
- Template Method: Gang of Four Design Patterns

