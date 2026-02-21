# rb-sys-cross

Cross-compile Ruby native extensions without Docker. Uses [zig](https://ziglang.org/) + [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild) to produce native `.gem` files for multiple platforms from a single machine.

## Prerequisites

- [Rust](https://rustup.rs/)
- [Zig](https://ziglang.org/download/) (`brew install zig` on macOS)
- [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild) (`cargo install cargo-zigbuild`)

Or run `rb-sys-cross setup` to check/install everything.

## Usage

```bash
# Cross-compile for multiple platforms and Ruby versions
rb-sys-cross build \
  --platform aarch64-linux \
  --platform x86_64-linux \
  --ruby-version 3.3 --ruby-version 3.4

# Output: pkg/my_gem-1.0.0-aarch64-linux.gem, pkg/my_gem-1.0.0-x86_64-linux.gem
```

Gem metadata (name, version, authors, license) is derived automatically from `Cargo.toml` via `cargo metadata`. No Ruby required.

## Gem structure

The output `.gem` follows the same conventions as nokogiri, commonmarker, and other precompiled native gems:

```
my_gem-1.0.0-aarch64-linux.gem
├── metadata.gz          # gemspec YAML (platform, ruby version bounds, etc.)
├── data.tar.gz
│   ├── lib/my_gem.rb              # entrypoint (loads extension.rb then your code)
│   ├── lib/my_gem/extension.rb    # version-aware native loader
│   ├── lib/my_gem/3.3/my_gem.so   # compiled for Ruby 3.3
│   └── lib/my_gem/3.4/my_gem.so   # compiled for Ruby 3.4
└── checksums.yaml.gz
```

The gemspec includes `required_ruby_version` bounds (e.g. `>= 3.3, < 3.5.dev`) matching the Ruby versions you build for, so Bundler/RubyGems will only install the gem on supported Ruby versions.

### Extension loading

The generated `lib/<gem_name>/extension.rb` handles loading the correct `.so` for the running Ruby version:

```ruby
begin
  ruby_version = /\d+\.\d+/.match(RUBY_VERSION)
  require_relative "#{ruby_version}/my_gem"
rescue LoadError
  require "my_gem/my_gem"
end
```

If you provide a `lib/<gem_name>.rb` via the `files` config, rb-sys-cross prepends `require "<gem_name>/extension"` to it so the native extension is loaded before your Ruby code. If you don't provide one, a minimal entrypoint is generated.

Your `lib/<gem_name>.rb` should **not** have its own `require` for the native `.so` — the extension loader handles it. For example:

```ruby
# lib/my_gem.rb — the extension loader require is prepended automatically
module MyGem
  # your Ruby code here
end
```

## Commands

```
rb-sys-cross build            # Cross-compile and package native gems
rb-sys-cross setup            # Install toolchain prerequisites
rb-sys-cross list-platforms   # Show supported targets
rb-sys-cross headers --list   # Show cached Ruby header bundles
```

### Build options

```
-p, --platform <PLATFORM>          Target platform (repeatable)
-r, --ruby-version <VERSION>       Ruby version (repeatable)
    --manifest-path <PATH>         Path to Cargo.toml (default: Cargo.toml)
    --profile <PROFILE>            release (default) or dev
    --output-dir <DIR>             Output directory (default: pkg/)
    --features <FEATURES>          Cargo features (repeatable)
    --config <PATH>                Path to rb-sys-cross.toml
```

## Supported Platforms

| Ruby Platform | Rust Target | Glibc |
|---|---|---|
| aarch64-linux | aarch64-unknown-linux-gnu | 2.17 |
| x86_64-linux | x86_64-unknown-linux-gnu | 2.17 |
| arm-linux | arm-unknown-linux-gnueabihf | 2.17 |
| aarch64-linux-musl | aarch64-unknown-linux-musl | — |
| x86_64-linux-musl | x86_64-unknown-linux-musl | — |
| x64-mingw-ucrt | x86_64-pc-windows-gnu | — |
| x64-mingw32 | x86_64-pc-windows-gnu | — |
| aarch64-mingw-ucrt | aarch64-pc-windows-gnullvm | — |

macOS targets (arm64-darwin, x86_64-darwin) are not supported via zig due to missing macOS SDK.

## Configuration

An optional `rb-sys-cross.toml` can be placed next to your `Cargo.toml` for Ruby-specific overrides:

```toml
[gem]
name = "my_extension"           # default: Cargo.toml package name
require_paths = ["lib"]         # default
files = ["lib/**/*.rb"]         # extra Ruby files to include
summary = "Override summary"    # default: Cargo.toml description

[build]
ext_dir = "ext/my_extension"    # default: auto-detect
```

All fields are optional — sensible defaults are derived from `Cargo.toml`.

## How It Works

1. **Resolve** platform → Rust target + glibc version
2. **Ensure** zig, cargo-zigbuild, and Rust target are installed
3. **Download/cache** Ruby headers + rbconfig (built from source with zig cc)
4. **Set** `RBCONFIG_*` env vars (so `rb-sys-build` skips shelling out to Ruby)
5. **Build** each Ruby version: `cargo zigbuild --target <target> --release --lib`
6. **Pack** a native `.gem` with versioned `.so` files, extension loader, and gemspec

## Building Ruby headers

Ruby headers and `libruby-static.a` are cross-compiled from source using zig as the C compiler. The `scripts/build-ruby-cross.sh` script automates this:

```bash
# Build Ruby 3.3.8 headers + libruby for aarch64-linux
./scripts/build-ruby-cross.sh 3.3.8 aarch64-linux
```

Artifacts are cached in `~/.cache/rb-sys-cross/headers/<platform>/<version>/`.
