# Prerequisites and Installation

This chapter provides a streamlined setup guide for building Ruby extensions with Rust.

## TL;DR

### 1. Install Prerequisites

```bash
# Install Ruby (3.0+ recommended)
# Using your preferred manager: rbenv, rvm, asdf, etc.

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install rb_sys gem
gem install rb_sys
```

### 2. Create a New Gem with Rust Extension

```bash
# Generate a new gem with Rust extension support
bundle gem --ext=rust mygem
cd mygem

# Build the extension
bundle install
bundle exec rake compile

# Try it out
bundle exec rake
```

That's it! You now have a working Ruby gem with Rust extension.

## Detailed Installation

If you encounter issues with the quick start above, here are the detailed requirements:

### Ruby Requirements

- Ruby 3.0+ recommended (2.6+ supported)
- Ruby development headers (usually part of official packages)
- Bundler (`gem install bundler`)

### Rust Requirements

- Rust 1.65.0+ via [rustup](https://rustup.rs/)
- Make sure Cargo is in your PATH (typically `~/.cargo/bin`)

### C Compiler Requirements

- **macOS**: Xcode Command Line Tools (`xcode-select --install`)
- **Linux**: build-essential (Debian/Ubuntu) or Development Tools (Fedora/RHEL)
- **Windows**: Microsoft Visual Studio C++ Build Tools

### libclang (for Ruby/Rust FFI bindings)

Simplest approach: add to your Gemfile
```ruby
gem "libclang", "~> 14.0"
```

## Verifying Your Setup

The simplest way to verify your setup is to create a test gem:

```bash
bundle gem --ext=rust hello_rusty
cd hello_rusty
bundle install
bundle exec rake compile
bundle exec rake test
```

If everything runs without errors, your environment is correctly set up.

## Troubleshooting

If you encounter issues:

1. **Missing libclang**: Add the `libclang` gem to your Gemfile
2. **Missing C compiler**: Install appropriate build tools for your platform
3. **Ruby headers not found**: Install Ruby development package

For detailed troubleshooting, consult the [rb-sys wiki](https://github.com/oxidize-rb/rb-sys/wiki/Troubleshooting).

## Next Steps

- Validate your setup with the [Quick Start](quick-start.md).
- Dive into [Build Process](build-process.md) for deeper compilation insights.
- Explore [Project Setup](project-setup.md) patterns.
- Learn [Testing Extensions](testing.md) to add tests.