# Getting started

## Creating a new gem

The easiest way to create a new gem is to use the `bundle gem` command. This will scaffold a new Rust gem using `rb-sys`
and `magnus`.

1. Install a Rust toolchain (if needed)

   ```bash
   $ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Upgrade RubyGems and Bundler

   ```bash
   $ gem update --system
   ```

3. Scaffold a new gem with a Rust extension

   ```bash
   $ bundle gem --ext=rust my_gem_name
   ```

This will create a new gem in the `my_gem_name` directory. Firstly, open up `my_gem_name.gemspec` in your text editor
and make sure you update all fields that contain `TODO`. Inside the directory, you should now have a fully working Rust
gem.

> **💡 Tip:** [Join the Slack channel][slack] to ask questions and get help from the community!

## Building the gem and running tests

The default Rake task is configured to compile the Rust code and run tests. Simply run:

```
$ bundle exec rake
```

At this point you should start reading the docs for [`magnus`][magnus] to get familiar with the API. It is designed to be
a safe and idiomatic wrapper around the Ruby C API.

## Next steps

- [Learn how to debug Rust code using LLDB](./tutorial/testing/debugging.md)
- [Learn how to publish cross-compiled gems](./tutorial/publishing/cross-compilation.md)

[rb-sys]: https://github.com/oxidize-rb/rb-sys
[magnus]: https://github.com/matsadler/magnus
[slack]: https://join.slack.com/t/oxidize-rb/shared_invite/zt-16zv5tqte-Vi7WfzxCesdo2TqF_RYBCw
