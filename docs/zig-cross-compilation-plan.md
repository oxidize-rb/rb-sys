# Plan: Replace rake-compiler with Zig for Cross-Compilation

## Executive Summary

This document proposes replacing the current rake-compiler-dock + platform-specific GCC/Clang
toolchain approach with Zig's cross-compilation capabilities for building native Ruby extensions
in Rust. The goal is to simplify the cross-compilation story, reduce Docker image sizes, eliminate
platform-specific toolchain maintenance, and eventually enable Docker-free cross-compilation for
most targets.

## Current Architecture

### How cross-compilation works today

```
User Rakefile
    |
    v
RbSys::ExtensionTask (extends Rake::ExtensionTask)
    |
    v
rb-sys-dock CLI (orchestrates Docker)
    |
    v
Docker container (FROM rake-compiler-dock-image:1.11.0-mri-{PLATFORM})
    |-- Pre-installed cross-compilation toolchain (GCC/Clang per target)
    |-- Pre-installed Ruby for each target platform
    |-- Rust toolchain with target triple
    |
    v
CargoBuilder -> `cargo rustc --target {triple}` with platform-specific linker flags
    |
    v
Native gem artifact (.so/.dylib/.dll)
```

### Key components

| Component | File(s) | Role |
|---|---|---|
| `ExtensionTask` | `gem/lib/rb_sys/extensiontask.rb` | Rake task subclass; sets `@cross_compile` and `@cross_platform` from `RUBY_TARGET` env |
| `CargoBuilder` | `gem/lib/rb_sys/cargo_builder.rb` | Generates `cargo rustc` commands with linker/library flags from RbConfig |
| `Mkmf` | `gem/lib/rb_sys/mkmf.rb` | Generates Makefiles; exports CC/CXX/AR and RBCONFIG_* env vars |
| `ToolchainInfo` | `gem/lib/rb_sys/toolchain_info.rb` | Maps Ruby platform <-> Rust target; stores rake-compiler-dock CC |
| `rb-sys-dock` | `gem/exe/rb-sys-dock` | CLI that pulls Docker images, mounts volumes, runs `rake native:$RUBY_TARGET gem` |
| Dockerfiles | `docker/Dockerfile.*` (12 files) | One per platform; installs GCC/Clang cross-compilers, sets CC/CXX/AR/LINKER env vars |
| Setup scripts | `docker/setup/*.sh` (8 files) | Install Rust, CMake, RubyGems inside Docker |
| `toolchains.json` | `data/toolchains.json` | Source-of-truth for platform mappings and rake-compiler-dock CC values |

### Supported targets

| Ruby Platform | Rust Target | Current CC | Docker Base |
|---|---|---|---|
| x86_64-linux | x86_64-unknown-linux-gnu | x86_64-linux-gnu-gcc | rake-compiler-dock |
| aarch64-linux | aarch64-unknown-linux-gnu | aarch64-linux-gnu-gcc | rake-compiler-dock |
| aarch64-linux-musl | aarch64-unknown-linux-musl | aarch64-linux-musl-gcc | rake-compiler-dock |
| x86_64-linux-musl | x86_64-unknown-linux-musl | x86_64-unknown-linux-musl-gcc | rake-compiler-dock |
| arm-linux | arm-unknown-linux-gnueabihf | arm-linux-gnueabihf-gcc | rake-compiler-dock |
| arm64-darwin | aarch64-apple-darwin | aarch64-apple-darwin-clang | rake-compiler-dock + osxcross |
| x86_64-darwin | x86_64-apple-darwin | x86_64-apple-darwin-clang | rake-compiler-dock + osxcross |
| x64-mingw-ucrt | x86_64-pc-windows-gnu | x86_64-w64-mingw32-gcc | rake-compiler-dock |
| x64-mingw32 | x86_64-pc-windows-gnu | x86_64-windows-gnu-gcc | rake-compiler-dock |
| aarch64-mingw-ucrt | aarch64-pc-windows-gnullvm | aarch64-w64-mingw32-clang | rake-compiler-dock |

### Pain points with the current approach

1. **12 separate Dockerfiles** with platform-specific toolchain configuration
2. **Large Docker images** (~2-4 GB each) because each bundles a full cross-compilation toolchain
3. **rake-compiler-dock dependency** - tightly coupled to their image release cycle (currently 1.11.0)
4. **osxcross complexity** - macOS cross-compilation requires a separate SDK and shebang workarounds
5. **Docker required** - no path to cross-compilation without Docker
6. **Slow CI** - building/pulling large Docker images adds significant time
7. **Hard to add new targets** - each new platform needs a new Dockerfile, toolchain installation, CI matrix entry

---

## Proposed Architecture with Zig

### Why Zig?

Zig's `zig cc` is a drop-in C/C++ cross-compiler that:
- Bundles **libc headers and libraries for 40+ targets** in a single ~40 MB binary
- Supports glibc (multiple versions), musl, mingw, and macOS targets out of the box
- Requires **no separate sysroot installation** for Linux and Windows targets
- Works as a **cross-linker** for Rust via `CC` / `CARGO_TARGET_*_LINKER` env vars
- Is a **single static binary** with no dependencies

### Target architecture

```
User Rakefile
    |
    v
RbSys::ExtensionTask (modified - no longer depends on Rake::ExtensionTask for cross)
    |
    v
rb-sys-cross CLI (new, replaces rb-sys-dock for cross-compilation)
    |
    +--[Docker mode]----> Lightweight Docker container
    |                         |-- Zig (single binary, ~40 MB)
    |                         |-- Pre-compiled Ruby headers per target
    |                         |-- Rust toolchain
    |                         v
    +--[Local mode]-----> Local zig + cargo
    |                         |
    v                         v
ZigToolchain (new) -> configures CC/CXX/AR/LINKER env vars using `zig cc`
    |
    v
CargoBuilder -> `cargo rustc --target {triple}` (unchanged interface)
    |
    v
Native gem artifact
```

### What `zig cc` replaces

| Current Tool | Zig Equivalent | Notes |
|---|---|---|
| `x86_64-linux-gnu-gcc` | `zig cc -target x86_64-linux-gnu` | Exact glibc version selectable (e.g., `-target x86_64-linux-gnu.2.17`) |
| `aarch64-linux-gnu-gcc` | `zig cc -target aarch64-linux-gnu` | Same |
| `x86_64-unknown-linux-musl-gcc` | `zig cc -target x86_64-linux-musl` | musl built-in |
| `x86_64-w64-mingw32-gcc` | `zig cc -target x86_64-windows-gnu` | mingw headers built-in |
| `aarch64-apple-darwin-clang` | `zig cc -target aarch64-macos` | **Requires macOS SDK sysroot** |
| `arm-linux-gnueabihf-gcc` | `zig cc -target arm-linux-gnueabihf` | Hard-float ARM built-in |

**Important limitation:** macOS targets still require the macOS SDK (Xcode headers/libs). Zig can
use them via `--sysroot`, but they cannot be freely redistributed. This means Darwin cross-compilation
will still need either Docker (with osxcross) or a local macOS SDK.

---

## Implementation Plan

### Phase 0: Proof of Concept (non-breaking)

**Goal:** Validate that zig cc works as a cross-linker for Rust-based Ruby extensions.

#### 0.1 Create a standalone PoC script

Build one of the existing test extensions (e.g., `rust_reverse` example) using zig as the
cross-compiler, without modifying any rb-sys code:

```bash
# Install zig
# Install Rust target
rustup target add aarch64-unknown-linux-gnu

# Create a zig-cc wrapper script
cat > /tmp/zig-cc-aarch64 << 'EOF'
#!/bin/sh
exec zig cc -target aarch64-linux-gnu.2.17 "$@"
EOF
chmod +x /tmp/zig-cc-aarch64

# Build with zig as the linker
export CC_aarch64_unknown_linux_gnu=/tmp/zig-cc-aarch64
export AR_aarch64_unknown_linux_gnu="zig ar"
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=/tmp/zig-cc-aarch64
cargo build --target aarch64-unknown-linux-gnu
```

**Validate:**
- [ ] .so is produced with correct ELF architecture
- [ ] No undefined symbols from wrong libc version
- [ ] Extension loads correctly on target platform (use QEMU or real hardware)

#### 0.2 Test all Linux and Windows targets

Repeat for each supported target:
- [ ] x86_64-unknown-linux-gnu
- [ ] aarch64-unknown-linux-gnu
- [ ] aarch64-unknown-linux-musl
- [ ] x86_64-unknown-linux-musl
- [ ] arm-unknown-linux-gnueabihf
- [ ] x86_64-pc-windows-gnu (mingw)

#### 0.3 Test macOS targets (with SDK sysroot)

- [ ] aarch64-apple-darwin (requires `--sysroot` pointing to macOS SDK)
- [ ] x86_64-apple-darwin

#### 0.4 Document findings

- Which targets work out of the box
- Which targets need additional configuration (sysroots, etc.)
- Any incompatibilities with Rust's linker expectations
- Performance comparison (compile times) vs. current GCC/Clang toolchains

---

### Phase 1: `RbSys::ZigToolchain` module

**Goal:** Create a Ruby module that configures zig as the cross-compiler for cargo builds.

#### 1.1 New file: `gem/lib/rb_sys/zig/toolchain.rb`

```ruby
module RbSys
  module Zig
    class Toolchain
      ZIG_TARGET_MAP = {
        "x86_64-unknown-linux-gnu"      => "x86_64-linux-gnu",
        "aarch64-unknown-linux-gnu"     => "aarch64-linux-gnu",
        "aarch64-unknown-linux-musl"    => "aarch64-linux-musl",
        "x86_64-unknown-linux-musl"     => "x86_64-linux-musl",
        "arm-unknown-linux-gnueabihf"   => "arm-linux-gnueabihf",
        "x86_64-pc-windows-gnu"         => "x86_64-windows-gnu",
        "aarch64-apple-darwin"          => "aarch64-macos",
        "x86_64-apple-darwin"           => "x86_64-macos",
      }.freeze

      attr_reader :rust_target, :zig_target, :glibc_version

      def initialize(rust_target, glibc_version: nil)
        @rust_target = rust_target
        @zig_target = ZIG_TARGET_MAP.fetch(rust_target)
        @glibc_version = glibc_version
      end

      # Full zig target string, e.g., "x86_64-linux-gnu.2.17"
      def zig_target_with_glibc
        if glibc_version && zig_target.include?("-gnu")
          "#{zig_target}.#{glibc_version}"
        else
          zig_target
        end
      end

      # Generate a wrapper script for `zig cc -target <target>`
      def cc_wrapper_path
        # Creates a small shell script in a tmp dir
      end

      # Environment variables to set for cargo cross-compilation
      def cargo_env
        target_slug = rust_target.tr("-", "_")
        {
          "CC_#{target_slug}"  => cc_wrapper_path,
          "CXX_#{target_slug}" => cxx_wrapper_path,
          "AR_#{target_slug}"  => ar_wrapper_path,
          "CARGO_TARGET_#{target_slug.upcase}_LINKER" => cc_wrapper_path,
        }
      end
    end
  end
end
```

#### 1.2 Zig wrapper script generation

The wrapper scripts are necessary because `zig cc -target foo` is multiple arguments, but
`CC` / `CARGO_TARGET_*_LINKER` expect a single executable path. The approach:

```ruby
def generate_wrapper(tool, target)
  wrapper_dir = File.join(Dir.tmpdir, "rb-sys-zig-#{rust_target}")
  FileUtils.mkdir_p(wrapper_dir)
  path = File.join(wrapper_dir, "zig-#{tool}")
  File.write(path, <<~SH)
    #!/bin/sh
    exec zig #{tool} -target #{zig_target_with_glibc} "$@"
  SH
  File.chmod(0o755, path)
  path
end
```

**Alternative (better):** Use Cargo's `[target.*.linker]` in a generated `.cargo/config.toml`
to avoid wrapper scripts entirely:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "zig"
rustflags = ["-C", "link-arg=cc", "-C", "link-arg=-target", "-C", "link-arg=aarch64-linux-gnu.2.17"]
```

This approach is cleaner but requires generating/managing a `.cargo/config.toml` file.
Both approaches should be evaluated in Phase 0.

#### 1.3 Zig installation management

Add a helper to detect or install zig:

```ruby
module RbSys
  module Zig
    class Installer
      ZIG_VERSION = "0.13.0"  # Pin to a known-good version

      def self.find_or_install
        # 1. Check if `zig` is already on PATH
        # 2. Check if zig is in rb-sys cache dir
        # 3. Download pre-built zig binary to cache dir
      end
    end
  end
end
```

Zig is distributed as a single static binary tarball (~40 MB), making it trivial to download
and cache.

#### 1.4 Update `toolchains.json` schema

Add zig-specific fields alongside existing rake-compiler-dock fields:

```json
{
  "ruby-platform": "aarch64-linux",
  "rust-target": "aarch64-unknown-linux-gnu",
  "rake-compiler-dock": {
    "cc": "aarch64-linux-gnu-gcc"
  },
  "zig": {
    "target": "aarch64-linux-gnu",
    "glibc-version": "2.17",
    "sysroot-required": false
  },
  "supported": true
}
```

---

### Phase 2: Integrate into CargoBuilder

**Goal:** Make `CargoBuilder` use zig when available and configured.

#### 2.1 Add zig cross-compilation mode to `CargoBuilder`

Modify `gem/lib/rb_sys/cargo_builder.rb`:

```ruby
def build_env
  build_env = rb_config_env

  if use_zig_cross?
    zig_toolchain = RbSys::Zig::Toolchain.new(target)
    build_env.merge!(zig_toolchain.cargo_env)
  end

  build_env["RUBY_STATIC"] = "true" if ruby_static? && ENV.key?("RUBY_STATIC")
  build_env.merge(env)
end

def use_zig_cross?
  target && ENV.fetch("RB_SYS_CROSS_COMPILER", "auto") != "gcc" &&
    RbSys::Zig::Toolchain.supports?(target)
end
```

#### 2.2 Handle linker_args differences

The current `linker_args` method extracts CC from `RbConfig::MAKEFILE_CONFIG["CC"]`, which
is the *target* Ruby's CC (e.g., `aarch64-linux-gnu-gcc`). When using zig, we need to
override this:

```ruby
def linker_args
  if use_zig_cross?
    zig_toolchain = RbSys::Zig::Toolchain.new(target)
    ["-C", "linker=#{zig_toolchain.cc_wrapper_path}"]
  else
    # existing logic
  end
end
```

#### 2.3 Handle platform_specific_rustc_args

The existing platform-specific linker flags (ASLR for mingw, `-undefined dynamic_lookup` for
darwin, `-crt-static` for musl) should remain unchanged -- these are passed to the linker
regardless of whether it is GCC, Clang, or Zig (which wraps Clang/LLD internally).

Verify each flag is compatible with zig's linker:
- [ ] `-Wl,--dynamicbase` (mingw) -- zig uses LLD for mingw, supports this
- [ ] `-Wl,-undefined,dynamic_lookup` (darwin) -- zig uses LLD for macOS, supports this
- [ ] `-static-libgcc` (mingw) -- may need adjustment for zig (zig links statically by default)
- [ ] `-C target-feature=-crt-static` (musl) -- Rust flag, independent of linker

---

### Phase 3: Lightweight Docker Images

**Goal:** Replace the 12 rake-compiler-dock-based Dockerfiles with a single universal image
(or a few minimal images) based on zig.

#### 3.1 New universal Dockerfile

```dockerfile
FROM debian:bookworm-slim

# Install minimal dependencies
RUN apt-get update && apt-get install -y \
    curl git build-essential ruby ruby-dev \
    && rm -rf /var/lib/apt/lists/*

# Install zig (single binary, ~40 MB)
ARG ZIG_VERSION=0.13.0
RUN curl -fsSL "https://ziglang.org/download/${ZIG_VERSION}/zig-linux-$(uname -m)-${ZIG_VERSION}.tar.xz" \
    | tar -xJ -C /opt && ln -s /opt/zig-linux-*/zig /usr/local/bin/zig

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:$PATH"

# Add all supported Rust targets
RUN rustup target add \
    x86_64-unknown-linux-gnu \
    aarch64-unknown-linux-gnu \
    aarch64-unknown-linux-musl \
    x86_64-unknown-linux-musl \
    arm-unknown-linux-gnueabihf \
    x86_64-pc-windows-gnu \
    aarch64-apple-darwin \
    x86_64-apple-darwin

# Pre-compiled Ruby headers for each target platform
# (These still need to come from rake-compiler-dock or be compiled from source)
COPY ruby-headers/ /opt/ruby-headers/
```

**Size comparison estimate:**
- Current: ~2-4 GB per platform image x 12 = ~24-48 GB total
- Proposed: ~500 MB single universal image (or ~1 GB with Ruby headers for all targets)

#### 3.2 The Ruby headers problem

The biggest remaining dependency on rake-compiler-dock is **pre-compiled Ruby headers and
libraries for each target platform**. These are needed because:

1. `rb-sys-build` (the Rust crate) reads RbConfig values at build time
2. The generated Makefile uses RbConfig for DLEXT, SOEXT, library paths
3. The extension links against libruby

**Options for Ruby headers:**

| Option | Pros | Cons |
|---|---|---|
| **A.** Continue using rake-compiler-dock Ruby headers | Battle-tested, complete | Still depends on rake-compiler-dock for header generation |
| **B.** Cross-compile Ruby from source per target | Full control, reproducible | Complex build matrix; macOS requires SDK |
| **C.** Extract minimal Ruby headers/stubs | Much smaller images | Must validate completeness for all Ruby versions |
| **D.** Use rb-sys's existing RBCONFIG_* env var approach | Already works today | Need to serialize RbConfig from each target Ruby |

**Recommended approach:** Option D. The `CargoBuilder` already exports all `RbConfig::CONFIG`
values as `RBCONFIG_*` environment variables. We can pre-generate these for each
(ruby_version, target_platform) pair and bundle them as JSON/env files. This decouples us
from needing the actual cross-compiled Ruby installed in the image.

The remaining need is for the Ruby header files themselves (ruby.h, etc.) which are
required by `bindgen` (used by `rb-sys-build`). These headers are mostly platform-independent
(with a few config.h differences). We could:

1. Extract and store `ruby/include/` and platform-specific `ruby/config.h` files
2. Pass them via `BINDGEN_EXTRA_CLANG_ARGS="--sysroot=..."` and
   `RBCONFIG_rubyhdrdir`, `RBCONFIG_rubyarchhdrdir`

#### 3.3 Phased Dockerfile migration

**Phase 3a:** Create `docker/Dockerfile.zig-universal` alongside existing Dockerfiles.
Test it for Linux targets only.

**Phase 3b:** Add Windows (mingw) support to the universal image.

**Phase 3c:** Add macOS support (requires bundling osxcross SDK or mount-point approach).

**Phase 3d:** Deprecate per-platform Dockerfiles. Update `rb-sys-dock` to use universal image
by default.

---

### Phase 4: `rb-sys-cross` CLI (Docker-free mode)

**Goal:** Enable cross-compilation without Docker for Linux and Windows targets.

#### 4.1 New CLI: `gem/exe/rb-sys-cross`

```
Usage: rb-sys-cross [OPTIONS]

Cross-compile a Ruby native extension using Zig.

Options:
    -p, --platform PLATFORM    Target Ruby platform (e.g., x86_64-linux)
    -r, --ruby-versions LIST   Ruby versions to target
        --use-docker           Force Docker mode (for macOS targets)
        --zig-path PATH        Path to zig binary
        --build                Build the gem
    -h, --help                 Print help
```

#### 4.2 Cross-compilation without Docker

For Linux and Windows targets, the only requirements would be:
1. **Zig** (auto-downloaded if not present)
2. **Rust** with the appropriate target installed
3. **Pre-generated RbConfig data** (bundled with the rb_sys gem or downloaded)
4. **Ruby header files** (bundled with the rb_sys gem or downloaded)

```ruby
module RbSys
  class CrossCompiler
    def initialize(platform:, ruby_versions:)
      @toolchain = ToolchainInfo.new(platform)
      @zig = Zig::Toolchain.new(@toolchain.rust_target)
      @ruby_versions = ruby_versions
    end

    def compile!
      ensure_zig_installed!
      ensure_rust_target_installed!
      download_ruby_headers!     # if not cached
      configure_environment!
      run_cargo_build!
      package_gem!
    end
  end
end
```

#### 4.3 Pre-built Ruby header bundles

Publish Ruby header bundles as GitHub Release assets or to a CDN:

```
rb-sys-ruby-headers-3.3.0-x86_64-linux.tar.gz
rb-sys-ruby-headers-3.3.0-aarch64-linux.tar.gz
rb-sys-ruby-headers-3.4.0-x86_64-linux.tar.gz
...
```

Each bundle contains:
- `include/ruby-{version}/` (header files)
- `rbconfig.json` (serialized RbConfig::CONFIG)

CI generates these as part of the rb-sys release process.

---

### Phase 5: Update ExtensionTask and Mkmf

**Goal:** Update the Rake integration to support zig-based cross-compilation.

#### 5.1 Modify `ExtensionTask`

The current `ExtensionTask` inherits from `Rake::ExtensionTask` and relies on
rake-compiler's cross-compilation machinery. With zig, we have two paths:

**Option A (Recommended): Keep rake-compiler for task orchestration, replace only the toolchain**

rake-compiler is still useful for:
- Managing the `native:{platform}` Rake tasks
- Packaging platform-specific gems
- Handling the gemspec manipulation for native gems

We only need to replace what happens *inside* the build -- which is already handled by
`CargoBuilder`. This means `ExtensionTask` changes are minimal:

```ruby
class ExtensionTask < Rake::ExtensionTask
  def init(name = nil, gem_spec = nil)
    super
    # ... existing setup ...
    @cross_compiler = ENV.fetch("RB_SYS_CROSS_COMPILER", "auto")
  end
end
```

**Option B: Replace rake-compiler entirely**

Only consider this if rake-compiler proves to be an obstacle. Would require reimplementing
native gem packaging, which is significant work for little benefit.

#### 5.2 Modify `Mkmf`

Update the Makefile generation to support zig:

```ruby
def env_vars(builder)
  if builder.use_zig_cross?
    zig_env_vars(builder)
  else
    # existing gcc/clang logic
  end
end

def zig_env_vars(builder)
  zig = RbSys::Zig::Toolchain.new(builder.target)
  lines = zig.cargo_env.map { |k, v| export_env(k, v) }
  lines.join("\n")
end
```

---

### Phase 6: CI/CD Updates

#### 6.1 New CI workflow: `zig-cross.yml`

```yaml
name: Zig Cross-Compilation
on: [push, pull_request]

jobs:
  cross-compile:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        platform:
          - x86_64-linux
          - aarch64-linux
          - aarch64-linux-musl
          - x86_64-linux-musl
          - arm-linux
          - x64-mingw-ucrt
        ruby: ['3.2', '3.3', '3.4']
    steps:
      - uses: actions/checkout@v4
      - uses: goto-bus-stop/setup-zig@v2
      - uses: ruby/setup-ruby@v1
        with:
          ruby-version: ${{ matrix.ruby }}
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.rust_target }}
      - run: rb-sys-cross --platform ${{ matrix.platform }} --build
      - name: Verify artifact
        run: file pkg/*.gem
```

#### 6.2 Keep existing Docker CI as fallback

During the transition, run both pipelines in parallel to catch regressions.

#### 6.3 Reduce Docker image matrix

Once zig cross-compilation is validated, the Docker workflow can be simplified:
- Keep Docker images only for macOS targets (which need osxcross SDK)
- Or use a single universal Docker image for all targets

---

## Migration Strategy

### Guiding principles

1. **Additive, not replacement** -- zig support is added alongside existing rake-compiler-dock
   support. Users can opt in.
2. **Environment variable driven** -- `RB_SYS_CROSS_COMPILER=zig` enables zig mode;
   default remains `auto` (which picks zig if available, falls back to existing toolchain).
3. **No breaking changes to public API** -- `RbSys::ExtensionTask`, `create_rust_makefile`,
   and `rb-sys-dock` continue to work exactly as before.
4. **Darwin targets last** -- macOS cross-compilation has the most constraints (SDK licensing)
   and should be migrated last.

### Rollout timeline

| Phase | Description | Breaking Changes | Prerequisites |
|---|---|---|---|
| 0 | PoC validation | None | None |
| 1 | `RbSys::Zig::Toolchain` module | None | Phase 0 validated |
| 2 | Integrate into `CargoBuilder` | None (opt-in via env var) | Phase 1 |
| 3 | Lightweight Docker images | None (new images, old still work) | Phase 2 |
| 4 | `rb-sys-cross` CLI | None (new CLI, `rb-sys-dock` still works) | Phase 2 |
| 5 | Update ExtensionTask/Mkmf | None (auto-detection) | Phase 2 |
| 6 | CI/CD updates | None | Phase 2 |
| 7 | Deprecate old Docker images | Soft deprecation warnings | Phases 3-6 stable |
| 8 | Remove rake-compiler-dock dep | **Breaking** (major version bump) | Phase 7 + 1 major version |

---

## Risks and Mitigations

### Risk 1: Zig linker incompatibilities

**Risk:** Zig's bundled LLD may produce different linking behavior than GCC's ld or Apple's ld64.

**Mitigation:**
- Phase 0 validates all targets before any code changes
- Keep GCC/Clang fallback available via `RB_SYS_CROSS_COMPILER=gcc`
- Run integration tests against real-world gems (oxi-test, magnus)

### Risk 2: Glibc version mismatch

**Risk:** Zig targets a specific glibc version. If the target Ruby was compiled against a different
glibc version, symbol versioning could cause runtime failures.

**Mitigation:**
- Pin glibc version in `toolchains.json` per target (e.g., `2.17` for maximum compatibility)
- Zig supports explicit glibc version selection: `zig cc -target x86_64-linux-gnu.2.17`
- Test against multiple glibc versions in CI

### Risk 3: macOS SDK licensing

**Risk:** Apple's macOS SDK cannot be freely redistributed, so Docker images cannot bundle it.

**Mitigation:**
- macOS targets continue to use osxcross approach in Docker
- On macOS hosts, use the locally installed SDK (no Docker needed)
- Document the SDK requirement clearly

### Risk 4: Zig version stability

**Risk:** Zig is pre-1.0 and APIs may change between releases.

**Mitigation:**
- Pin a specific Zig version (e.g., 0.13.0)
- Only use `zig cc` / `zig ar` (stable, well-tested interfaces)
- Test against 2-3 zig versions in CI
- `zig cc` is the most stable part of Zig -- it's essentially a Clang frontend

### Risk 5: Upstream gem ecosystem compatibility

**Risk:** Gems that use rb-sys may have C dependencies (via `-sys` crates) that don't compile
with zig cc.

**Mitigation:**
- `zig cc` is highly compatible with GCC/Clang flags (it's Clang underneath)
- Known incompatibilities are rare and well-documented
- Fallback to GCC toolchain remains available

### Risk 6: Windows ARM (aarch64-mingw-ucrt)

**Risk:** Zig's Windows ARM support may be less mature than x86_64.

**Mitigation:**
- aarch64-pc-windows-gnullvm currently uses clang (not gcc) anyway
- Test thoroughly in Phase 0
- Keep existing toolchain as fallback

---

## Files to Create/Modify

### New files

| File | Purpose |
|---|---|
| `gem/lib/rb_sys/zig/toolchain.rb` | Zig toolchain configuration and env var generation |
| `gem/lib/rb_sys/zig/installer.rb` | Zig binary detection and auto-installation |
| `gem/lib/rb_sys/zig/wrapper.rb` | Generate zig-cc / zig-c++ / zig-ar wrapper scripts |
| `gem/lib/rb_sys/cross_compiler.rb` | Orchestrator for Docker-free cross-compilation |
| `gem/exe/rb-sys-cross` | New CLI for zig-based cross-compilation |
| `docker/Dockerfile.zig-universal` | Single lightweight Docker image with zig |
| `.github/workflows/zig-cross.yml` | CI for zig cross-compilation |
| `gem/test/zig/toolchain_test.rb` | Tests for Zig::Toolchain |
| `gem/test/zig/installer_test.rb` | Tests for Zig::Installer |

### Modified files

| File | Changes |
|---|---|
| `data/toolchains.json` | Add `"zig"` section to each toolchain entry |
| `gem/lib/rb_sys/toolchain_info.rb` | Expose zig target, glibc version from toolchains.json |
| `gem/lib/rb_sys/cargo_builder.rb` | Add `use_zig_cross?` method; modify `build_env` and `linker_args` |
| `gem/lib/rb_sys/mkmf.rb` | Update `env_vars` to support zig mode |
| `gem/lib/rb_sys/extensiontask.rb` | Add `cross_compiler` configuration option |
| `gem/rb_sys.gemspec` | No new gem dependencies (zig is a system binary) |
| `Gemfile` | No changes (rake-compiler remains for now) |
| `rakelib/docker.rake` | Add tasks for building zig-universal Docker image |

### Eventually deprecated/removed

| File | Timeline |
|---|---|
| `docker/Dockerfile.x86_64-linux` | After Phase 7 (kept for macOS targets) |
| `docker/Dockerfile.aarch64-linux` | After Phase 7 |
| `docker/Dockerfile.x86_64-linux-musl` | After Phase 7 |
| `docker/Dockerfile.aarch64-linux-musl` | After Phase 7 |
| `docker/Dockerfile.arm-linux` | After Phase 7 |
| `docker/Dockerfile.x64-mingw-ucrt` | After Phase 7 |
| `docker/Dockerfile.x64-mingw32` | After Phase 7 |
| `docker/Dockerfile.aarch64-mingw-ucrt` | After Phase 7 |
| (keep) `docker/Dockerfile.arm64-darwin` | Keep until zig macOS support matures |
| (keep) `docker/Dockerfile.x86_64-darwin` | Keep until zig macOS support matures |

---

## Open Questions

1. **Glibc version policy:** What minimum glibc version should we target? rake-compiler-dock
   currently uses whatever CentOS/Ubuntu the base image provides. With zig we can be explicit
   (e.g., 2.17 for RHEL 7 compat, 2.28 for RHEL 8+).

2. **Zig version pinning:** Should we pin to a single zig version or support a range? Given
   that `zig cc` is quite stable, supporting `>= 0.11` is probably safe.

3. **Ruby header distribution:** How should pre-compiled Ruby headers be distributed?
   Options: bundled in rb_sys gem, separate gem, GitHub releases, CDN.

4. **Default behavior:** Should zig be the default cross-compiler immediately (Phase 2), or
   should it require explicit opt-in until Phase 7?

5. **cargo-zigbuild:** Should we use [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild)
   instead of configuring zig-cc wrapper scripts manually? It handles the wrapper script
   complexity and is well-maintained. Trade-off: adds a Rust binary dependency.

6. **Nix integration:** The project already has a `flake.nix`. Should we add zig to the
   development shell and test the zig cross-compilation path in the Nix CI?
