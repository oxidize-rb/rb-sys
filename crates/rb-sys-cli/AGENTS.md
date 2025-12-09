# Agent Instructions for rb-sys-cli

## Build/Test/Lint Commands

- **Build preparation**: `rake data:derive && rake cli:prepare` (required before first build)
- **Build CLI**: `./script/run cargo build -p rb-sys-cli`
- **Run all tests**: `./script/run cargo test -p rb-sys-cli`
- **Run single test**: `./script/run cargo test -p rb-sys-cli test_name`
- **Lint**: `./script/run cargo fmt --check && cargo clippy -p rb-sys-cli`
- **Format**: `./script/run cargo fmt`

## Code Style

- **Rust**: Edition 2021, MSRV 1.71. Use default `rustfmt` and `clippy` settings.
- **Imports**: Group std, external crates, then local modules. Separate groups with blank lines.
- **Naming**: snake_case for functions/variables, CamelCase for types/structs.
- **Error handling**: Use `anyhow::Result` for CLI errors. Propagate with `?`, add context with `.context()`.
- **CLI structure**: Use `clap` derive macros for argument parsing.
- **Logging**: Use `tracing` macros (`trace!`, `debug!`, `info!`, `warn!`, `error!`).
- **Tests**: Use `assert_cmd` and `predicates` for CLI integration tests.

## Crate Structure

- `src/main.rs` - CLI entry point and command routing
- `src/build.rs` - Build command implementation
- `src/zig/` - Zig compiler shim generation and argument handling
- `src/platform/` - Platform-specific implementations (linux, macos, windows)
- `src/assets/` - Embedded asset management
- `src/sysroot.rs` - Hermetic sysroot extraction and management
- `src/toolchain.rs` - Rust target to Ruby platform mapping
- `phase_0/`, `phase_1/` - Build-time codegen phases (OCI extraction, asset packaging)

## Architecture Overview

### Phase-Based Build System

The CLI uses a multi-phase build system to embed cross-compilation assets:

```
BUILD-TIME (rake cli:prepare)
==============================

  phase_0          phase_1           Embedded in binary
  (OCI pull)  -->  (codegen)   -->   src/embedded/assets.tar.zst
                                     src/embedded/manifest.json
      |                |
      v                v
  ~/.cache/rb-sys/cli  data/derived/
  (extracted rubies,   (generated Rust,
   sysroots)           toolchain mappings)


RUNTIME (cargo gem build)
=========================

  AssetManager  -->  SysrootManager  -->  Zig Shims  -->  cargo build
  (lazy extract)     (mount/unmount)      (cc,c++,       --target
                                           ar,ld)
```

### Key Components

#### 1. Build Phases (build-time only)

**phase_0** (`phase_0/`): OCI Image Extraction

- Downloads rake-compiler-dock images from OCI registries (no Docker required)
- Extracts Ruby headers, libraries, and sysroot files
- Stores in `~/.cache/rb-sys/cli/<ruby-platform>/`
- Uses `oci-distribution` crate for registry access
- Shows progress bars via `tracing-indicatif`

**phase_1** (`phase_1/`): Codegen & Packaging

- Generates Rust toolchain mappings from `data/toolchains.json`
- Creates runtime manifest (`manifest.json`)
- Packages all assets into `src/embedded/assets.tar.zst`
- Assets are embedded in the final binary via `include_bytes!`

#### 2. Asset Management (`src/assets/`)

**AssetManager**: Manages embedded assets and lazy extraction

- Embeds `assets.tar.zst` and `manifest.json` at compile time
- Lazily extracts platform-specific files on first use
- Caches extracted files in `~/.cache/rb-sys/cli/` (runtime cache)
- Tracks extraction state with marker files

**Manifest**: Maps Rust targets to Ruby platforms

- Stores metadata: ruby versions, sysroot availability, OCI digests
- Provides lookup by Rust target or Ruby platform

#### 3. Sysroot Management (`src/sysroot.rs`)

**SysrootManager**: Handles hermetic build environments

- Extracts sysroot to `target/rb-sys/<target>/sysroot/` (build-local)
- Contains headers (OpenSSL, zlib) and libraries for the target
- Ruby headers stay in runtime cache (shared across builds)
- Cleans up on successful build, keeps on failure for debugging

**MountedSysroot**: RAII guard for sysroot lifecycle

- `path()` - sysroot location (libs, OpenSSL headers)
- `rubies_path()` - Ruby headers location
- Auto-cleanup on drop (unless `.keep()` called)

#### 4. Zig Cross-Compilation (`src/zig/`)

```
src/zig/
├── mod.rs      # Module exports
├── target.rs   # RustTarget parsing, Zig target translation
├── shim.rs     # Bash shim generation (cc, c++, ar, ld, dlltool)
├── cc.rs       # C/C++ compiler argument filtering
├── ar.rs       # Archiver argument handling
├── ld.rs       # Linker argument handling
├── dlltool.rs  # Windows import library generation
├── args.rs     # Common argument processing utilities
├── env.rs      # Cargo environment variable setup
├── cpu.rs      # CPU feature detection
└── libc.rs     # Zig libc include path discovery
```

**RustTarget** (`target.rs`): Target triple handling

- Parses Rust target triples (e.g., `x86_64-unknown-linux-gnu`)
- Converts to Zig format (e.g., `x86_64-linux-gnu.2.17`)
- Supports 9 targets: Linux (glibc/musl), macOS, Windows

**Shim Generation** (`shim.rs`): Creates wrapper scripts

- Generates bash scripts that call back to `cargo-gem zig-cc/zig-cxx/zig-ar/zig-ld`
- Shims live in `target/rb-sys/<target>/bin/`
- Allows argument filtering and target-specific transformations

**Argument Filtering** (`cc.rs`, `ar.rs`, `ld.rs`): Cleans compiler args

- Strips host-specific flags that break cross-compilation
- Rewrites paths to use sysroot
- Handles Zig-incompatible flags (e.g., `-Wl,--as-needed`)

#### 5. Toolchain Info (`src/toolchain.rs`)

Maps between Rust targets and Ruby platforms:

- Loads from `data/toolchains.json`
- Provides lookup by Rust target or Ruby platform
- Used to find correct sysroot and Ruby headers

#### 6. Build Command (`src/build.rs`)

Orchestrates the cross-compilation:

1. Parse and validate target
2. Validate Zig installation
3. Mount sysroot (extract from embedded assets)
4. Generate compiler shims
5. Set up environment variables (CC, CXX, AR, BINDGEN_EXTRA_CLANG_ARGS, etc.)
6. Run `cargo build --target <target>`

### Supported Targets

| Rust Target                   | Ruby Platform        | Zig Target                 |
| ----------------------------- | -------------------- | -------------------------- |
| `arm-unknown-linux-gnueabihf` | `arm-linux`          | `arm-linux-gnueabihf.2.17` |
| `aarch64-unknown-linux-gnu`   | `aarch64-linux`      | `aarch64-linux-gnu.2.17`   |
| `aarch64-unknown-linux-musl`  | `aarch64-linux-musl` | `aarch64-linux-musl`       |
| `x86_64-unknown-linux-gnu`    | `x86_64-linux`       | `x86_64-linux-gnu.2.17`    |
| `x86_64-unknown-linux-musl`   | `x86_64-linux-musl`  | `x86_64-linux-musl`        |
| `aarch64-apple-darwin`        | `arm64-darwin`       | `aarch64-macos-none`       |
| `x86_64-apple-darwin`         | `x86_64-darwin`      | `x86_64-macos-none`        |
| `x86_64-pc-windows-gnu`       | `x64-mingw-ucrt`     | `x86_64-windows-gnu`       |
| `aarch64-pc-windows-gnullvm`  | `arm64-mingw-ucrt`   | `aarch64-windows-gnu`      |

### Data Flow

```
User runs: cargo gem build --target x86_64-unknown-linux-gnu

1. Parse target -> RustTarget { arch: X86_64, os: Linux, env: Gnu }
2. Load toolchain info -> ruby_platform: "x86_64-linux"
3. Mount sysroot:
   - Extract from embedded assets.tar.zst
   - Place in target/rb-sys/x86_64-unknown-linux-gnu/sysroot/
4. Generate shims in target/rb-sys/x86_64-unknown-linux-gnu/bin/:
   - cc  -> calls `cargo-gem zig-cc --target ... -- $@`
   - c++ -> calls `cargo-gem zig-cxx --target ... -- $@`
   - ar  -> calls `cargo-gem zig-ar -- $@`
   - ld  -> calls `cargo-gem zig-ld --target ... -- $@`
5. Set environment:
   - CC_x86_64_unknown_linux_gnu=/path/to/shim/cc
   - CXX_x86_64_unknown_linux_gnu=/path/to/shim/c++
   - AR_x86_64_unknown_linux_gnu=/path/to/shim/ar
   - CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/path/to/shim/cc
   - BINDGEN_EXTRA_CLANG_ARGS="-I/zig/libc/include -I/sysroot/usr/include"
   - RBCONFIG_rubyhdrdir=/path/to/ruby/headers
6. Run: cargo build --target x86_64-unknown-linux-gnu
7. Cleanup sysroot (on success)
```

### Environment Variables

**Build-time (phase_0/phase_1)**:

- `RB_SYS_BUILD_CACHE_DIR` - Override build cache location
- `RB_SYS_SKIP_EXTRACTION` - Skip OCI extraction (testing)

**Runtime**:

- `RB_SYS_RUNTIME_CACHE_DIR` - Override runtime cache location
- `SDKROOT` - macOS SDK path (required for Darwin targets)
- `ZIG_PATH` - Path to Zig compiler (default: `zig`)
- `RUST_LOG` - Tracing log level (e.g., `rb_sys_cli=debug`)

**Set by CLI during build**:

- `CC_<target>`, `CXX_<target>`, `AR_<target>` - Compiler shim paths
- `CARGO_TARGET_<TARGET>_LINKER` - Linker shim path
- `CRATE_CC_NO_DEFAULTS=1` - Prevent cc-rs from adding host flags
- `RB_SYS_CROSS_COMPILING=1` - Signal cross-compilation mode
- `BINDGEN_EXTRA_CLANG_ARGS` - Include paths for bindgen
- `PKG_CONFIG_PATH` - Sysroot pkg-config directory
- `RBCONFIG_rubyhdrdir`, `RBCONFIG_rubyarchhdrdir` - Ruby header paths

### Testing

Unit tests are inline in each module (run with `cargo test -p rb-sys-cli`):

- `src/zig/target.rs` - Target parsing and Zig conversion tests
- `src/zig/shim.rs` - Shim generation tests
- `src/build.rs` - Build config tests
- `src/toolchain.rs` - Toolchain lookup tests
- `src/main.rs` - CLI argument validation
