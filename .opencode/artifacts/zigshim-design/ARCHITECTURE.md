# ZigShim Trait Design - Architecture Diagram

## Current Architecture (Before)

```
┌─────────────────────────────────────────────────────────────────┐
│                         main.rs                                 │
│  Commands::ZigCc(args) → zig::cc::run_cc(args, false)          │
│  Commands::ZigCxx(args) → zig::cc::run_cc(args, true)          │
│  Commands::ZigAr(args) → zig::ar::run_ar(args)                 │
│  Commands::ZigLd(args) → zig::ld::run_ld(args)                 │
│  Commands::ZigDlltool(args) → zig::dlltool::run_dlltool(args)  │
└─────────────────────────────────────────────────────────────────┘
                              ↓
        ┌─────────────────────┼─────────────────────┐
        ↓                     ↓                     ↓
    ┌────────┐           ┌────────┐           ┌────────┐
    │ cc.rs  │           │ ld.rs  │           │ ar.rs  │
    │        │           │        │           │        │
    │ run_cc │           │ run_ld │           │ run_ar │
    │ (221L) │           │ (310L) │           │ (87L)  │
    └────────┘           └────────┘           └────────┘
        ↓                     ↓                     ↓
    ┌────────────────────────────────────────────────────┐
    │  DUPLICATED LOGIC (200+ lines)                     │
    │  1. Parse target                                   │
    │  2. Validate requirements                          │
    │  3. Build base command                             │
    │  4. Add platform-specific flags                    │
    │  5. Filter user arguments                          │
    │  6. Execute and handle exit status                 │
    └────────────────────────────────────────────────────┘
        ↓
    ┌────────────────────────────────────────────────────┐
    │  args.rs (32K lines)                               │
    │  - ArgFilter                                       │
    │  - filter_cc_args()                                │
    │  - filter_link_args()                              │
    │  - filter_ar_args()                                │
    └────────────────────────────────────────────────────┘
```

## Proposed Architecture (After)

```
┌─────────────────────────────────────────────────────────────────┐
│                         main.rs                                 │
│  Commands::ZigCc(args) → {                                      │
│    let target = RustTarget::parse(&args.target)?;              │
│    let shim = ZigCc { target, is_cxx: false };                 │
│    shim.run(args)?;                                             │
│  }                                                               │
└─────────────────────────────────────────────────────────────────┘
                              ↓
        ┌─────────────────────┼─────────────────────┐
        ↓                     ↓                     ↓
    ┌────────────┐       ┌────────────┐       ┌────────────┐
    │ ZigCc      │       │ ZigLd      │       │ ZigAr      │
    │ (struct)   │       │ (struct)   │       │ (struct)   │
    └────────────┘       └────────────┘       └────────────┘
        ↓                     ↓                     ↓
    ┌────────────────────────────────────────────────────┐
    │              ZigShim Trait                         │
    │  ┌──────────────────────────────────────────────┐  │
    │  │ fn run(&self, args: Self::Args) -> Result   │  │
    │  │ {                                            │  │
    │  │   1. validate(&args)?                        │  │
    │  │   2. build_command(&args)                    │  │
    │  │   3. add_platform_flags(&mut cmd, &args)?   │  │
    │  │   4. filter_args(&args.user_args())         │  │
    │  │   5. execute and handle exit status         │  │
    │  │ }                                            │  │
    │  └──────────────────────────────────────────────┘  │
    │                                                     │
    │  Customization Points (impl by each tool):         │
    │  - subcommand() → &str                             │
    │  - target() → Option<&RustTarget>                  │
    │  - validate(&args) → Result<()>                    │
    │  - add_platform_flags(&mut cmd, &args) → Result   │
    │  - filter_args(&[String]) → Vec<String>           │
    └────────────────────────────────────────────────────┘
        ↓
    ┌────────────────────────────────────────────────────┐
    │              ShimArgs Trait                        │
    │  - zig_path() → &PathBuf                           │
    │  - user_args() → &[String]                         │
    │  - target() → Option<&str>                         │
    │  - sysroot() → Option<&PathBuf>                    │
    └────────────────────────────────────────────────────┘
        ↓
    ┌────────────────────────────────────────────────────┐
    │  args.rs (unchanged)                               │
    │  - ArgFilter                                       │
    │  - filter_cc_args()                                │
    │  - filter_link_args()                              │
    │  - filter_ar_args()                                │
    └────────────────────────────────────────────────────┘
```

## Module Structure

### Before
```
crates/rb-sys-cli/src/zig/
├── cc.rs          (221 lines - run_cc function)
├── ld.rs          (310 lines - run_ld function)
├── ar.rs          (87 lines - run_ar function)
├── dlltool.rs     (TBD - run_dlltool function)
├── args.rs        (32K lines - argument filtering)
├── target.rs      (unchanged)
├── cpu.rs         (unchanged)
├── env.rs         (unchanged)
├── libc.rs        (unchanged)
├── manager.rs     (unchanged)
├── shim.rs        (unchanged - bash shim generation)
└── mod.rs         (updated exports)
```

### After
```
crates/rb-sys-cli/src/zig/
├── tool.rs                    (NEW - trait definitions)
├── tools/                     (NEW - tool implementations)
│   ├── mod.rs                 (exports)
│   ├── cc.rs                  (ZigCc impl)
│   ├── ld.rs                  (ZigLd impl)
│   ├── ar.rs                  (ZigAr impl)
│   └── dlltool.rs             (ZigDlltool impl)
├── cc.rs                      (deprecated wrapper)
├── ld.rs                      (deprecated wrapper)
├── ar.rs                      (deprecated wrapper)
├── dlltool.rs                 (deprecated wrapper)
├── args.rs                    (unchanged)
├── target.rs                  (unchanged)
├── cpu.rs                     (unchanged)
├── env.rs                     (unchanged)
├── libc.rs                    (unchanged)
├── manager.rs                 (unchanged)
├── shim.rs                    (unchanged)
└── mod.rs                     (updated exports)
```

## Trait Hierarchy

```
┌─────────────────────────────────────────────────────────────┐
│                    ShimArgs Trait                           │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ fn zig_path(&self) -> &PathBuf                       │   │
│  │ fn user_args(&self) -> &[String]                     │   │
│  │ fn target(&self) -> Option<&str> { None }           │   │
│  │ fn sysroot(&self) -> Option<&PathBuf> { None }      │   │
│  └──────────────────────────────────────────────────────┘   │
│                          ↑                                   │
│         ┌────────────────┼────────────────┐                 │
│         ↑                ↑                ↑                 │
│    ZigCcArgs        ZigLdArgs         ZigArArgs             │
│    (impl)           (impl)            (impl)                │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                    ZigShim Trait                            │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ type Args: ShimArgs                                  │   │
│  │ fn subcommand(&self) -> &str                         │   │
│  │ fn target(&self) -> Option<&RustTarget> { None }    │   │
│  │ fn validate(&self, &Self::Args) -> Result { Ok(()) }│   │
│  │ fn add_platform_flags(&self, &mut Cmd, &Self::Args) │   │
│  │   -> Result { Ok(()) }                               │   │
│  │ fn filter_args(&self, &[String]) -> Vec<String>     │   │
│  │ fn build_command(&self, &Self::Args) -> Command     │   │
│  │ fn run(&self, Self::Args) -> Result { ... }         │   │
│  └──────────────────────────────────────────────────────┘   │
│                          ↑                                   │
│         ┌────────────────┼────────────────┐                 │
│         ↑                ↑                ↑                 │
│       ZigCc            ZigLd            ZigAr               │
│      (impl)           (impl)           (impl)               │
└─────────────────────────────────────────────────────────────┘
```

## Execution Flow

### Template Method Pattern

```
┌─────────────────────────────────────────────────────────────┐
│                    shim.run(args)                           │
│                  (Template Method)                          │
└─────────────────────────────────────────────────────────────┘
                          ↓
        ┌─────────────────────────────────────┐
        │ 1. VALIDATE                         │
        │    self.validate(&args)?            │
        │    (Tool-specific validation)       │
        └─────────────────────────────────────┘
                          ↓
        ┌─────────────────────────────────────┐
        │ 2. BUILD COMMAND                    │
        │    let mut cmd = self.build_command │
        │    (Default: zig + subcommand)      │
        └─────────────────────────────────────┘
                          ↓
        ┌─────────────────────────────────────┐
        │ 3. ADD PLATFORM FLAGS               │
        │    self.add_platform_flags(         │
        │      &mut cmd, &args)?              │
        │    (Tool-specific flags)            │
        └─────────────────────────────────────┘
                          ↓
        ┌─────────────────────────────────────┐
        │ 4. FILTER ARGUMENTS                 │
        │    let filtered =                   │
        │      self.filter_args(user_args)    │
        │    (Tool-specific filtering)        │
        └─────────────────────────────────────┘
                          ↓
        ┌─────────────────────────────────────┐
        │ 5. EXECUTE                          │
        │    cmd.status()?                    │
        │    (Common execution logic)         │
        └─────────────────────────────────────┘
                          ↓
                    Result<()>
```

## Code Reduction

### Before (Duplicated)
```
cc.rs:      221 lines (run_cc function)
ld.rs:      310 lines (run_ld function)
ar.rs:       87 lines (run_ar function)
dlltool.rs:  TBD lines (run_dlltool function)
────────────────────────
Total:      ~600+ lines of similar logic
```

### After (Unified)
```
tool.rs:    ~100 lines (trait definitions + template method)
tools/cc.rs: ~80 lines (ZigCc impl + ShimArgs impl)
tools/ld.rs: ~100 lines (ZigLd impl + ShimArgs impl)
tools/ar.rs: ~30 lines (ZigAr impl + ShimArgs impl)
tools/dlltool.rs: ~30 lines (ZigDlltool impl + ShimArgs impl)
────────────────────────
Total:      ~340 lines (40% reduction)
```

## Customization Points

Each tool customizes these methods:

```
ZigCc:
  ✓ subcommand() → "cc" or "c++"
  ✓ target() → Some(&self.target)
  ✓ validate() → Check sysroot/SDKROOT
  ✓ add_platform_flags() → Add -target, -mcpu, -g, platform-specific
  ✓ filter_args() → Filter CC arguments

ZigLd:
  ✓ subcommand() → "ld.lld", "ld64.lld", or "cc"
  ✓ target() → Some(&self.target)
  ✓ validate() → Check sysroot
  ✓ add_platform_flags() → Add -m, --sysroot, -syslibroot
  ✓ filter_args() → Filter linker arguments

ZigAr:
  ✓ subcommand() → "ar"
  ✓ target() → None (not needed)
  ✓ validate() → (default: no validation)
  ✓ add_platform_flags() → (default: no flags)
  ✓ filter_args() → Filter AR arguments

ZigDlltool:
  ✓ subcommand() → "dlltool"
  ✓ target() → Some(&self.target)
  ✓ validate() → (default: no validation)
  ✓ add_platform_flags() → (default: no flags)
  ✓ filter_args() → (default: no filtering)
```

## Benefits Summary

| Aspect | Before | After |
|--------|--------|-------|
| **Code Duplication** | ~200 lines | Eliminated |
| **Lines of Code** | ~600+ | ~340 |
| **Consistency** | Varies | Guaranteed |
| **Extensibility** | Hard | Easy |
| **Testability** | Difficult | Simple |
| **Type Safety** | Basic | Enhanced |
| **Maintenance** | High | Low |

---

**Design Pattern**: Template Method + Strategy + Associated Types  
**Complexity**: Low (simple trait with default implementations)  
**Risk**: Low (backward compatible with wrappers)  
**Performance**: Zero-cost (traits compile to efficient code)
