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
