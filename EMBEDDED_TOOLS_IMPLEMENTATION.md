# Embedded Assets Implementation Summary

## Overview

This document describes the embedded asset system in rb-sys, which supports two main use cases:
1. **Pre-generated Ruby bindings** for cross-compilation (rb-sys-build)
2. **Embedded tooling** (Zig, libclang) for cargo-gem (planned/partial)

## 1. Pre-Generated Bindings (rb-sys-build)

### What Was Built

A system for embedding pre-generated Ruby C API bindings to enable cross-compilation without requiring bindgen/libclang on the target system.

### Key Features

✅ **Embedded Bindings**: 282KB compressed tarball with bindings for 19 platform/version combinations
✅ **Extract-Once Caching**: Single decompression per build to `$OUT_DIR/rb-sys-embedded-bindings/`
✅ **Hash-Based Invalidation**: Fast fingerprint (length + first/last 8 bytes) detects tarball changes
✅ **Feature-Aware Filtering**: Removes items based on enabled features (rbimpls, deprecated-types)
✅ **Automatic Fallback**: Gracefully falls back to bindgen if no embedded bindings available
✅ **Zero Global State**: Uses `OUT_DIR` during builds, temp dir for tests

### Files

**Core Implementation:**
- `crates/rb-sys-build/src/pregenerated.rs` - Main extraction and processing logic
- `crates/rb-sys-build/src/bindings.rs` - Unified generate_or_load() entry point
- `crates/rb-sys-build/data/assets.tar.zst` - Embedded bindings archive (282KB → 12.7MB)

**Build Support:**
- `crates/rb-sys-cli/phase_1/src/bindings.rs` - Bindings generation for phase_1
- `crates/rb-sys-cli/phase_1/src/assets.rs` - Asset packaging

### Data Flow

#### Build Time (rb-sys-build)

```
include_bytes!("data/assets.tar.zst")
     ↓
First access (has_embedded_bindings or load_embedded)
     ↓
ensure_extracted() checks $OUT_DIR/rb-sys-embedded-bindings/.extracted
     ↓
If missing or hash mismatch:
  - Extract entire tarball to disk (single 12.7MB decompression)
  - Write hash marker
     ↓
Subsequent accesses:
  - build_embedded_index() walks extracted directory (fast)
  - extract_embedded_bindings() reads files directly (fast)
```

#### Runtime (consuming crate build.rs)

```
RbConfig::current()
     ↓
bindings::generate_or_load(rbconfig, ...)
     ↓
Priority order:
  1. RB_SYS_PREGENERATED_BINDINGS_PATH env var
  2. Embedded bindings (if has_embedded_bindings())
  3. Bindgen (fallback)
     ↓
load_embedded():
  - extract_embedded_bindings(platform, version)
  - syn::parse_file() to AST
  - filter_bindings_for_features()
  - Apply sanitizer transforms
  - categorize_bindings()
  - Write to $OUT_DIR/bindings-{version}-{slug}.rs
```

### Embedded Bindings Structure

```
bindings/
├── aarch64-linux/
│   ├── 2.7.8/
│   │   ├── bindings.rs  (~196KB)
│   │   └── bindings.cfg (~1KB)
│   ├── 3.0.7/
│   │   ├── bindings.rs  (~230KB)
│   │   └── bindings.cfg
│   └── 3.1.7/ ... 3.4.5/
│       ├── bindings.rs  (~850KB)
│       └── bindings.cfg
├── arm-linux/ (2.7.8 - 3.4.5)
├── x64-mingw-ucrt/ (3.1.7 - 3.4.5)
├── x64-mingw32/ (2.7.8 - 3.0.7)
└── aarch64-mingw-ucrt/ (3.4.5)
```

**Total:** 19 platform/version combinations, 12.7MB uncompressed

### Memory Optimization (Dec 2024)

**Problem:** Original implementation decompressed the 12.7MB tarball twice per build:
1. Once to build an index of available bindings
2. Again to extract specific bindings files

**Solution:** Extract-once-to-disk strategy:
- Added `ensure_extracted()` that extracts tarball to disk on first access
- `build_embedded_index()` now walks extracted directory (filesystem I/O)
- `extract_embedded_bindings()` uses simple `fs::read_to_string()`

**Impact:**
| Operation | Before | After | Savings |
|-----------|--------|-------|---------|
| Build index | 12.7MB decompress | Filesystem walk | -12.7MB |
| Extract files | 12.7MB decompress | File reads | -12.7MB |
| **Total per build** | **25.4MB** | **12.7MB (once)** | **~50%** |

**Cache Invalidation:**
- Hash computed from: `length + first_8_bytes + last_8_bytes`
- Stored in `$OUT_DIR/rb-sys-embedded-bindings/.extracted`
- Changes to tarball trigger re-extraction

### Environment Variables

**Build-time:**
- `RB_SYS_PREGENERATED_BINDINGS_PATH` - Override with specific bindings.rs file
- `RB_SYS_PREGENERATED_CFG_PATH` - Sidecar cfg metadata file
- `RB_SYS_FORCE_BINDGEN` - Skip pre-generated bindings, always use bindgen

**Runtime (set by rb-sys during build):**
- `OUT_DIR` - Extraction directory (or temp dir for tests)
- `BINDGEN_EXTRA_CLANG_ARGS` - Additional clang flags for cross-compilation

### Supported Platforms

| Ruby Platform | Rust Target | Ruby Versions |
|--------------|-------------|---------------|
| aarch64-linux | aarch64-unknown-linux-gnu | 2.7, 3.0-3.4 |
| arm-linux | arm-unknown-linux-gnueabihf | 2.7, 3.0-3.4 |
| x64-mingw-ucrt | x86_64-pc-windows-gnu | 3.1-3.4 |
| x64-mingw32 | x86_64-pc-windows-gnu | 2.7, 3.0 |
| aarch64-mingw-ucrt | aarch64-pc-windows-gnullvm | 3.4 |

### Feature Filtering

Pre-generated bindings include all items. At runtime, items are filtered based on features:

- **Without `bindgen-rbimpls`**: Removes items matching `^rbimpl_.*` and `^RBIMPL_.*`
- **Without `bindgen-deprecated-types`**: Removes items matching `^_bindgen_ty_9.*`

This is implemented via regex filtering on the parsed syn AST.

### Known Memory Consumers

While the double-decompression issue is fixed, significant memory usage remains:

1. **syn::parse_file()**: Parsing ~800KB of Rust source creates ~40-80MB AST
   - syn AST nodes have 10-50x memory overhead vs source text
   - This is the dominant memory consumer (~80-100MB peak)

2. **Token stream conversion**: `into_token_stream().to_string()` creates another large allocation

3. **Bindings processing**: Filtering, sanitizer transforms, categorization operate on full AST

**Future optimization opportunities:**
- Pre-process bindings at generation time to avoid runtime parsing
- Use lighter-weight text filtering instead of syn parsing
- Generate separate bindings per feature combination

---

## 2. Embedded Tooling (cargo-gem) [PARTIAL/PLANNED]

### What Was Built

A framework for embedding Zig compiler and libclang into `cargo-gem` binaries with integrity verification and automatic extraction.

**Status:** Infrastructure is in place, but tool archives are not yet packaged.

### Key Features

✅ **Unified Asset System**: Tools packaged alongside Ruby sysroots in single `assets.tar.zst`
✅ **BLAKE3 Verification**: Type-safe hash verification at build and runtime
✅ **Lazy Extraction**: Tools extracted on-demand to cache directory
✅ **Auto-wiring**: Automatic environment variable configuration
✅ **CLI Inspection**: `cargo gem tools` command to view embedded tools
✅ **Multi-platform**: Supports all platforms in `data/toolchains.json`
✅ **Backward Compatible**: Legacy `bundled-zig` feature still works

### Files

**New Files:**
1. `data/tools.json` - Tool manifest (maps host → tool → metadata)
2. `crates/rb-sys-cli/src/blake3_hash.rs` - Type-safe BLAKE3 hash type
3. `crates/rb-sys-cli/src/tools.rs` - Tool extraction helpers
4. `crates/rb-sys-cli/src/libclang.rs` - libclang configuration module
5. `crates/rb-sys-cli/phase_1/src/tools.rs` - Tool manifest loader
6. `crates/rb-sys-cli/EMBEDDED_TOOLS.md` - User documentation

**Modified Files:**
1. `crates/rb-sys-cli/Cargo.toml` - Added `blake3` and `thiserror` dependencies
2. `crates/rb-sys-cli/src/main.rs` - Added tools module, enhanced Tools command
3. `crates/rb-sys-cli/src/assets/manifest.rs` - Added `ToolInfo` with `blake3` field
4. `crates/rb-sys-cli/src/assets/mod.rs` - Added `extract_tool()` with verification
5. `crates/rb-sys-cli/phase_1/src/manifest.rs` - Added `RuntimeTool` and tools field
6. `crates/rb-sys-cli/src/build.rs` - Integrated libclang env configuration
7. `crates/rb-sys-cli/src/zig/manager.rs` - Added `try_unified_assets_zig()`

### Data Flow

#### Build Time (rake cli:prepare)

```
data/tools.json
     ↓
phase_1 loads manifest
     ↓
Generates RuntimeTool entries
     ↓
Writes to data/derived/rb-sys-cli-manifest.json
     ↓
Packages into crates/rb-sys-cli/src/embedded/assets.tar.zst
     ↓
Embedded via include_bytes!()
```

#### Runtime (cargo gem build)

```
Parse target → Resolve Zig path
     ↓
Check unified assets for zig
     ↓
Extract if present → Verify BLAKE3
     ↓
Set ZIG_PATH = extracted/zig
     ↓
Similarly for libclang → LIBCLANG_PATH
     ↓
Generate shims with resolved paths
     ↓
cargo build with env vars
```

### Tool Manifest Schema

```json
{
  "version": 1,
  "tools": {
    "<host-platform>": {
      "<tool-name>": {
        "version": "X.Y.Z",
        "blake3": "64-char-hex-hash",
        "archive_path": "tools/<host>/<tool>.tar.zst",
        "notes": "optional description"
      }
    }
  }
}
```

### Type Safety: Blake3Hash

```rust
pub struct Blake3Hash([u8; 32]);

impl FromStr for Blake3Hash { /* validates 64 hex chars */ }
impl Serialize / Deserialize { /* JSON string <-> bytes */ }
impl Display { /* lowercase hex */ }
```

**Properties:**
- Compile-time size guarantee
- Early validation failure on malformed hashes
- Shared between phase_1 and runtime

### Security Properties

1. **Content Integrity**: BLAKE3 prevents tampering or corruption
2. **Early Failure**: Invalid hashes fail during phase_1 (build-time)
3. **Runtime Verification**: Re-verifies when extracting from embedded assets
4. **Clear Errors**: Shows expected vs actual hash on mismatch

### Environment Variable Hierarchy

**Zig:**
1. `--zig-path` / `ZIG_PATH` (explicit override)
2. Embedded from unified assets (**NEW, not yet packaged**)
3. Legacy `bundled-zig` feature
4. System `zig` command

**libclang:**
1. `LIBCLANG_PATH` (explicit override)
2. Embedded from unified assets (**NEW, not yet packaged**)
3. System libclang (bindgen default)

### CLI Commands

```bash
# View embedded tools for current host
cargo gem tools

# Clear entire cache (tools + sysroots)
cargo gem cache clear

# Show cache location
cargo gem cache path
```

### Next Steps: To Make Tools Functional

1. **Populate `data/tools.json`** with real BLAKE3 hashes:
   - Download Zig 0.13.0 for each host platform
   - Download/build libclang 19.1.5 for each host platform
   - Compute BLAKE3 hashes: `b3sum <file>`
   - Update `data/tools.json` with real hashes

2. **Update phase_0** to download tools (optional):
   - Extend `phase_0` to fetch tool archives from URLs
   - Verify hashes during download
   - Store in build cache alongside Ruby assets

3. **Update phase_1 asset packaging**:
   - Read tool archives from cache or specified directory
   - Pack into `assets.tar.zst` at the specified `archive_path`
   - Include in embedded manifest

4. **Test end-to-end**:
   ```bash
   rake cli:prepare         # Package tools
   cargo build -p rb-sys-cli  # Embed tools
   cargo gem build --target <target>  # Use embedded tools
   ```

### Tool Archive Structure

Each tool archive should be a `.tar.zst` file with this structure:

**Zig:**
```
zig/
├── zig (or zig.exe on Windows)
├── lib/
└── ... (rest of Zig distribution)
```

**libclang:**
```
libclang/
├── lib/
│   ├── libclang.so (Linux)
│   ├── libclang.dylib (macOS)
│   └── libclang.dll (Windows)
├── include/
│   └── clang/ (resource dir headers)
└── ... (other LLVM libs if needed)
```

---

## Performance Considerations

### Pre-Generated Bindings
- **Binary Size**: +282KB compressed (rb-sys-build)
- **Extraction Time**: First-run ~50ms (zstd decompress), then cached
- **Memory Peak**: ~80-100MB during syn parsing
- **Decompression**: Single 12.7MB extraction per build (cached in OUT_DIR)

### Embedded Tools (when packaged)
- **Binary Size**: +50-150MB per host platform per tool
- **Extraction Time**: First-run ~2-5s, then cached
- **BLAKE3 Speed**: Very fast (~5GB/s), minimal overhead
- **zstd Decompression**: Efficient, ~500MB/s typical

## Testing Strategy

### Pre-Generated Bindings
1. **Unit Tests** (passing):
   - Platform normalization
   - Index population from extracted directory
   - Bindings extraction
   - Hash computation

2. **Integration** (implicit):
   - rb-sys-build tests use pregenerated module
   - Cross-compilation builds exercise full path

### Embedded Tools
1. **Unit Tests**:
   - `Blake3Hash` parsing and serialization
   - Tool extraction with mock archives
   - Manifest loading

2. **Integration Tests**:
   - Full extraction flow with real (small) tool archives
   - BLAKE3 verification (valid and invalid hashes)
   - Environment variable configuration

3. **End-to-End**:
   - Build rb-sys-cli with embedded tools
   - Run `cargo gem build` for all supported targets
   - Verify tools extracted and used correctly

## Compatibility

- **MSRV**: Rust 1.71 (unchanged)
- **Platforms**: All existing platforms supported
- **Backward Compat**:
  - Pre-generated bindings fall back to bindgen
  - Legacy `bundled-zig` feature still works
- **Graceful Degradation**: Falls back to system tools if not embedded

## Known Limitations

### Pre-Generated Bindings
1. Limited to 5 platforms and Ruby 2.7-3.4 (19 combinations)
2. syn parsing still consumes 40-80MB peak memory
3. Feature filtering requires full AST parse
4. Generated bindings ~3-4x larger than original headers

### Embedded Tools
1. Tools are per-host (not per-target), so cross-compiling the CLI itself requires multiple builds
2. No automatic tool updates (must re-run `rake cli:prepare`)
3. Cache grows over time (old versions not auto-cleaned)
4. Windows ARM64 support TBD
5. **Tool archives not yet packaged** - infrastructure ready, awaiting tool preparation

## Future Enhancements

### Pre-Generated Bindings
- [ ] Pre-process bindings to avoid runtime syn parsing
- [ ] Generate per-feature variants to skip filtering
- [ ] Compress with better zstd levels for smaller binary
- [ ] Support for more platforms (musl, macOS)
- [ ] Incremental bindings updates

### Embedded Tools
- [ ] Auto-download tools if missing from phase_0 cache
- [ ] Version update detection and prompts
- [ ] Cache cleanup command (`cargo gem tools prune`)
- [ ] Tool signature verification (in addition to BLAKE3)
- [ ] Incremental asset updates (delta encoding)
- [ ] Compression ratio optimization per tool

## References

- **Pre-generated bindings**: `crates/rb-sys-build/src/pregenerated.rs`
- **Embedded tools**: `crates/rb-sys-cli/src/tools.rs`
- **User docs**: `crates/rb-sys-cli/EMBEDDED_TOOLS.md`
- **Phase system**: `crates/rb-sys-cli/AGENTS.md`
