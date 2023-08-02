# `rb-sys`

[![.github/workflows/ci.yml](https://github.com/oxidize-rb/rb-sys/actions/workflows/ci.yml/badge.svg)](https://github.com/oxidize-rb/rb-sys/actions/workflows/ci.yml)
[![Join the discussion](https://img.shields.io/badge/slack-chat-blue.svg)][slack]
![Crates.io](https://img.shields.io/crates/v/rb-sys?style=flat) ![Gem](https://img.shields.io/gem/v/rb_sys?style=flat)

The primary goal of `rb-sys` is to make building native Ruby extensions in Rust **easier** than it would be in C. If
it's not easy, it's a bug.

- [Battle-tested **Ruby FFI bindings** ](./crates/rb-sys/readme.md) for Rust (via `rb-sys` crate)
- [Ruby gem for **compiling extensions**](./gem/README.md)
- [GitHub action][setup-action] to **setup a test environment in CI**
- [GitHub action][cross-gem-action] to easily **cross compile in CI**
- [**Test helpers**)][test-helpers] for testing Ruby extensions in Rust

## Features

- Battle-tested Rust bindings for [the Ruby C API][ruby-c-api]
- Support for Ruby 2.4+
- Supports all major platforms (Linux, macOS, Windows)
- Cross compilation of gems
- Integration with [`rake-compiler`][rake-compiler]

## Usage

Below are some examples of how to use `rb-sys` to build native Rust extensions. Use these as a starting point for your
building your own gem.

- The [`wasmtime-rb`][wasmtime-rb] gem uses `rb-sys` and [`magnus`][magnus] to package the rust [`wasmtime`][wasmtime]
  library as a Ruby gem.
- The [`oxi-test` gem][oxi-test] is the canonical example of how to use `rb-sys`. It is a minimal, fully tested with
  GitHub actions, and cross-compiles native gem binaries. This should be your first stop for learning how to use
  `rb-sys`.
- [Docs for the `rb_sys` gem][rb-sys-gem-docs] and using it with an `extconf.rb` file.
- The [`magnus` repo has some solid examples][magnus-examples].
- This [demo repository][rust-talk] that @ianks made for a talk has a gem which has 4 native extensions in the `ext`
  directory.
- This [PR for the `yrb` gem][yrb] shows how to integrate `rb-sys` and [`magnus`][magnus] into an existing gem.
- A [guide for setting debug breakpoints in VSCode][debugging-guide] is available.

## Supported Toolchains

- Ruby: <!--toolchains .policy.minimum-supported-ruby-version -->2.4<!--/toolchains-->+ (for full compatibility with
  Rubygems)
- Rust: <!--toolchains .policy.minimum-supported-rust-version -->1.60<!--/toolchains-->+

## Supported Platforms

We support cross compilation to the following platforms (this information is also available in the [`./data`](./data)
directory for automation purposes):

| Platform          | Supported | Docker Image                                   |
| ----------------- | --------- | ---------------------------------------------- |
| x86_64-linux      | ✅         | [`rbsys/x86_64-linux:0.9.79`][docker-hub]      |
| x86_64-linux-musl | ✅         | [`rbsys/x86_64-linux-musl:0.9.79`][docker-hub] |
| aarch64-linux     | ✅         | [`rbsys/aarch64-linux:0.9.79`][docker-hub]     |
| arm-linux         | ✅         | [`rbsys/arm-linux:0.9.79`][docker-hub]         |
| arm64-darwin      | ✅         | [`rbsys/arm64-darwin:0.9.79`][docker-hub]      |
| x64-mingw32       | ✅         | [`rbsys/x64-mingw32:0.9.79`][docker-hub]       |
| x64-mingw-ucrt    | ✅         | [`rbsys/x64-mingw-ucrt:0.9.79`][docker-hub]    |
| mswin             | ✅         | not available on Docker                        |

## Getting Help

We make a concerted effort to help out new users. If you have questions, please join our [Slack][slack] and post your
question in the `#general` channel. Alternatively, you can [open an issue][issues] and we'll try to help you out.

## Contributing

See the [CONTRIBUTING.md](./CONTRIBUTING.md) file for information about setting up a development environment.

Bug reports and pull requests are welcome on GitHub at https://github.com/oxidize-rb/rb-sys.

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[docker-hub]: https://hub.docker.com/r/rbsys/rcd
[magnus]: https://github.com/matsadler/magnus
[yrb]: https://github.com/y-crdt/yrb/pull/32/files
[rust-talk]: https://github.com/ianks/2022-09-09-ruby-on-rust-intro
[oxi-test]: https://github.com/oxidize-rb/oxi-test
[cross-gem-action]: https://github.com/oxidize-rb/actions/blob/main/cross-gem/readme.md
[rake-compiler]: https://github.com/rake-compiler/rake-compiler
[setup-action]: https://github.com/oxidize-rb/actions/tree/main/setup-ruby-and-rust
[ruby-c-api]: https://docs.ruby-lang.org/en/master/doc/extension_rdoc.html
[slack]: https://join.slack.com/t/oxidize-rb/shared_invite/zt-16zv5tqte-Vi7WfzxCesdo2TqF_RYBCw
[issues]: https://github.com/oxidize-rb/rb-sys/issues
[magnus-examples]: https://github.com/matsadler/magnus/tree/main/examples
[debugging-guide]: https://oxidize-rb.github.io/rb-sys/tutorial/testing/debugging.html
[rb-sys-gem-docs]: https://github.com/oxidize-rb/rb-sys/tree/main/gem#the-rb_sys-gem
[wasmtime-rb]: https://github.com/bytecodealliance/wasmtime-rb
[wasmtime]: https://github.com/bytecodealliance/wasmtime
[test-helpers]: ./crates/rb-sys-test-helpers/readme.md
