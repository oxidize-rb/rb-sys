# Data Directory

This directory contains manifests and derived data for rb-sys-cli.

## files

### `assets_manifest.toml`

**Unified assets manifest** for rb-sys-cli. Replaces both `phase_0_manifest.toml` and `tools.json`.

**Purpose:** Defines what assets to fetch (zig, libclang, SDKs) and how to embed them.

**Structure:**
```toml
[manifest]
version = 1

[[assets]]
name = "zig"
version = "0.15.2"
host = "x86_64-unknown-linux-gnu"
category = "tool"

# Fetch specification (used by phase_0)
fetch_type = "tarball"
fetch_url = "https://ziglang.org/download/..."
fetch_digest = "sha256:..."
strip_components = 1

# Archive path in embedded tarball (used by phase_1/runtime)
archive_path = "tools/x86_64-unknown-linux-gnu/zig.tar.zst"
```

**Categories:**
- `tool` - Build tools (zig, libclang)
- `sdk` - Platform SDKs (macOS SDK)
- `ruby` - Ruby sysroots (future)

**Used by:**
- phase_0: Reads fetch spec, downloads assets, computes BLAKE3
- phase_1: Reads archive paths, packages into embedded tarball
- Runtime: Extracts and verifies assets using BLAKE3

### `derived/`

Generated files (not checked into git):

- `phase_0_lock.toml` - Lockfile with BLAKE3 hashes computed by phase_0
- `rb-sys-cli-manifest.json` - Runtime manifest embedded into binary
- `rb-sys-cli.json` - Build metadata
- Other derived files...

### `staging/`

Temporary directory for phase_0 asset processing (not checked into git).

### Deprecated Files

- `phase_0_manifest.toml.backup` - Old manifest (replaced by assets_manifest.toml)
- `tools.json.backup` - Old tools manifest (replaced by assets_manifest.toml)

## Workflow

```
1. Edit assets_manifest.toml
      ↓
2. rake cli:prepare
      ├─ phase_0: Fetch assets → compute BLAKE3 → write phase_0_lock.toml
      └─ phase_1: Package assets → generate manifest → embed in binary
      ↓
3. cargo build -p rb-sys-cli
      ↓
4. cargo gem build --target <target>
      └─ Extracts and verifies assets using BLAKE3
```

## Adding a New Asset

1. Add entry to `assets_manifest.toml`:
   ```toml
   [[assets]]
   name = "my-tool"
   version = "1.0.0"
   host = "x86_64-unknown-linux-gnu"
   category = "tool"
   fetch_type = "tarball"
   fetch_url = "https://example.com/my-tool.tar.xz"
   fetch_digest = "sha256:..."  # Get from upstream
   archive_path = "tools/x86_64-unknown-linux-gnu/my-tool.tar.zst"
   ```

2. Run `rake cli:prepare` to fetch and package

3. Rebuild rb-sys-cli: `cargo build -p rb-sys-cli`

## Validating the Manifest

```bash
# Check manifest syntax
toml-cli check data/assets_manifest.toml

# Or just run phase_0 (validates automatically)
cd crates/rb-sys-cli/phase_0
cargo run -- --manifest ../../data/assets_manifest.toml
```
