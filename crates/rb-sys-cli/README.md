# cargo-gem

A CLI tool for easily cross-compiling Rust native gems using Zig as the C/C++ compiler.

## Features

- 🔨 **Easy cross-compilation**: Build native gems for any supported platform with a single command
- 🐳 **Docker-free**: Extract Ruby headers directly from OCI registries without Docker
- ⚡ **Zig-powered**: Uses Zig's universal C/C++ compiler for seamless cross-compilation
- 🎯 **Target-aware**: Automatically configures compiler shims for your target platform
- 📦 **Cache management**: Efficiently cache Ruby headers and libraries locally

## Installation

```bash
cd crates/rb-sys-cli
cargo install --path .
```

## Prerequisites

- **Rust**: 1.71 or later
- **Zig**: 0.11 or later ([install instructions](https://ziglang.org/download/))
- **Rustup targets**: Install targets you want to compile for:
  ```bash
  rustup target add aarch64-unknown-linux-gnu
  rustup target add x86_64-unknown-linux-musl
  # etc.
  ```

## Usage

### List Supported Targets

```bash
cargo gem list targets
```

Example output:

```
📋 Supported target platforms:

  • arm-unknown-linux-gnueabihf (arm-linux)
  • aarch64-unknown-linux-gnu (aarch64-linux)
  • x86_64-unknown-linux-gnu (x86_64-linux)
  • x86_64-unknown-linux-musl (x86_64-linux-musl)
  ...

Use: cargo gem build --target <rust-target>
```

### Build a Native Gem

```bash
cd your-gem-project
cargo gem build --target aarch64-unknown-linux-gnu
```

With glibc version targeting (for older Linux systems):

```bash
cargo gem build --target x86_64-unknown-linux-gnu --glibc 2.17
```

Full options:

```bash
cargo gem build \
  --target <RUST_TARGET> \
  --profile release \
  --features "feature1,feature2" \
  --glibc 2.17 \
  --verbose
```

### Extract Ruby Headers (Experimental)

Extract Ruby headers and libraries from rake-compiler-dock images:

```bash
cargo gem extract ghcr.io/rake-compiler/rake-compiler-dock-image:1.3.0-mri-x86_64-linux
```

List cached Ruby versions:

```bash
cargo gem list rubies
```

### Cache Management

Show cache location:

```bash
cargo gem cache path
```

Clear cache:

```bash
cargo gem cache clear
```

## How It Works

### 1. Compiler Shim Generation

When you run `cargo gem build`, the tool:

1. Creates temporary compiler shims that wrap `zig cc`, `zig c++`, and `zig ar`
2. Configures environment variables to point cargo/cc-rs to these shims
3. The shims automatically:
   - Strip the `-unknown-` vendor field from target triples (zig expects `x86_64-linux-gnu` not
     `x86_64-unknown-linux-gnu`)
   - Inject glibc version suffixes if specified (e.g., `x86_64-linux-gnu.2.17`)
   - Forward all other arguments unchanged

### 2. Target-Specific Environment

The tool sets these environment variables:

- `CC_<target>`: Points to the cc shim (e.g., `CC_aarch64_unknown_linux_gnu`)
- `CXX_<target>`: Points to the c++ shim
- `AR_<target>`: Points to the ar shim
- `CARGO_TARGET_<TARGET>_LINKER`: Points to the cc shim
- `CRATE_CC_NO_DEFAULTS=1`: Prevents cc-rs from adding host-specific flags
- `RB_SYS_CROSS_COMPILING=1`: Signals rb-sys that we're cross-compiling
- `RUBY_STATIC=true`: Prefer static linking of Ruby

**Important**: Generic `CC`, `CXX`, `AR` are NOT set to avoid affecting host builds (proc-macros, build scripts).

### 3. OCI Image Extraction

The `extract` command:

1. Connects to an OCI registry (e.g., GitHub Container Registry) anonymously
2. Pulls the image manifest
3. Streams each layer, decompressing and extracting on-the-fly
4. Filters for files matching `/usr/local/rake-compiler/rubies/<version>/include/` and
   `/usr/local/rake-compiler/rubies/<version>/lib/`
5. Rebases paths to `~/.cache/rb-sys/rubies/<version>/`

This allows extracting Ruby headers without Docker.

## Current Limitations

### Known Issues

1. **Ruby Headers Required**: Cross-compilation currently requires Ruby headers for the target platform. The `extract`
   command can help, but integration with rb-sys for using these headers is not yet complete.

2. **Nix Shell Compatibility**: If using Nix, ensure you're using rustup's Rust toolchain, not Nix's, as target
   installations differ:

   ```bash
   export PATH="$HOME/.rustup/toolchains/stable-$(rustc -Vv | grep host | cut -d' ' -f2)/bin:$PATH"
   ```

3. **Build Script Headers**: Some gems' build scripts (build.rs) invoke bindgen directly, which may not find system
   headers. This requires additional configuration.

4. **Proc-Macro Builds**: Proc-macros always build for the host platform. The tool correctly avoids using zig for these,
   but complex dependency graphs may still encounter issues.

## Architecture

### Module Structure

```
crates/rb-sys-cli/src/
├── main.rs              # CLI entry point
├── build.rs             # Build command implementation
├── shim_generator.rs    # Compiler shim generation (Unix/Windows)
├── toolchain.rs         # Target triple <-> Ruby platform mapping
└── extractor.rs         # OCI image layer extraction
```

### Design Decisions

1. **Binary Shims on Windows**: Windows uses compiled Rust executables as shims instead of batch scripts to avoid
   cmd.exe quote-stripping issues.

2. **Target-Only Environment Variables**: Only target-specific CC/CXX/AR variables are set to prevent zig from being
   used for host builds where it doesn't support all platform-specific linker flags.

3. **Streaming Extraction**: Image layers are streamed and processed in-memory to avoid downloading entire images.

## Contributing

See [AGENTS.md](../../../AGENTS.md) for development guidelines.

## License

MIT OR Apache-2.0
