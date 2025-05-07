{{#title Ruby on Rust: Introduction}}

# Introduction

Welcome to the rb-sys guide. This book will show you how to build Ruby extensions in Rust that are both powerful and reliable.

The primary goal of `rb-sys` is to make building native Ruby extensions in Rust **easier** than it would be in C. If it's not easy, it's a bug.

## Key Features

- Battle-tested Rust bindings for the Ruby C API
- Support for Ruby 2.6+
- Support for all major platforms (Linux, macOS, Windows)
- Cross-compilation support for gems
- Integration with `rake-compiler`
- Test helpers for Ruby extensions

## Why Rust for Ruby Extensions?

Ruby extensions have traditionally been written in C, requiring manual memory management and careful handling of Ruby's internals. This approach is error-prone and often results in security vulnerabilities, memory leaks, and crashes.

<div class="note">

While C extensions offer flexibility and minimal dependencies, Rust extensions provide a superior developer experience with improved safety guarantees and access to a rich ecosystem of libraries.

</div>

Rust offers a compelling alternative with several advantages:

- **Memory safety without garbage collection**
- **Strong type system that catches errors at compile time**
- **Modern language features like pattern matching and traits**
- **Access to the vast Rust ecosystem via crates.io**
- **Strong tooling for testing, documentation, and deployment**

<div class="warning">

Importantly, performance should not be the sole motivation for using Rust. With Ruby's YJIT compiler, pure Ruby code is now faster than ever. Instead, consider Rust when you need memory safety, type safety, or want to leverage the rich Rust ecosystem's capabilities.

</div>

## The rb-sys Project Ecosystem

rb-sys consists of several components working together:

1. **rb-sys crate**: Provides low-level Rust bindings to Ruby's C API
2. **rb_sys gem**: Handles the Ruby side of extension compilation
3. **Magnus**: A higher-level, ergonomic API for Rust/Ruby interoperability
4. **rb-sys-dock**: Docker-based cross-compilation tooling
5. **GitHub Actions**: Setup and cross-compilation automation for CI workflows 

<div class="tip">

Most developers will use the Magnus API when building their extensions, as it provides a much more ergonomic interface than using rb-sys directly.

</div>

Here's how these components interact when building a typical Ruby gem with Rust:

```diagram,hidelines=~
Your Ruby Code (.rb files)
       ↓
  Your Rust Code (.rs files)
       ↓
~   Magnus API
       ↓
~  rb-sys crate
       ↓
~Ruby C API Bindings
       ↓
~  Ruby VM
```

During compilation:

```diagram,hidelines=~
~  Your gem's extconf.rb
       ↓
~  rb_sys gem's create_rust_makefile
       ↓
~  Cargo build process using rb-sys crate
       ↓
~  Native extension (.so/.bundle/.dll)
```

You can click the eye icon (<i class="fa fa-eye"></i>) to see the hidden details in these diagrams.

## Comparison with Traditional C Extensions

Let's compare writing extensions in Rust versus C:

| Aspect | C Extensions | Rust Extensions |
|--------|-------------|----------------|
| **Memory Safety** | Manual memory management | Guaranteed memory safety at compile time |
| **Type Safety** | Weak typing, runtime errors | Strong static typing, compile-time checks |
| **API Ergonomics** | Low-level C API | High-level Magnus API |
| **Development Speed** | Slower, more error-prone | Faster, safer development cycle |
| **Ecosystem Access** | Limited to C libraries | Full access to Rust crates |
| **Debugging** | Harder to debug memory issues | Easier to debug with Rust's safety guarantees |
| **Cross-Compilation** | Complex manual configuration | Simplified with rb-sys-dock |

While C extensions offer flexibility and minimal dependencies, Rust extensions provide a superior developer experience with improved safety guarantees and access to a rich ecosystem of libraries.

## Real-World Examples

These gems demonstrate rb-sys in action:

- [lz4-ruby](https://github.com/yoshoku/lz4-ruby) - LZ4 compression library with rb-sys
- [wasmtime-rb](https://github.com/bytecodealliance/wasmtime-rb) - WebAssembly runtime with rb-sys and Magnus
- [oxi-test](https://github.com/oxidize-rb/oxi-test) - Canonical example of how to use rb-sys (minimal, fully tested, cross-compiled)
- [blake3-ruby](https://github.com/oxidize-rb/blake3-ruby) - Fast cryptographic hash function with full cross-platform support

## Supported Toolchains

- Ruby: 2.6+ (for full compatibility with Rubygems)
- Rust: 1.65+

## Dependencies

To build a Ruby extension in Rust, you'll need:

- Ruby development headers (usually part of ruby-dev packages)
- Rust (via rustup)
- libclang (for bindgen)
  - On macOS: `brew install llvm`
  - On Linux: `apt-get install libclang-dev`

## Getting Help

If you have questions, please join our [Slack channel][chat] or [open an issue on GitHub][issues].

## Contributing to this book

This book is open source! Find a typo? Did we overlook something? [**Send us a pull request!**][repo]. Help wanted!

## License

rb-sys is licensed under either:

- Apache License, Version 2.0
- MIT license

at your option.

## Next Steps

- Proceed to [Getting Started](getting-started.md) to set up your development environment.
- Try the [Quick Start](quick-start.md) to build your first extension.
- Explore core concepts in [Build Process](build-process.md) and [Memory Management & Safety](memory-management.md).
- Learn advanced topics like [Cross-Platform Development](cross-platform.md) and [Testing Extensions](testing.md).

[repo]: https://github.com/oxidize-rb/rb-sys
[chat]: https://join.slack.com/t/oxidize-rb/shared_invite/zt-16zv5tqte-Vi7WfzxCesdo2TqF_RYBCw
[issues]: https://github.com/oxidize-rb/rb-sys/issues