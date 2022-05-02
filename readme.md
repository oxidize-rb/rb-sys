# rb-sys

[![Join the discussion](https://img.shields.io/badge/slack-chat-blue.svg)](https://join.slack.com/t/oxidize-rb/shared_invite/zt-16zv5tqte-Vi7WfzxCesdo2TqF_RYBCw) [![.github/workflows/ci.yml](https://github.com/oxidize-rb/rb-sys/actions/workflows/ci.yml/badge.svg)](https://github.com/oxidize-rb/rb-sys/actions/workflows/ci.yml)

The primary goal of `rb-sys` is to make building native Ruby extensions in Rust **easier** than it would be in C. If it's not easy, it's a bug.

## Features

- [GitHub action](https://github.com/oxidize-rb/cross-gem-action) to easily cross compile in CI
- Integration with [rake-compiler](https://github.com/rake-compiler/rake-compiler)
- Cross compilation of gems
- Auto-generated Rust bindings for libruby classes

## Supported Platforms

We support cross compilation to the following platforms:

| Platform       | Supported |
| -------------- | --------- |
| x86_64-linux   | ✅        |
| aarch64-linux  | ✅        |
| arm-linux      | ✅        |
| x86_64-darwin  | ✅        |
| arm64-darwin   | ✅        |
| x64-mingw32    | ✅        |
| x64-mingw-ucrt | ✅        |
| x86-mingw32    | ❌        |
| x86-linux      | ❌        |

## Usage

Please see the [examples](./examples) to see a full example of how to use `rb-sys`.

## Contributing

See the [CONTRIBUTING.md](./CONTRIBUTING.md) file for information about setting up a development environment.

Bug reports and pull requests are welcome on GitHub at https://github.com/oxidize-rb/rb-sys.

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
