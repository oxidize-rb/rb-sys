# The `rb_sys` gem

The `rb_sys` gem is a Ruby gem makes it easy to build native Ruby extensions in Rust. It interops with the existing Ruby
native extension toolchains (i.e. `rake-compiler`) to make testing, building, and cross compilation of gems easy.

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
  # (optional)
  r.profile = ENV.fetch("RB_SYS_CARGO_PROFILE", :dev).to_sym

  # Can be overridden with `RB_SYS_CARGO_FEATURES` env var
  # (optional)
  r.features = ["test-feature"]

  # You can add whatever env vars you want to the env hash
  # (optional)
  r.env = {"FOO" => "BAR"}

  # If your Cargo.toml is in a different directory, you can specify it here
  # (optional)
  r.ext_dir = "."
end
```
