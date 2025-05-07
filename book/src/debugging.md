# Debugging & Troubleshooting

This chapter covers techniques for debugging Rust-based Ruby extensions, common error patterns, and approaches to
solving the most frequent issues.

## Overview

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

## Common Errors and Solutions

### Compilation Errors

#### Missing Ruby Headers

**Error:**

```
fatal error: ruby.h: No such file or directory
#include <ruby.h>
         ^~~~~~~~
compilation terminated.
```

**Solution:**

- Ensure Ruby development headers are installed
- Check that `rb_sys_env::activate()` is being called in your `build.rs`
- Verify that your Ruby installation is accessible to your build environment

#### Incompatible Ruby Version

**Error:**

```
error: failed to run custom build command for `rb-sys v0.9.78`
```

With details mentioning Ruby version compatibility issues.

**Solution:**

- Ensure your rb-sys version is compatible with your Ruby version
- Update rb-sys to the latest version
- Check your build environment's Ruby version with `ruby -v`

#### Linking Errors

**Error:**

```
error: linking with `cc` failed: exit status: 1
... undefined reference to `rb_define_module` ...
```

**Solution:**

- Ensure proper linking configuration in `build.rs`
- Make sure you've called `rb_sys_env::activate()`
- Verify that your Ruby installation is correctly detected

### Runtime Errors

#### Segmentation Faults

Segmentation faults typically occur when accessing memory improperly:

**Common Causes:**

1. Accessing Ruby objects after they've been garbage collected
2. Not protecting Ruby values from garbage collection during C API calls
3. Incorrect use of raw pointers

**Solutions:**

- Use `TypedData` and implement the `mark` method to protect Ruby objects
- Use `rb_gc_guard!` macro when working with raw C API
- Prefer the higher-level Magnus API over raw rb-sys

#### Already Borrowed: BorrowMutError

When using `RefCell` for interior mutability:

**Error:**

```
thread '<unnamed>' panicked at 'already borrowed: BorrowMutError', ...
```

**Solution:**

- Complete all immutable borrows before attempting mutable borrows
- Copy required data out of immutable borrows before borrowing mutably
- See the [RefCell and Interior Mutability](memory-management.md#refcell-and-interior-mutability) section in the Memory
  Management chapter

#### Method Argument Mismatch

**Error:**

```
ArgumentError: wrong number of arguments (given 2, expected 1)
```

**Solution:**

- Check method definitions in your Rust code
- Ensure `function!` and `method!` macros have the correct arity
- Verify Ruby method calls match the defined signatures

#### Type Conversion Failures

**Error:**

```
TypeError: no implicit conversion of Integer into String
```

**Solution:**

- Add proper type checking and conversions in Rust
- Use `try_convert` and handle conversion errors gracefully
- Add explicit type annotations to clarify intent

## Debugging Techniques

### Using Backtraces

Ruby's built-in backtraces can help identify where problems originate:

```ruby
begin
  # Code that might raise an exception
  MyExtension.problematic_method
rescue => e
  puts e.message
  puts e.backtrace
end
```

You can enhance backtraces with the `backtrace` gem:

```ruby
require 'backtrace'
Backtrace.enable_ruby_source_inspect!

begin
  MyExtension.problematic_method
rescue => e
  puts Backtrace.for(e)
end
```

### VSCode + LLDB

The [code-lldb](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb) extension for VSCode is a great
way to debug Rust code. Here is an example configuration file:

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

##### Using [`chruby`](https://github.com/postmodern/chruby) or [`ruby-build`](https://github.com/rbenv/ruby-build)

1. First, compile Ruby like so:

   ```sh
   $ RUBY_CFLAGS="-Og -ggdb" ruby-build --keep 3.1.2 /opt/rubies/3.1.2-debug
   ```

2. Make sure your `.vscode/launch.json` file is configured to use `/opt/rubies/3.1.2-debug/bin/ruby`.

##### Using [`rbenv`](https://github.com/rbenv/rbenv)

1. First, compile Ruby like so:

   ```sh
   $ RUBY_CFLAGS="-Og -ggdb" rbenv install --keep 3.1.2
   ```

2. Make sure your `.vscode/launch.json` file is configured to use `$RBENV_ROOT/versions/3.1.2/bin/ruby`.

### LLDB from the Command Line

LLDB is an excellent tool for debugging Rust extensions from the command line:

1. Compile with debug symbols:

   ```bash
   RUSTFLAGS="-g" bundle exec rake compile
   ```

2. Run Ruby with LLDB:

   ```bash
   lldb -- ruby -I lib -e 'require "my_extension"; MyExtension.method_to_debug'
   ```

3. Set breakpoints and run:

   ```
   (lldb) breakpoint set --name rb_my_method
   (lldb) run
   ```

4. Common LLDB commands:
   - `bt` - Display backtrace
   - `frame variable` - Show local variables
   - `p expression` - Evaluate expression
   - `n` - Step over
   - `s` - Step into
   - `c` - Continue

### GDB for Linux

GDB offers similar capabilities to LLDB on Linux systems:

1. Compile with debug symbols:

   ```bash
   RUSTFLAGS="-g" bundle exec rake compile
   ```

2. Run Ruby with GDB:

   ```bash
   gdb --args ruby -I lib -e 'require "my_extension"; MyExtension.method_to_debug'
   ```

3. Set breakpoints and run:

   ```
   (gdb) break rb_my_method
   (gdb) run
   ```

4. Common GDB commands:
   - `bt` - Display backtrace
   - `info locals` - Show local variables
   - `p expression` - Evaluate expression
   - `n` - Step over
   - `s` - Step into
   - `c` - Continue

### Rust Debugging Statements

Strategic use of Rust's debug facilities can help identify issues:

```rust
// Debug prints only included in debug builds
#[cfg(debug_assertions)]
println!("Debug: counter value = {}", counter);

// More structured logging
use log::{debug, error, info};

fn some_function() -> Result<(), Error> {
    debug!("Entering some_function");

    if let Err(e) = fallible_operation() {
        error!("Operation failed: {}", e);
        return Err(e.into());
    }

    info!("Operation succeeded");
    Ok(())
}
```

To enable logging output, add a logger like `env_logger`:

```rust
fn init(ruby: &Ruby) -> Result<(), Error> {
    env_logger::init();
    // Rest of initialization...
    Ok(())
}
```

And set the log level when running Ruby:

```bash
RUST_LOG=debug ruby -I lib -e 'require "my_extension"'
```

## Memory Leak Detection

### Using ruby_memcheck

The [ruby_memcheck](https://github.com/Shopify/ruby_memcheck) gem helps identify memory leaks in Ruby extensions by
filtering out Ruby's internal memory management noise when running Valgrind.

1. Install dependencies:

   ```bash
   gem install ruby_memcheck
   # On Debian/Ubuntu
   apt-get install valgrind
   ```

2. Set up in your Rakefile:

   ```ruby
   require 'ruby_memcheck'

   test_config = lambda do |t|
     t.libs << "test"
     t.test_files = FileList["test/**/*_test.rb"]
   end

   namespace :test do
     RubyMemcheck::TestTask.new(valgrind: :compile, &test_config)
   end
   ```

3. Run memory leak detection:
   ```bash
   bundle exec rake test:valgrind
   ```

For more detailed instructions and configuration options, refer to the
[ruby_memcheck documentation](https://github.com/Shopify/ruby_memcheck).

## Best Practices

1. **Add Meaningful Error Messages**: Make your error messages descriptive and helpful
2. **Test Edge Cases**: Thoroughly test edge cases like nil values, empty strings, etc.
3. **Maintain a Test Suite**: Comprehensive tests catch issues early
4. **Use Memory Safety Features**: Leverage Rust's safety features rather than bypassing them
5. **Provide Debugging Symbols**: Always include debug symbol builds for better debugging
6. **Document Troubleshooting**: Add a troubleshooting section to your extension's documentation
7. **Log Appropriately**: Include contextual information in log messages

## Next Steps

- Build your extension with `RB_SYS_CARGO_PROFILE=dev` and practice setting breakpoints.
- Explore GDB as an alternative to LLDB for low-level debugging.
- See the Memory Management & Safety chapter for GC-related troubleshooting.
- If you're still stuck, [join the Slack channel][slack] to ask questions and get help from the community!

[slack]: https://join.slack.com/t/oxidize-rb/shared_invite/zt-16zv5tqte-Vi7WfzxCesdo2TqF_RYBCw
