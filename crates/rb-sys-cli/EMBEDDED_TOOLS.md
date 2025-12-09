# Embedded Tooling in cargo-gem

## Overview

Starting with version X.X.X, `cargo-gem` can bundle essential cross-compilation tools (Zig compiler and libclang) directly into the binary. This eliminates the need to install these tools separately and ensures consistent, reproducible builds across all environments.

## Architecture

### Tool Manifest (`data/tools.json`)

Tools are defined in a structured JSON manifest:

```json
{
  "version": 1,
  "tools": {
    "x86_64-apple-darwin": {
      "zig": {
        "version": "0.13.0",
        "blake3": "...",
        "archive_path": "tools/x86_64-apple-darwin/zig.tar.zst"
      },
      "libclang": {
        "version": "19.1.5",
        "blake3": "...",
        "archive_path": "tools/x86_64-apple-darwin/libclang.tar.zst"
      }
    }
  }
}
```

### Build-Time Packaging (Phase 1)

During `rake cli:prepare`:

1. **Phase 0**: Downloads rake-compiler-dock OCI images and extracts Ruby headers/sysroots
2. **Phase 1**: 
   - Loads `data/tools.json`
   - Packages tool archives into `src/embedded/assets.tar.zst`
   - Generates runtime manifest with tool metadata and BLAKE3 hashes
   - Embeds everything via `include_bytes!` at compile time

### Runtime Extraction

When `cargo gem build` runs:

1. **Tool Resolution**: Checks for tools matching the current host platform
2. **Lazy Extraction**: Extracts tools from embedded assets on first use
3. **BLAKE3 Verification**: Validates integrity before extraction
4. **Caching**: Stores in `~/.cache/rb-sys/cli/tools/<host>/<tool>/<version>/`
5. **Auto-wiring**: Sets environment variables (`ZIG_PATH`, `LIBCLANG_PATH`) automatically

## Tool Priority Order

### Zig

1. Explicit `--zig-path` or `ZIG_PATH` env var
2. Embedded Zig from unified assets (new)
3. Legacy bundled Zig (old `bundled-zig` feature)
4. System Zig via `which zig`

### libclang

1. Explicit `LIBCLANG_PATH` env var
2. Embedded libclang from unified assets
3. System libclang (bindgen default)

## CLI Commands

### View Embedded Tools

```bash
$ cargo gem tools
📦 Embedded tools for host: aarch64-apple-darwin

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Tool:     zig
Version:  0.13.0
BLAKE3:   abc123...
Path:     tools/aarch64-apple-darwin/zig.tar.zst
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✨ These tools will be automatically extracted and used during builds.
   Cache location: /Users/user/.cache/rb-sys/cli/tools
```

### Clear Tool Cache

```bash
$ cargo gem cache clear
```

### Check Cache Location

```bash
$ cargo gem cache path
/Users/user/.cache/rb-sys/cli
```

## Security

### BLAKE3 Verification

Every tool archive is verified using BLAKE3 hashing:

- **Build-time**: Hashes computed when packaging `assets.tar.zst`
- **Runtime**: Hashes verified before extraction from embedded assets
- **Type-safe**: `Blake3Hash` newtype enforces 64-char hex format

Failed verification causes build failure with clear error message showing expected vs actual hash.

## Supported Platforms

Tools are embedded for the following host platforms:

| Host Platform              | Zig | libclang |
|----------------------------|-----|----------|
| `x86_64-apple-darwin`      | ✅  | ✅       |
| `aarch64-apple-darwin`     | ✅  | ✅       |
| `x86_64-unknown-linux-gnu` | ✅  | ✅       |
| `aarch64-unknown-linux-gnu`| ✅  | ✅       |
| `x86_64-pc-windows-gnu`    | ✅  | ✅       |

## Adding New Tools

1. **Obtain Tool Archive**: Download or build the tool as a `.tar.zst` file
2. **Compute BLAKE3**: 
   ```bash
   b3sum tool.tar.zst
   ```
3. **Update `data/tools.json`**: Add entry with hash
4. **Place Archive**: Copy to expected location for phase_1 packaging
5. **Run Build**: `rake cli:prepare` to regenerate embedded assets

## Environment Variables

### Build-time

- `RB_SYS_BUILD_CACHE_DIR`: Override phase_0 cache location

### Runtime

- `RB_SYS_RUNTIME_CACHE_DIR`: Override tool extraction cache
- `ZIG_PATH`: Explicit Zig path (bypasses embedded)
- `LIBCLANG_PATH`: Explicit libclang path (bypasses embedded)

## Changes from Previous Versions

The previous approach used per-platform `#[cfg]` directives and a `bundled-zig` feature flag. The new unified assets approach provides:

- ✅ Single binary for all platforms (no more per-platform builds)
- ✅ Integrity verification with BLAKE3
- ✅ Unified tooling (Zig + libclang together)
- ✅ Extensible for future tools
- ✅ Better caching and version management

The legacy `bundled-zig` feature has been removed in favor of the unified approach.

## File Layout

```
rb-sys/
├── data/
│   └── tools.json              # Tool manifest (source of truth)
├── crates/rb-sys-cli/
│   ├── phase_1/
│   │   └── src/tools.rs        # Tool manifest loader
│   ├── src/
│   │   ├── blake3_hash.rs      # BLAKE3 hash type
│   │   ├── tools.rs            # Tool extraction helpers
│   │   ├── libclang.rs         # libclang configuration
│   │   ├── assets/
│   │   │   └── manifest.rs     # Runtime ToolInfo struct
│   │   ├── zig/
│   │   │   └── manager.rs      # Zig path resolution
│   │   └── embedded/
│   │       ├── assets.tar.zst  # Embedded tools + sysroots (generated)
│   │       └── manifest.json   # Runtime manifest (generated)
```
