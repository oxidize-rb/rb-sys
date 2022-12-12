# Cross-Compilation

Publishing native gem binaries is incredibly important for Ruby on Rust gems. No one likes seeing the infamous
`Compiling native extensions. This could take a while...` message when they install a gem. An in Rust, we all know that
compiling can take a while...

It's important to make sure that your gem is as fast as possible to install, that why `rb-sys` is built from the ground
up to support this use-case. `rb-sys` integrates seamlessly with
[`rake-compiler`](https://github.com/rake-compiler/rake-compiler) and
[`rake-compiler-dock`](https://github.com/rake-compiler/rake-compiler). By leveraging the hard-work of others,
cross-compilation for Ruby gems is as simple and reliable as it would be for a C extension.

## Using the `rb-sys-dock` helper

The [`rb-sys-dock`] executable allows you to easily enter the Docker container used to cross compile your gem. You can
use you tool to build your gem, and then exit the container. The gem will be available in the `pkg` directory.

```bash
$ rb-sys-dock -p aarch64-linux
🐳 Entering shell in Docker container "rbsys/aarch64-linux:0.9.49"
#...
$ bundle install
# ...
$ bundle exec rake native:$RUBY_TARGET # builds the gem
#...
```

## Github Actions

The [`oxi-test`] gem is meant to serve as the canonical example of how to setup cross gem compilation. Here's a
walkthrough of the important files to reference:

1. Setup the `Rake::ExtensionTask` in the [`Rakefile`](https://github.com/oxidize-rb/oxi-test/blob/main/Rakefile)
2. Setup a [`cross-gem.yml`](https://github.com/oxidize-rb/oxi-test/blob/main/.github/workflows/cross-gem.yml) GitHub
   action to build the gem for multiple platforms.
3. Download the [`cross-gem` artifacts ](https://github.com/oxidize-rb/oxi-test/actions/runs/3348359067) from the GitHub
   action and test them out.

## In the wild

- [`wasmtime-rb`](https://github.com/bytecodealliance/wasmtime-rb)
- [`yrb`](https://github.com/y-crdt/yrb)
- [`commonmarker`](https://github.com/gjtorikian/commonmarker)

## Resources

- [Cross Gem Action](https://github.com/oxidize-rb/cross-gem-action) to easily cross compile with GitHub actions
- [Docker images](https://index.docker.io/u/rbsys)
