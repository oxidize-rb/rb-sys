# The `rb_sys` gem

![Gem](https://img.shields.io/gem/v/rb_sys)
[![Documentation](https://img.shields.io/badge/docs-rdoc.info-blue.svg)](https://www.rubydoc.info/gems/rb_sys/frames)
[![Join the discussion](https://img.shields.io/badge/slack-chat-blue.svg)](https://join.slack.com/t/oxidize-rb/shared_invite/zt-16zv5tqte-Vi7WfzxCesdo2TqF_RYBCw)

The `rb_sys` gem makes it easy to build native Ruby extensions in Rust. It interoperates with existing Ruby native extension toolchains (i.e. `rake-compiler`) to make testing, building, and cross-compilation of gems easy.

## Documentation

For comprehensive documentation, please refer to the [Ruby on Rust Book](https://oxidize-rb.github.io/rb-sys/), which includes:

- [API Reference for rb_sys Gem Configuration](https://oxidize-rb.github.io/rb-sys/api-reference/rb-sys-gem-config.html)
- [The Build Process](https://oxidize-rb.github.io/rb-sys/build-process.html) 
- [Cross-Platform Development](https://oxidize-rb.github.io/rb-sys/cross-platform.html)

## Basic Usage

```ruby
# Rakefile
require "rb_sys/extensiontask"

GEMSPEC = Gem::Specification.load("my_gem.gemspec")

RbSys::ExtensionTask.new("my-crate-name", GEMSPEC) do |ext|
  ext.lib_dir = "lib/my_gem"
  ext.cross_compile = true  # For rb-sys-dock cross-compilation
end
```

```ruby
# ext/my_gem/extconf.rb
require "mkmf"
require "rb_sys/mkmf"

create_rust_makefile("my_gem")
```

For full configuration options and more advanced usage, see the [rb_sys Gem Configuration](https://oxidize-rb.github.io/rb-sys/api-reference/rb-sys-gem-config.html) reference.