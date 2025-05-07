# Project Structure

In this chapter, we'll explore how to set up and organize a Ruby gem with a Rust extension. We'll focus on practical
patterns and highlight how to leverage valuable Rust libraries without introducing unnecessary complexity.

## Enhanced Project Structure

Building on the structure created by `bundle gem --ext=rust`, a well-organized rb-sys project typically looks like this:

```
my_gem/
├── Cargo.toml                # Rust workspace configuration
├── Gemfile                   # Ruby dependencies
├── Rakefile                  # Build tasks
├── my_gem.gemspec            # Gem specification
├── ext/
│   └── my_gem/
│       ├── Cargo.toml        # Rust crate configuration
│       ├── extconf.rb        # Ruby extension configuration
│       └── src/
│           └── lib.rs        # Main Rust entry point
├── lib/
│   ├── my_gem.rb             # Main Ruby file
│   └── my_gem/
│       └── version.rb        # Version information
└── test/                     # Tests
```

Let's examine a practical example using a useful but simple Rust library.

## Example: URL Parsing with the `url` crate

The [url](https://crates.io/crates/url) crate, developed by the Servo team, is a robust implementation of the URL
Standard. It provides accurate URL parsing that would be complex to implement from scratch. Here's a simple example:

### 1. Extension Cargo.toml

```toml
# ext/url_parser/Cargo.toml
[package]
name = "url_parser"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
license = "MIT"
publish = false

[lib]
crate-type = ["cdylib"]

[dependencies]
# High-level Ruby bindings with rb-sys feature
magnus = { version = "0.7", features = ["rb-sys"] }

# The main Rust library we're wrapping
url = "2.4"

[build-dependencies]
rb-sys-env = "0.1"
```

### 2. Main Rust Implementation

```rust
// ext/url_parser/src/lib.rs
use magnus::{define_module, define_class, function, method, prelude::*, Error, Ruby};
use url::Url;

// Simple URL wrapper class
struct UrlWrapper {
    inner: Url,
}

#[magnus::wrap(class = "UrlParser::URL")]
impl UrlWrapper {
    // Parse a URL string
    fn parse(url_str: String) -> Result<Self, Error> {
        match Url::parse(&url_str) {
            Ok(url) => Ok(UrlWrapper { inner: url }),
            Err(err) => {
                Err(Error::new(magnus::exception::arg_error(), format!("Invalid URL: {}", err)))
            }
        }
    }

    // Basic getters
    fn scheme(&self) -> String {
        self.inner.scheme().to_string()
    }

    fn host(&self) -> Option<String> {
        self.inner.host_str().map(|s| s.to_string())
    }

    fn path(&self) -> String {
        self.inner.path().to_string()
    }

    fn query(&self) -> Option<String> {
        self.inner.query().map(|s| s.to_string())
    }

    // String representation of the URL
    fn to_string(&self) -> String {
        self.inner.to_string()
    }
}

// Module-level utilities
fn is_valid_url(url_str: String) -> bool {
    Url::parse(&url_str).is_ok()
}

// Module init function - Ruby extension entry point
#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    // Define the main module
    let module = ruby.define_module("UrlParser")?;

    // Add utility function at module level
    module.define_singleton_method("valid_url?", function!(is_valid_url, 1))?;

    // Define and configure the URL class
    let class = module.define_class("URL", ruby.class_object())?;
    class.define_singleton_method("parse", function!(UrlWrapper::parse, 1))?;

    // Instance methods
    class.define_method("scheme", method!(UrlWrapper::scheme, 0))?;
    class.define_method("host", method!(UrlWrapper::host, 0))?;
    class.define_method("path", method!(UrlWrapper::path, 0))?;
    class.define_method("query", method!(UrlWrapper::query, 0))?;
    class.define_method("to_s", method!(UrlWrapper::to_string, 0))?;

    Ok(())
}
```

### 3. Ruby Integration

```ruby
# lib/url_parser.rb
require_relative "url_parser/version"
require_relative "url_parser/url_parser"

module UrlParser
  class Error < StandardError; end

  # Parse a URL string and return a URL object
  def self.parse(url_string)
    URL.parse(url_string)
  rescue => e
    raise Error, "Failed to parse URL: #{e.message}"
  end

  # Check if a URL has an HTTPS scheme
  def self.https?(url_string)
    return false unless valid_url?(url_string)
    url = parse(url_string)
    url.scheme == "https"
  end
end
```

### 4. Simple Tests

```ruby
# test/test_url_parser.rb
require "test_helper"

class TestUrlParser < Minitest::Test
  def test_basic_url_parsing
    url = UrlParser::URL.parse("https://example.com/path?query=value")

    assert_equal "https", url.scheme
    assert_equal "example.com", url.host
    assert_equal "/path", url.path
    assert_equal "query=value", url.query
  end

  def test_url_validation
    assert UrlParser.valid_url?("https://example.com")
    refute UrlParser.valid_url?("not a url")
  end

  def test_https_check
    assert UrlParser.https?("https://example.com")
    refute UrlParser.https?("http://example.com")
  end

  def test_invalid_url_raises_error
    assert_raises UrlParser::Error do
      UrlParser.parse("not://a.valid/url")
    end
  end
end
```

## Key Aspects of this Project

### 1. Simplicity with Value

This example demonstrates how to:

- Wrap a useful Rust library (`url`) with minimal code
- Expose only the most essential functionality
- Handle errors properly
- Integrate with Ruby idiomatically

### 2. Why Use Rust for URL Parsing?

Ruby has URI handling in its standard library, but the Rust `url` crate offers advantages:

- Full compliance with the URL standard used by browsers
- Better handling of internationalized domain names (IDNs)
- Robust error detection
- Significant performance benefits for URL-heavy applications

### 3. Project Organization Principles

- **Keep dependencies minimal**: Just what you need, nothing more
- **Clean public API**: Expose only what users need
- **Proper error handling**: Map Rust errors to meaningful Ruby exceptions
- **Simple tests**: Verify both functionality and edge cases
