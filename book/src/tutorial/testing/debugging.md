# Debugging

To debug Rust extensions, you can use either LLDB or GDB. First, you will need to compile with the `dev` Cargo profile,
so debug symbols are available.

To do that you can run: `RB_SYS_CARGO_PROFILE=dev rake compile`. Alternatively, you can add a helper Rake task to make
this easier:

```ruby
# Rakefile

desc "Compile the extension with debug symbols"
task "compile:debug" do
  ENV["RB_SYS_CARGO_PROFILE"] = "dev"
  Rake::Task["compile"].invoke
end
```

> **💡 Tip:** [Join the Slack channel][slack] to ask questions and get help from the community!

## VSCode + LLDB

The [code-lldb]() extension for VSCode is a great way to debug Rust code. Here is an example configuration file:

```json
// .vscode/launch.json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug",
      "preLaunchTask": {
        "task": "compile:debug",
        "type": "rake"
      },
      "program": "~/.asdf/installs/ruby/3.1.1/bin/ruby",
      "args": ["-Ilib", "test/test_helper.rb"],
      "cwd": "${workspaceFolder}",
      "sourceLanguages": ["rust"]
    }
  ]
}
```

### Debugging the Ruby C API

With this basic setup, you can set breakpoints and interactively debug your Rust code. However, if Ruby is not built
with debug symbols, any calls into the Ruby C API become a black box. Luckily, it's straight-forward to fix this.

#### Compiling Ruby with debug symbols and source code

#### Using [`chruby`](https://github.com/postmodern/chruby) or [`ruby-build`](https://github.com/rbenv/ruby-build)

1. First, compile Ruby like so:

   ```sh
   $ RUBY_CFLAGS="-Og -ggdb" ruby-build --keep 3.1.2 /opt/rubies/3.1.2-debug
   ```

2. Make sure your `.vscode/launch.json` file is configured to use `/opt/rubies/3.1.2-debug/bin/ruby`.

#### Using [`rbenv`](https://github.com/rbenv/rbenv)

1. First, compile Ruby like so:

   ```sh
   $ RUBY_CFLAGS="-Og -ggdb" rbenv install --keep 3.1.2
   ```

2. Make sure your `.vscode/launch.json` file is configured to use `$RBENV_ROOT/versions/3.1.2/bin/ruby`.

[slack]: https://join.slack.com/t/oxidize-rb/shared_invite/zt-16zv5tqte-Vi7WfzxCesdo2TqF_RYBCw
