# Quick Start: Your First Extension

This chapter shows you how to create a Ruby gem with a Rust extension using Bundler's built-in Rust support.

## Creating a Gem with Bundler

The easiest way to create a new gem with a Rust extension is with Bundler:

```bash
# Create a new gem with a Rust extension
bundle gem --ext=rust hello_rusty
cd hello_rusty
```

This command generates everything you need to build a Ruby gem with a Rust extension.

## Understanding the Generated Files

Let's examine the key files Bundler created:

```
hello_rusty/
├── ext/hello_rusty/          # Rust extension directory
│   ├── Cargo.toml            # Rust dependencies
│   ├── extconf.rb            # Ruby extension config
│   └── src/lib.rs            # Rust code
├── lib/hello_rusty.rb        # Main Ruby file
└── hello_rusty.gemspec       # Gem specification
```

### The Rust Code (lib.rs)

Bundler generates a simple "Hello World" implementation:

```rust
# // ext/hello_rusty/src/lib.rs
use magnus::{define_module, function, prelude::*, Error};

#[magnus::init]
fn init() -> Result<(), Error> {
    let module = define_module("HelloRusty")?;
    module.define_singleton_method("hello", function!(|| "Hello from Rust!", 0))?;
    Ok(())
}
```

<div class="note">

You can click the "play" button on code blocks to try them out in the Rust Playground where appropriate. For code that
depends on the Ruby API, you won't be able to run it directly, but you can experiment with Rust syntax and standard
library functions.

</div>

### The Extension Configuration (extconf.rb)

```ruby
# ext/hello_rusty/extconf.rb
require "mkmf"
require "rb_sys/mkmf"

create_rust_makefile("hello_rusty/hello_rusty")
```

This file connects Ruby's build system to Cargo.

## Enhancing the Default Implementation

Let's improve the default implementation by adding a simple class:

```rust,noplayground,hidelines=#
# // This is our enhanced implementation
use magnus::{define_module, define_class, function, method, prelude::*, Error, Ruby};

// Define a struct to hold state and
// implement Ruby wrapper for the struct
#[magnus::wrap(class = "HelloRusty::Greeter")]
struct Greeter {
    name: String,
}

impl Greeter {
    // Constructor
    fn new(name: String) -> Self {
        Greeter { name }
    }

    // Instance method
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name)
    }
}

# // Let's also add a method that takes a parameter
# impl Greeter {
#     fn greet_with_prefix(&self, prefix: String) -> String {
#         format!("{} Hello, {}!", prefix, self.name)
#     }
# }

// Module initialization function
#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("HelloRusty")?;

    // Define and configure the Greeter class
    let class = module.define_class("Greeter", ruby.class_object())?;
    class.define_singleton_method("new", function!(Greeter::new, 1))?;
    class.define_method("greet", method!(Greeter::greet, 0))?;

    # // We could also expose the additional method
    # // class.define_method("greet_with_prefix", method!(Greeter::greet_with_prefix, 1))?;

    Ok(())
}
```

<div class="tip">

Click the eye icon (<i class="fa fa-eye"></i>) to reveal commented lines with additional functionality that you could
add to your implementation.

</div>

## Building and Testing

### Compile the Extension

```bash
# Install dependencies and compile
bundle install
bundle exec rake compile
```

What happens during compilation:

1. Ruby's `mkmf` reads your `extconf.rb`
2. `create_rust_makefile` generates a Makefile with Cargo commands
3. Cargo compiles your Rust code to a dynamic library
4. The binary is copied to `lib/hello_rusty/hello_rusty.{so,bundle,dll}`

### Run the Tests

Bundler generates a basic test file. Let's update it:

```ruby,hidelines=#
# test/test_hello_rusty.rb
require "test_helper"

class TestHelloRusty < Minitest::Test
  def test_that_it_has_a_version_number
    refute_nil ::HelloRusty::VERSION
  end

  def test_greeter
    greeter = HelloRusty::Greeter.new("Rustacean")
    assert_equal "Hello, Rustacean!", greeter.greet
  end

  # # If we implemented the additional method, we could test it
  # def test_greeter_with_prefix
  #   greeter = HelloRusty::Greeter.new("Rustacean")
  #   assert_equal "Howdy! Hello, Rustacean!", greeter.greet_with_prefix("Howdy!")
  # end
end
```

Run the tests:

```bash,hidelines=#
# Run the standard test suite
bundle exec rake test

# You can also run specific tests
# bundle exec ruby -Ilib:test test/test_hello_rusty.rb -n test_greeter
```

### Try It in the Console

```bash,hidelines=#
# Start the console
bundle exec bin/console

# You can also use irb directly
# bundle exec irb -Ilib -rhello_rusty
```

Once in the console, you can interact with your extension:

```ruby,hidelines=#
# Create a new greeter object
greeter = HelloRusty::Greeter.new("World")

# Call the greet method
puts greeter.greet  # => "Hello, World!"

# # If you added the additional method, you could call it
# puts greeter.greet_with_prefix("Howdy!")  # => "Howdy! Hello, World!"
```

## Customizing the Build

You can customize the build process with environment variables:

```bash,hidelines=#
# Release build (optimized)
RB_SYS_CARGO_PROFILE=release bundle exec rake compile

# With specific Cargo features
RB_SYS_CARGO_FEATURES=feature1,feature2 bundle exec rake compile

# You can also combine variables
# RB_SYS_CARGO_PROFILE=release RB_SYS_CARGO_FEATURES=feature1 bundle exec rake compile

# For more verbose output
# RB_SYS_CARGO_VERBOSE=1 bundle exec rake compile
```

<div class="warning">

Remember that building in release mode will produce optimized, faster code but will increase compilation time.

</div>

## Next Steps

Congratulations! You've created a Ruby gem with a Rust extension. In the next chapters, we'll explore:

- Better project organization
- Working with Ruby objects in Rust
- Memory management and safety
- Performance optimization
