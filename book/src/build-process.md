# The Build Process

## Overview

This chapter explains what happens behind the scenes when rb-sys compiles your Rust extension, helping you debug issues
and optimize builds.

This chapter explains what happens behind the scenes when rb-sys compiles your Rust extension. Understanding this
process will help you debug issues and optimize your extension.

## How rb-sys Compiles Your Code

When you run `bundle exec rake compile`, several steps happen in sequence:

1. Ruby's `mkmf` system reads your `extconf.rb` file
2. The `create_rust_makefile` function generates a Makefile
3. Cargo builds your Rust code as a dynamic library
4. The resulting binary is copied to your gem's lib directory

Let's examine each step in detail.

## The Role of extconf.rb

The `extconf.rb` file is the entry point for Ruby's native extension system. For rb-sys projects, it typically looks
like this:

```ruby
# extconf.rb
require "mkmf"
require "rb_sys/mkmf"

create_rust_makefile("my_gem/my_gem")
```

The `create_rust_makefile` function:

1. Sets up the environment for compiling Rust code
2. Generates a Makefile with appropriate Cargo commands
3. Configures where the compiled library should be placed

### Configuration Options

You can customize the build process by passing a block to `create_rust_makefile`:

```ruby
create_rust_makefile("my_gem/my_gem") do |config|
  # Set cargo profile (defaults to ENV["RB_SYS_CARGO_PROFILE"] or :dev)
  config.profile = ENV.fetch("MY_GEM_PROFILE", :dev).to_sym

  # Enable specific cargo features
  config.features = ["feature1", "feature2"]

  # Set environment variables for cargo
  config.env = { "SOME_VAR" => "value" }

  # Specify extra Rust flags
  config.extra_rustflags = ["--cfg=feature=\"custom_feature\""]

  # Clean up target directory after installation to reduce gem size
  config.clean_after_install = true

  # Force installation of Rust toolchain if not present
  config.force_install_rust_toolchain = "stable"

  # Auto-install Rust toolchain during gem installation
  config.auto_install_rust_toolchain = true
end
```

For a complete reference of all available configuration options, see the
[rb_sys Gem Configuration](./api-reference/rb-sys-gem-config.md) documentation.

## Environment Variables

Several environment variables affect the build process:

| Variable                | Description                                | Default |
| ----------------------- | ------------------------------------------ | ------- |
| `RB_SYS_CARGO_PROFILE`  | Cargo profile to use (`dev` or `release`)  | `dev`   |
| `RB_SYS_CARGO_FEATURES` | Comma-separated list of features to enable | None    |
| `RB_SYS_CARGO_ARGS`     | Additional arguments to pass to cargo      | None    |

For example:

```bash
RB_SYS_CARGO_PROFILE=release bundle exec rake compile
```

## Debugging the Build Process

When things go wrong, you can debug the build process:

### 1. Enable Verbose Output

```bash
bundle exec rake compile VERBOSE=1
```

### 2. Inspect Generated Files

Look at the generated Makefile and Cargo configuration:

```bash
cat ext/my_gem/Makefile
```

### 3. Run Cargo Directly

You can run Cargo commands directly in the extension directory:

```bash
cd ext/my_gem
cargo build -v
```

## Optimizing the Build

### Development vs. Release Builds

During development, use the default dev profile for faster compilation:

```bash
RB_SYS_CARGO_PROFILE=dev bundle exec rake compile
```

For production releases, use the release profile for optimized performance:

```bash
RB_SYS_CARGO_PROFILE=release bundle exec rake compile
```

### Cargo Configuration

In your Cargo.toml, you can customize optimization levels:

```toml
[profile.release]
lto = true             # Link-time optimization
opt-level = 3          # Maximum optimization
codegen-units = 1      # Optimize for size at the expense of compile time
```

### Build Scripts (build.rs)

For advanced customization, you can use Rust's build script feature:

```rust
// ext/my_gem/build.rs
fn main() {
    // Detect features at build time
    if std::env::var("TARGET").unwrap().contains("windows") {
        println!("cargo:rustc-cfg=feature=\"windows\"");
    }

    // Link to system libraries if needed
    println!("cargo:rustc-link-lib=dylib=ssl");

    // Rerun if specific files change
    println!("cargo:rerun-if-changed=src/native_code.h");
}
```

Remember to add this to your `Cargo.toml`:

```toml
# ext/my_gem/Cargo.toml
[package]
# ...
build = "build.rs"
```

## Cross-Compilation with rb-sys-dock

The real power of rb-sys is its ability to cross-compile extensions using `rb-sys-dock`. This tool runs your build in
Docker containers configured for different platforms.

### Basic Cross-Compilation

To set up cross-compilation with the `RbSys::ExtensionTask`:

```ruby
# Rakefile
RbSys::ExtensionTask.new("my_gem", GEMSPEC) do |ext|
  ext.lib_dir = "lib/my_gem"
  ext.cross_compile = true
  ext.cross_platform = ['x86_64-linux', 'x86_64-darwin', 'arm64-darwin']
end
```

Then you can cross-compile with:

```bash
bundle exec rake native:my_gem:x86_64-linux
```

### Using rb-sys-dock Directly

You can also use rb-sys-dock directly:

```bash
bundle exec rb-sys-dock --platform x86_64-linux --build
```

### Supported Platforms

rb-sys supports many platforms, including:

- x86_64-linux (Linux on Intel/AMD 64-bit)
- x86_64-linux-musl (Static Linux builds)
- aarch64-linux (Linux on ARM64)
- x86_64-darwin (macOS on Intel)
- arm64-darwin (macOS on Apple Silicon)
- x64-mingw-ucrt (Windows 64-bit UCRT)

## CI/CD with oxidize-rb/actions

The [oxidize-rb/actions](https://github.com/oxidize-rb/actions) repository provides GitHub Actions specifically designed
for rb-sys projects:

### setup-ruby-and-rust

```yaml
# .github/workflows/ci.yml
jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
        ruby: [3.0, 3.1, 3.2, 3.3]
    steps:
      - uses: actions/checkout@v4
      - uses: oxidize-rb/actions/setup-ruby-and-rust@v1
        with:
          ruby-version: ${{ matrix.ruby }}
          bundler-cache: true
          cargo-cache: true
      - name: Compile
        run: bundle exec rake compile
      - name: Test
        run: bundle exec rake test
```

### cross-gem

```yaml
# .github/workflows/cross-gem.yml
jobs:
  cross_gems:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        platform: ["x86_64-linux", "x86_64-darwin", "arm64-darwin"]
    steps:
      - uses: actions/checkout@v4
      - uses: ruby/setup-ruby@v1
        with:
          ruby-version: "3.3"
      - uses: oxidize-rb/actions/cross-gem@v1
        with:
          platform: ${{ matrix.platform }}
```

For a complete CI/CD setup, combine these actions to test your extension on multiple Ruby versions and platforms, then
cross-compile for release.

## Next Steps

- Explore [Cross-Platform Development](cross-platform.md) to learn about cross-compilation.
- Learn [Debugging](debugging.md) techniques to troubleshoot build failures.
- See [Testing Extensions](testing.md) for CI/CD testing strategies.
- Dive into [Project Setup](project-setup.md) for organizing your gem’s structure.
