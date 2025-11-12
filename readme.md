# rb-sys

[![.github/workflows/ci.yml](https://github.com/oxidize-rb/rb-sys/actions/workflows/ci.yml/badge.svg)](https://github.com/oxidize-rb/rb-sys/actions/workflows/ci.yml)
[![Join the discussion](https://img.shields.io/badge/slack-chat-blue.svg)](https://join.slack.com/t/oxidize-rb/shared_invite/zt-16zv5tqte-Vi7WfzxCesdo2TqF_RYBCw)
![Crates.io](https://img.shields.io/crates/v/rb-sys?style=flat) ![Gem](https://img.shields.io/gem/v/rb_sys?style=flat)

The primary goal of `rb-sys` is to make building native Ruby extensions in Rust **easier** than it would be in C. If
it's not easy, it's a bug.

## Documentation

For comprehensive documentation, please refer to the [Ruby on Rust Book](https://oxidize-rb.github.io/rb-sys/), which
covers:

- Getting started and quick start tutorials
- Working with Ruby objects, classes, and modules
- Memory management and safety
- Cross-platform development
- Testing and debugging
- API reference for rb-sys crate and rb_sys gem

## Components

- **rb-sys crate**: Battle-tested Rust bindings for the Ruby C API
- **rb_sys gem**: Simplifies compiling Rust code into Ruby extensions
- **rb-sys-test-helpers**: Utilities for testing Ruby extensions from Rust
- **rb-sys-dock**: Docker-based cross-compilation tooling

## Supported Toolchains

- Ruby: <!-- toolchains .policy.minimum-supported-ruby-version -->2.7<!-- /toolchains -->+
- Rust: <!-- toolchains .policy.minimum-supported-rust-version -->1.71<!-- /toolchains -->+

## Real-World Examples

- [oxi-test](https://github.com/oxidize-rb/oxi-test) - Canonical example of rb-sys usage (minimal, tested,
  cross-compiled)
- [blake3-ruby](https://github.com/oxidize-rb/blake3-ruby) - Fast cryptographic hash function
- [wasmtime-rb](https://github.com/bytecodealliance/wasmtime-rb) - WebAssembly runtime with rb-sys and Magnus
- [lz4-ruby](https://github.com/yoshoku/lz4-ruby) - LZ4 compression library

## Getting Help

- Join the [Oxidize Ruby Slack](https://join.slack.com/t/oxidize-rb/shared_invite/zt-16zv5tqte-Vi7WfzxCesdo2TqF_RYBCw)
  and post in the `#general` channel
- [Open an issue](https://github.com/oxidize-rb/rb-sys/issues) on GitHub

## Contributing

See the [CONTRIBUTING.md](./CONTRIBUTING.md) file for information about setting up a development environment.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
