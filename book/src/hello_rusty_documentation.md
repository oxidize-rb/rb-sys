# Anatomy of a Rusty Ruby Gem: hello_rusty

This documentation provides a comprehensive walkthrough of the `hello_rusty` gem, a simple but complete Ruby gem that uses Rust for its native extension. This example demonstrates the key components of creating a Ruby gem with Rust using rb-sys and magnus.

## Project Structure

A properly structured Rusty Ruby Gem follows the standard Ruby gem conventions with the addition of Rust-specific elements. Here's the structure of the `hello_rusty` gem:

```
hello_rusty/
├── bin/                      # Executable files
├── ext/                      # Native extension code
│   └── hello_rusty/          # The Rust extension directory
│       ├── Cargo.toml        # Rust package manifest
│       ├── extconf.rb        # Ruby extension configuration
│       └── src/
│           └── lib.rs        # Rust implementation
├── lib/                      # Ruby code
│   ├── hello_rusty.rb        # Main Ruby file
│   └── hello_rusty/
│       └── version.rb        # Version definition
├── sig/                      # RBS type signatures
│   └── hello_rusty.rbs       # Type definitions
├── test/                     # Test files
│   ├── test_hello_rusty.rb   # Test for the gem
│   └── test_helper.rb        # Test setup
├── Cargo.lock                # Rust dependency lock file
├── Cargo.toml                # Workspace-level Rust config (optional)
├── Gemfile                   # Ruby dependencies
├── LICENSE.txt               # License file
├── Rakefile                  # Build tasks
├── README.md                 # Documentation
└── hello_rusty.gemspec       # Gem specification
```

## Key Components

### 1. Ruby Components

#### Gemspec (`hello_rusty.gemspec`)

The gemspec defines metadata about the gem and specifies build requirements:

```ruby
# frozen_string_literal: true

require_relative "lib/hello_rusty/version"

Gem::Specification.new do |spec|
  spec.name = "hello_rusty"
  spec.version = HelloRusty::VERSION
  spec.authors = ["Ian Ker-Seymer"]
  spec.email = ["hello@ianks.com"]

  # ... metadata ...
  
  spec.required_ruby_version = ">= 3.0.0"
  
  # Files to include in the gem
  spec.files = [...]
  
  # IMPORTANT: This line tells RubyGems that this gem has a native extension
  # and where to find the build configuration
  spec.extensions = ["ext/hello_rusty/Cargo.toml"]
  
  spec.require_paths = ["lib"]
end
```

Key points:
- The `extensions` field points to the Cargo.toml file
- Version is defined in a separate Ruby file
- Required Ruby version is specified

#### Main Ruby file (`lib/hello_rusty.rb`)

```ruby
# frozen_string_literal: true

require_relative "hello_rusty/version"
require_relative "hello_rusty/hello_rusty"  # Loads the compiled Rust extension

module HelloRusty
  class Error < StandardError; end
  # Additional Ruby code can go here
end
```

Key points:
- Requires the version file
- Requires the compiled native extension
- Defines a module matching the Rust module

#### Version file (`lib/hello_rusty/version.rb`)

```ruby
# frozen_string_literal: true

module HelloRusty
  VERSION = "0.1.0"
end
```

#### Type Definitions (`sig/hello_rusty.rbs`)

RBS type definitions for better IDE support and type checking:

```rbs
module HelloRusty
  VERSION: String
  # Add type signatures for your methods here
end
```

### 2. Rust Components

#### Cargo Configuration (`ext/hello_rusty/Cargo.toml`)

```toml
[package]
name = "hello_rusty"
version = "0.1.0"
edition = "2021"
authors = ["Ian Ker-Seymer <hello@ianks.com>"]
license = "MIT"
publish = false

[lib]
crate-type = ["cdylib"]  # Outputs a dynamic library

[dependencies]
magnus = { version = "0.6.2" }  # High-level Ruby bindings
```

Key points:
- Uses `cdylib` crate type to build a dynamic library
- Depends on `magnus` for high-level Ruby bindings
- Version should match the Ruby gem version

#### Rust Implementation (`ext/hello_rusty/src/lib.rs`)

```rust
use magnus::{function, prelude::*, Error, Ruby};

fn hello(subject: String) -> String {
    format!("Hello from Rust, {subject}!")
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("HelloRusty")?;
    module.define_singleton_method("hello", function!(hello, 1))?;
    Ok(())
}
```

Key points:
- Uses the `magnus` crate for Ruby integration
- The `#[magnus::init]` macro marks the entry point for the extension
- Defines a Ruby module matching the gem name
- Exposes the `hello` Rust function as a Ruby method

### 3. Build System

#### Extension Configuration (`ext/hello_rusty/extconf.rb`)

```ruby
# frozen_string_literal: true

require "mkmf"
require "rb_sys/mkmf"

create_rust_makefile("hello_rusty/hello_rusty")
```

Key points:
- Uses `rb_sys/mkmf` to handle Rust compilation
- Creates a makefile for the native extension

#### Rakefile (`Rakefile`)

```ruby
# frozen_string_literal: true

require "bundler/gem_tasks"
require "minitest/test_task"
require "rubocop/rake_task"
require "rb_sys/extensiontask"

task build: :compile

GEMSPEC = Gem::Specification.load("hello_rusty.gemspec")

RbSys::ExtensionTask.new("hello_rusty", GEMSPEC) do |ext|
  ext.lib_dir = "lib/hello_rusty"
end

Minitest::TestTask.create
RuboCop::RakeTask.new

task default: %i[compile test rubocop]
```

Key points:
- Uses `RbSys::ExtensionTask` to manage Rust compilation
- Sets the output directory to `lib/hello_rusty`
- Defines standard tasks for building, testing, and linting

### 4. Testing

#### Test File (`test/test_hello_rusty.rb`)

```ruby
# frozen_string_literal: true

require "test_helper"

class TestHelloRusty < Minitest::Test
  def test_that_it_has_a_version_number
    refute_nil ::HelloRusty::VERSION
  end

  def test_hello
    result = HelloRusty.hello("World")
    assert_equal "Hello from Rust, World!", result
  end
end
```

Key points:
- Tests basic functionality of the gem
- Verifies the version is defined
- Tests the Rust-implemented `hello` method

## Build Process

When building a Rusty Ruby Gem, the following steps occur:

1. `rake compile` is run (either directly or through `rake build`)
2. The `RbSys::ExtensionTask` processes the extension:
   - It reads the `ext/hello_rusty/Cargo.toml` file
   - It sets up the appropriate build environment
   - It runs `cargo build` with the appropriate options
   - It copies the resulting `.so`/`.bundle`/`.dll` to `lib/hello_rusty/`
3. The compiled binary is then packaged into the gem

## Usage

Once installed, this gem can be used in Ruby code as follows:

```ruby
require "hello_rusty"

# Call the Rust-implemented method
greeting = HelloRusty.hello("Rusty Rubyist")
puts greeting  # => "Hello from Rust, Rusty Rubyist!"
```

## Key Concepts Demonstrated

1. **Module Structure**: The gem defines a Ruby module that's implemented in Rust
2. **Function Exposure**: Rust functions are exposed as Ruby methods
3. **Type Conversion**: Rust handles string conversion automatically through magnus
4. **Error Handling**: The Rust code uses `Result<T, Error>` for Ruby-compatible error handling
5. **Build Integration**: The gem uses rb-sys to integrate with Ruby's build system
6. **Testing**: Standard Ruby testing tools work with the Rust-implemented functionality

## Next Steps for Expansion

To expand this basic example, you could:

1. Add Ruby classes backed by Rust structs using TypedData
2. Implement more complex methods with various argument types
3. Add error handling with custom Ruby exceptions
4. Use the Ruby GVL (Global VM Lock) for thread safety
5. Implement memory management through proper object marking
6. Add benchmarks to demonstrate performance characteristics

This example provides a solid foundation for understanding the structure and implementation of Rusty Ruby Gems with rb-sys.