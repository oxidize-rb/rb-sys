# Cross-Compilation

Publishing native gem binaries is incredibly important for Ruby on Rust gems. No one likes seeing the infamous
`Compiling native extensions. This could take a while...` message when they install a gem. And in Rust, we all know that
compiling can take a while...

It's important to make sure that your gem is as fast as possible to install, that's why `rb-sys` is built from the ground
up to support this use-case. `rb-sys` integrates seamlessly with [`rake-compiler`][rake-compiler] and
[`rake-compiler-dock`][rcd]. By leveraging the hard-work of others, cross-compilation for Ruby gems is as simple and
reliable as it would be for a C extension.

> **💡 Tip:** [Join the Slack channel][slack] to ask questions and get help from the community!

## Using the `rb-sys-dock` helper

The `rb-sys-dock` executable allows you to easily enter the Docker container used to cross compile your gem. You can use
your tool to build your gem, and then exit the container. The gem will be available in the `pkg` directory.

```bash
$ bundle exec rb-sys-dock -p aarch64-linux --build
$ ls pkg # => my_gem_name-0.1.0-aarch64-linux.gem
```

## GitHub Actions

The [`oxi-test`][oxi-test] gem is meant to serve as the canonical example of how to setup cross gem compilation. Here's
a walkthrough of the important files to reference:

1. Setup the `Rake::ExtensionTask` in the [`Rakefile`](https://github.com/oxidize-rb/oxi-test/blob/main/Rakefile)
2. Setup a [`cross-gem.yml`](https://github.com/oxidize-rb/oxi-test/blob/main/.github/workflows/cross-gem.yml) GitHub
   action to build the gem for multiple platforms.
3. Download the [`cross-gem` artifacts ](https://github.com/oxidize-rb/oxi-test/actions/runs/3348359067) from the GitHub
   action and test them out.

## In the wild

- [`wasmtime-rb`](https://github.com/bytecodealliance/wasmtime-rb)
- [`yrb`](https://github.com/y-crdt/yrb)
- [`commonmarker`](https://github.com/gjtorikian/commonmarker)

> **💡 Tip:** Add your gem to this list by opening a PR!

## Resources

- [Cross Gem Action](https://github.com/oxidize-rb/actions/blob/main/cross-gem/readme.md) to easily cross compile with
  GitHub actions
- [Docker images](https://index.docker.io/u/rbsys)

[rake-compiler]: https://github.com/rake-compiler/rake-compiler
[rcd]: https://github.com/rake-compiler/rake-compiler-dock
[oxi-test]: https://github.com/oxidize-rb/oxi-test
[slack]: https://join.slack.com/t/oxidize-rb/shared_invite/zt-16zv5tqte-Vi7WfzxCesdo2TqF_RYBCw
