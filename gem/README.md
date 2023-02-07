# The `rb_sys` gem

![Gem](https://img.shields.io/gem/v/rb_sys)
[![Documentation](https://img.shields.io/badge/docs-rdoc.info-blue.svg)](https://www.rubydoc.info/gems/rb_sys/frames)
[![Join the discussion](https://img.shields.io/badge/slack-chat-blue.svg)](https://join.slack.com/t/oxidize-rb/shared_invite/zt-16zv5tqte-Vi7WfzxCesdo2TqF_RYBCw)

The `rb_sys` gem is a Ruby gem makes it easy to build native Ruby extensions in Rust. It interops with the existing Ruby
native extension toolchains (i.e. `rake-compiler`) to make testing, building, and cross compilation of gems easy.

## `RbSys::ExtensionTask`

This gem provides a `RbSys::ExtensionTask` class that can be used to build a Ruby extension in Rust. It's a thin wrapper
around `Rake::ExtensionTask` that provides sane defaults for building Rust extensions.

```ruby
# Rakefile

require "rb_sys/extensiontask"

GEMSPEC = Gem::Specification.load("my_gem.gemspec")

RbSys::ExtensionTask.new("my-crate-name", GEMSPEC) do |ext|
  ext.lib_dir = "lib/my_gem"
end
```

## `create_rust_makefile`

This gem provides a simple helper to build a Ruby compatible Makefile for you Rust extension. For a full example, see
the [examples](./../examples) directory.

```ruby
# ext/rust_reverse/extconf.rb

# We need to require mkmf *first* this since `rake-compiler` injects code here for cross compilation
require "mkmf"
require "rb_sys/mkmf"

create_rust_makefile("rust_reverse") do |r|
  # Create debug builds in dev. Make sure that release gems are compiled with
  # `RB_SYS_CARGO_PROFILE=release` (optional)
  r.profile = ENV.fetch("RB_SYS_CARGO_PROFILE", :dev).to_sym

  # Can be overridden with `RB_SYS_CARGO_FEATURES` env var (optional)
  r.features = ["test-feature"]

  # You can add whatever env vars you want to the env hash (optional)
  r.env = {"FOO" => "BAR"}

  # If your Cargo.toml is in a different directory, you can specify it here (optional)
  r.ext_dir = "."

  # Extra flags to pass to the $RUSTFLAGS environment variable (optional)
  r.extra_rustflags = ["--cfg=some_nested_config_var_for_crate"]

  # Force a rust toolchain to be installed via rustup (optional)
  # You can also set the env var `RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN=true`
  r.force_install_rust_toolchain = "stable"

  # Clean up the target/ dir after `gem install` to reduce bloat (optional)
  r.clean_after_install = false # default: true if invoked by rubygems

  # Auto-install Rust toolchain if not present on "gem install" (optional)
  r.auto_install_rust_toolchain = false # default: true if invoked by rubygems
end
```

## Tips and Tricks

- When using `rake-compiler` to build your gem, you can use the `RB_SYS_CARGO_PROFILE` environment variable to set the
  Cargo profile (i.e. `release` or `dev`).

- You can pass Cargo arguments to `rake-compiler` like so: `rake compile -- --verbose`

- It's possible to force an installation of a Rust toolchain by setting the `RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN`
  environment variable. This will install [rustup](https://rustup.rs/) and [cargo](https://crates.io/) in the build
  directory, so the end user does not have to have Rust pre-installed. Ideally, this should be a last resort, as it's
  better to already have the toolchain installed on your system.

## Troubleshooting

### Libclang issues

If you see an error like this:

```
thread 'main' panicked at 'Unable to find libclang: "couldn't find any valid shared libraries matching: \['libclang.so', 'libclang-*.so', 'libclang.so.*', 'libclang-*.so.*'\], set the `LIBCLANG_PATH` environment variable to a path where one of these files can be found (invalid: \[\])"'
```

This means that bindgen is having issues finding a usable version of libclang. An easy way to fix this is to install the
[`libclang` gem](https://github.com/oxidize-rb/libclang-rb), which will install a pre-built version of libclang for you.
`rb_sys` will automatically detect this gem and use it.

```ruby
# Gemfile

gem "libclang", "~> 14.0.6"
```
