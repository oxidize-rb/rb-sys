# Cross-Platform Development

## Overview

One of rb-sys's greatest strengths is its support for cross-platform Ruby extensions. This chapter covers how to
develop, test, and distribute extensions across multiple platforms.

## Supported Platforms

rb-sys supports cross-compilation to the following platforms:

| Platform           | Supported | Docker Image               |
| ------------------ | --------- | -------------------------- |
| x86_64-linux       | ✅        | `rbsys/x86_64-linux`       |
| x86_64-linux-musl  | ✅        | `rbsys/x86_64-linux-musl`  |
| aarch64-linux      | ✅        | `rbsys/aarch64-linux`      |
| aarch64-linux-musl | ✅        | `rbsys/aarch64-linux-musl` |
| arm-linux          | ✅        | `rbsys/arm-linux`          |
| arm64-darwin       | ✅        | `rbsys/arm64-darwin`       |
| x64-mingw32        | ✅        | `rbsys/x64-mingw32`        |
| x64-mingw-ucrt     | ✅        | `rbsys/x64-mingw-ucrt`     |
| mswin              | ✅        | not available on Docker    |
| truffleruby        | ✅        | not available on Docker    |

The Docker images are available on [Docker Hub](https://hub.docker.com/r/rbsys/rcd) and are automatically updated with
each rb-sys release.

## Platform Considerations

Ruby extensions face several cross-platform challenges:

- Different operating systems (Linux, macOS, Windows)
- Different CPU architectures (x86_64, ARM64)
- Different Ruby implementations
- Different compilers and linkers
- System libraries and dependencies

rb-sys provides tools to handle these differences effectively.

## Understanding Platform Targets

Ruby identifies platforms with standardized strings:

| Platform String  | Description                   |
| ---------------- | ----------------------------- |
| `x86_64-linux`   | 64-bit Linux on Intel/AMD     |
| `aarch64-linux`  | 64-bit Linux on ARM           |
| `x86_64-darwin`  | 64-bit macOS on Intel         |
| `arm64-darwin`   | 64-bit macOS on Apple Silicon |
| `x64-mingw-ucrt` | 64-bit Windows (UCRT)         |
| `x64-mingw32`    | 64-bit Windows (older)        |

These platform strings are used by:

- RubyGems to select the correct pre-built binary
- rake-compiler for cross-compilation
- rb-sys-dock to build for different platforms

## Conditional Compilation

Rust's conditional compilation features allow you to write platform-specific code:

```rust
// Platform-specific code
#[cfg(target_os = "windows")]
fn platform_specific() {
    // Windows-specific implementation
}

#[cfg(target_os = "macos")]
fn platform_specific() {
    // macOS-specific implementation
}

#[cfg(target_os = "linux")]
fn platform_specific() {
    // Linux-specific implementation
}
```

For architectures:

```rust
#[cfg(target_arch = "x86_64")]
fn arch_specific() {
    // x86_64 implementation
}

#[cfg(target_arch = "aarch64")]
fn arch_specific() {
    // ARM64 implementation
}
```

### Complete Example: File Path Handling

Here's a real-world example of handling paths differently across platforms:

```rust
use std::path::PathBuf;

fn get_config_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let mut path = PathBuf::new();
        if let Some(profile) = std::env::var_os("USERPROFILE") {
            path.push(profile);
            path.push("AppData");
            path.push("Roaming");
            path.push("MyApp");
            path.push("config.toml");
        }
        path
    }

    #[cfg(target_os = "macos")]
    {
        let mut path = PathBuf::new();
        if let Some(home) = std::env::var_os("HOME") {
            path.push(home);
            path.push("Library");
            path.push("Application Support");
            path.push("MyApp");
            path.push("config.toml");
        }
        path
    }

    #[cfg(target_os = "linux")]
    {
        let mut path = PathBuf::new();
        if let Some(config_dir) = std::env::var_os("XDG_CONFIG_HOME") {
            path.push(config_dir);
        } else if let Some(home) = std::env::var_os("HOME") {
            path.push(home);
            path.push(".config");
        }
        path.push("myapp");
        path.push("config.toml");
        path
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        // Default for other platforms
        PathBuf::from("config.toml")
    }
}
```

## Platform-Specific Dependencies

Cargo.toml supports platform-specific dependencies:

```toml
[dependencies]
# Common dependencies...

[target.'cfg(target_os = "linux")'.dependencies]
jemallocator = { version = "0.5", features = ["disable_initial_exec_tls"] }

[target.'cfg(target_os = "windows")'.dependencies]
winapi = { version = "0.3", features = ["winbase"] }

[target.'cfg(target_os = "macos")'.dependencies]
core-foundation = "0.9"
```

### Example: System-specific Memory Allocation

```rust
#[cfg(target_os = "linux")]
use jemallocator::Jemalloc;

#[cfg(target_os = "linux")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

// Rest of your code...
```

## Using build.rs for Platform Detection

The Rust build script (`build.rs`) can be used to detect platforms and configure builds:

```rust
// ext/my_gem/build.rs
fn main() {
    // Detect OS
    let target = std::env::var("TARGET").unwrap_or_default();

    if target.contains("windows") {
        println!("cargo:rustc-link-lib=dylib=user32");
        println!("cargo:rustc-cfg=feature=\"windows_specific\"");
    } else if target.contains("darwin") {
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-cfg=feature=\"macos_specific\"");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=dylib=dl");
        println!("cargo:rustc-cfg=feature=\"linux_specific\"");
    }

    // Tell Cargo to invalidate the built crate whenever the build script changes
    println!("cargo:rerun-if-changed=build.rs");
}
```

Then in your code:

```rust
#[cfg(feature = "windows_specific")]
fn platform_init() {
    // Windows initialization code
}

#[cfg(feature = "macos_specific")]
fn platform_init() {
    // macOS initialization code
}

#[cfg(feature = "linux_specific")]
fn platform_init() {
    // Linux initialization code
}
```

## Cross-Compilation with rb-sys-dock

rb-sys-dock is a Docker-based tool that simplifies cross-compilation:

### Setting Up rb-sys-dock in Your Gem

1. Add rb-sys-dock to your Gemfile:

```ruby
# Gemfile
group :development do
  gem "rb-sys-dock", "~> 0.1"
end
```

2. Configure your Rakefile for cross-compilation:

```ruby
# Rakefile
require "rb_sys/extensiontask"

GEMSPEC = Gem::Specification.load("my_gem.gemspec")

RbSys::ExtensionTask.new("my_gem", GEMSPEC) do |ext|
  ext.lib_dir = "lib/my_gem"
  ext.cross_compile = true
  ext.cross_platform = [
    "x86_64-linux",
    "aarch64-linux",
    "x86_64-darwin",
    "arm64-darwin",
    "x64-mingw-ucrt"
  ]
end
```

### Building for a Specific Platform

To build for a specific platform:

```bash
bundle exec rake native:my_gem:x86_64-linux
```

This creates a platform-specific gem in the `pkg` directory.

### Building for All Platforms

To build for all configured platforms:

```bash
bundle exec rake native
```

### Using rb-sys-dock Directly

For more control, use rb-sys-dock directly:

```bash
# Build for a specific platform
bundle exec rb-sys-dock --platform x86_64-linux --build

# Start a shell in the Docker container
bundle exec rb-sys-dock --platform x86_64-linux --shell
```

## Testing Cross-Platform Builds

### Local Testing with Docker

You can test your cross-compiled Linux extensions locally:

```bash
# Run tests inside a Docker container
bundle exec rb-sys-dock --platform x86_64-linux --command "bundle exec rake test"
```

### Local Testing on macOS

If you're on macOS with Apple Silicon, you can test both architectures:

```bash
# Test arm64-darwin build (native)
bundle exec rake test

# Test x86_64-darwin build (cross-compiled)
arch -x86_64 bundle exec rake test
```

## CI/CD for Multiple Platforms

GitHub Actions is ideal for testing across platforms:

### Testing on Multiple Platforms

```yaml
# .github/workflows/test.yml
name: Test

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        ruby: ["3.0", "3.1", "3.2", "3.3"]

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4
      - uses: oxidize-rb/actions/setup-ruby-and-rust@v1
        with:
          ruby-version: ${{ matrix.ruby }}
          bundler-cache: true
      - run: bundle exec rake compile
      - run: bundle exec rake test
```

### Ruby-Head Compatibility

When supporting `ruby-head` or development versions of Ruby, you must publish a source gem alongside your precompiled gems. This is necessary because:

1. The Ruby ABI (Application Binary Interface) can change between development versions
2. Precompiled binary gems built against one ruby-head version may be incompatible with newer ruby-head versions
3. Source gems allow users to compile the extension against their specific ruby-head version

To ensure compatibility, add a source gem to your release process:

```ruby
# Rakefile
RbSys::ExtensionTask.new("my_gem", GEMSPEC) do |ext|
  # Configure cross-platform gems as usual
  ext.cross_compile = true
  ext.cross_platform = ['x86_64-linux', 'arm64-darwin', ...]

  # The default platform will build the source gem
end
```

Then in your CI/CD pipeline, include both platform-specific and source gem builds:

```yaml
# .github/workflows/release.yml
jobs:
  # First build all platform-specific gems
  cross_compile:
    # ...

  # Then build the source gem
  source_gem:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: ruby/setup-ruby@v1
        with:
          ruby-version: "3.3"
      - run: bundle install
      - run: bundle exec rake build # Builds the source gem
      - uses: actions/upload-artifact@v3
        with:
          name: source-gem
          path: pkg/*.gem # Include source gem without platform suffix
```

### Cross-Compiling for Release

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags: ["v*"]

jobs:
  cross_compile:
    strategy:
      fail-fast: false
      matrix:
        platform: ["x86_64-linux", "aarch64-linux", "x86_64-darwin", "arm64-darwin", "x64-mingw-ucrt"]

    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4
      - uses: ruby/setup-ruby@v1
        with:
          ruby-version: "3.1"
      - uses: oxidize-rb/actions/cross-gem@v1
        with:
          platform: ${{ matrix.platform }}
      - uses: actions/upload-artifact@v3
        with:
          name: gem-${{ matrix.platform }}
          path: pkg/*-${{ matrix.platform }}.gem
```

### Complete CI Workflow Example

Here's a more complete workflow showing an automated release process with tests and cross-compilation:

```yaml
# .github/workflows/gem-release.yml
name: Gem Release

on:
  push:
    tags:
      - "v*"

jobs:
  fetch-data:
    runs-on: ubuntu-latest
    outputs:
      platforms: ${{ steps.fetch.outputs.supported-ruby-platforms }}
    steps:
      - id: fetch
        uses: oxidize-rb/actions/fetch-ci-data@v1
        with:
          supported-ruby-platforms: |
            exclude: [x86-linux, x86-darwin, arm-linux]

  test:
    needs: fetch-data
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        ruby: ["3.0", "3.1", "3.2", "3.3"]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: oxidize-rb/actions/setup-ruby-and-rust@v1
        with:
          ruby-version: ${{ matrix.ruby }}
          bundler-cache: true
      - run: bundle exec rake compile
      - run: bundle exec rake test

  cross-compile:
    needs: [fetch-data, test]
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        platform: ${{ fromJSON(needs.fetch-data.outputs.platforms) }}
    steps:
      - uses: actions/checkout@v4
      - uses: ruby/setup-ruby@v1
        with:
          ruby-version: "3.1"
      - uses: oxidize-rb/actions/cross-gem@v1
        with:
          platform: ${{ matrix.platform }}
      - uses: actions/upload-artifact@v3
        with:
          name: gem-${{ matrix.platform }}
          path: pkg/*-${{ matrix.platform }}.gem

  release:
    needs: cross-compile
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: ruby/setup-ruby@v1
        with:
          ruby-version: "3.1"
      - uses: actions/download-artifact@v3
        with:
          path: artifacts
      - name: Move gems to pkg directory
        run: |
          mkdir -p pkg
          find artifacts -name "*.gem" -exec mv {} pkg/ \;
      - name: Publish to RubyGems
        run: |
          mkdir -p ~/.gem
          echo -e "---\n:rubygems_api_key: ${RUBYGEMS_API_KEY}" > ~/.gem/credentials
          chmod 0600 ~/.gem/credentials
          gem push pkg/*.gem
        env:
          RUBYGEMS_API_KEY: ${{ secrets.RUBYGEMS_API_KEY }}
```

## Platform-Specific Issues and Solutions

### Windows

Windows presents unique challenges for Ruby extensions:

- **Path Handling**: Use forward slashes (`/`) in paths, not backslashes (`\`)

  ```rust
  // Instead of this:
  let path = "C:\\Users\\Name\\file.txt";

  // Do this:
  let path = "C:/Users/Name/file.txt";
  ```

- **DLL Loading**: Handle DLL loading carefully

  ```rust
  #[cfg(target_os = "windows")]
  fn load_library(name: &str) -> Result<(), Error> {
      use std::os::windows::ffi::OsStrExt;
      use std::ffi::OsStr;
      use winapi::um::libloaderapi::LoadLibraryW;

      let name_wide: Vec<u16> = OsStr::new(name)
          .encode_wide()
          .chain(std::iter::once(0))
          .collect();

      let handle = unsafe { LoadLibraryW(name_wide.as_ptr()) };
      if handle.is_null() {
          return Err(Error::new("Failed to load library"));
      }

      Ok(())
  }
  ```

- **Asynchronous I/O**: Windows has different async I/O APIs

  ```rust
  #[cfg(target_os = "windows")]
  use windows_specific_io::read_file;

  #[cfg(not(target_os = "windows"))]
  use posix_specific_io::read_file;
  ```

### macOS

- **Architectures**: Support both Intel and Apple Silicon

  ```ruby
  # Rakefile
  RbSys::ExtensionTask.new("my_gem", GEMSPEC) do |ext|
    ext.cross_platform = ["x86_64-darwin", "arm64-darwin"]
  end
  ```

- **Framework Linking**: Link against macOS frameworks

  ```rust
  // build.rs
  #[cfg(target_os = "macos")]
  {
      println!("cargo:rustc-link-lib=framework=Security");
      println!("cargo:rustc-link-lib=framework=CoreFoundation");
  }
  ```

- **Universal Binary**: Consider building universal binaries
  ```ruby
  # extconf.rb
  if RUBY_PLATFORM =~ /darwin/
    ENV['RUSTFLAGS'] = "-C link-arg=-arch -C link-arg=arm64 -C link-arg=-arch -C link-arg=x86_64"
  end
  ```

### Linux

- **glibc vs musl**: Consider both glibc and musl for maximum compatibility

  ```ruby
  # Rakefile
  RbSys::ExtensionTask.new("my_gem", GEMSPEC) do |ext|
    ext.cross_platform = ["x86_64-linux", "x86_64-linux-musl"]
  end
  ```

- **Static Linking**: Increase portability with static linking

  ```toml
  # Cargo.toml
  [target.'cfg(target_os = "linux")'.dependencies]
  openssl-sys = { version = "0.9", features = ["vendored"] }
  ```

- **Multiple Distributions**: Test on different distributions in CI
  ```yaml
  # .github/workflows/linux-test.yml
  jobs:
    test:
      strategy:
        matrix:
          container: ["ubuntu:20.04", "debian:bullseye", "alpine:3.15"]
      container: ${{ matrix.container }}
  ```

## Best Practices

1. **Start with cross-compilation early** - Don't wait until release time
2. **Test on all target platforms** - Ideally in CI
3. **Use platform-specific code sparingly** - Abstract platform differences when possible
4. **Prefer conditional compilation over runtime checks** - Better performance and safer code
5. **Document platform requirements** - Make dependencies clear to users
6. **Use feature flags for optional platform support** - Allow users to opt-in to platform-specific features

### Example: Good Platform Abstraction

```rust
// Platform abstraction module
mod platform {
    pub struct FileHandle(PlatformSpecificHandle);

    impl FileHandle {
        pub fn open(path: &str) -> Result<Self, Error> {
            #[cfg(target_os = "windows")]
            {
                // Windows-specific implementation
                // ...
            }

            #[cfg(unix)]
            {
                // Unix-based implementation (Linux, macOS, etc.)
                // ...
            }

            #[cfg(not(any(target_os = "windows", unix)))]
            {
                return Err(Error::new("Unsupported platform"));
            }
        }

        pub fn read(&self, buf: &mut [u8]) -> Result<usize, Error> {
            // Platform-specific reading implementation
            // ...
        }

        pub fn write(&self, buf: &[u8]) -> Result<usize, Error> {
            // Platform-specific writing implementation
            // ...
        }
    }
}

// User code just uses the abstraction
use platform::FileHandle;

fn process_file(path: &str) -> Result<(), Error> {
    let file = FileHandle::open(path)?;
    // Common code without platform-specific details
    Ok(())
}
```

## Complete Example: Cross-Platform Release Workflow

Here's a complete example for releasing a cross-platform gem:

1. **Develop locally** on your preferred platform
2. **Test your changes** locally with `bundle exec rake test`
3. **Verify cross-platform builds** with
   `bundle exec rb-sys-dock --platform x86_64-linux --command "bundle exec rake test"`
4. **Commit and push** your changes
5. **CI tests** run on all supported platforms
6. **Create a release tag** when ready (`git tag v1.0.0 && git push --tags`)
7. **Cross-compilation workflow** builds platform-specific gems
8. **Publish gems** to RubyGems or your private repository

By following this workflow, you can be confident your extension works consistently across platforms.

## Real-World Examples

Many real-world gems use rb-sys for cross-platform development:

- [blake3-ruby](https://github.com/oxidize-rb/blake3-ruby) - Fast cryptographic hash function implementation with full
  cross-platform support
- [lz4-ruby](https://github.com/yoshoku/lz4-ruby) - LZ4 compression library with rb-sys
- [wasmtime-rb](https://github.com/bytecodealliance/wasmtime-rb) - WebAssembly runtime

These projects demonstrate successful cross-platform strategies and can serve as references for your own extensions.

### Example from wasmtime-rb

wasmtime-rb wraps platform-specific functionality while presenting a consistent API:

```rust
#[cfg(unix)]
mod unix {
    pub unsafe fn map_memory(addr: *mut u8, len: usize) -> Result<(), Error> {
        // Unix-specific memory mapping
    }
}

#[cfg(windows)]
mod windows {
    pub unsafe fn map_memory(addr: *mut u8, len: usize) -> Result<(), Error> {
        // Windows-specific memory mapping
    }
}

// Public API uses the platform-specific implementation
pub unsafe fn map_memory(addr: *mut u8, len: usize) -> Result<(), Error> {
    #[cfg(unix)]
    {
        return unix::map_memory(addr, len);
    }

    #[cfg(windows)]
    {
        return windows::map_memory(addr, len);
    }

    #[cfg(not(any(unix, windows)))]
    {
        return Err(Error::new("Unsupported platform"));
    }
}
```

## Summary

Cross-platform development with rb-sys leverages Rust's excellent platform-specific features:

1. **Conditional compilation** provides platform-specific code paths
2. **Platform-specific dependencies** allow different libraries per platform
3. **rb-sys-dock** enables easy cross-compilation for multiple platforms
4. **GitHub Actions integration** automates testing and releases

By following the patterns in this chapter, your Ruby extensions can work seamlessly across all major platforms while
minimizing platform-specific code and maintenance burden.

## Next Steps

- Visit [Build Process](build-process.md) to see local compilation details.
- Check out [Testing Extensions](testing.md) for CI workflows across platforms.
- Use [Debugging](debugging.md) strategies when cross-compiling fails.
- Review [Project Setup](project-setup.md) to organize multi-platform gems.
