# rb-sys-test-helpers

The `rb-sys-test-helpers` crate provides utilities for testing Ruby extensions from Rust. It makes it easy to run tests with a valid Ruby VM.

## Usage

Add this to your `Cargo.toml`:

```toml
[dev-dependencies]
rb-sys-env = { version = "0.1" }
rb-sys-test-helpers = { version = "0.2" }
```

Then, in your crate's `build.rs`:

```rust
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = rb_sys_env::activate()?;

    Ok(())
}
```

Then, you can use the `ruby_test` attribute macro in your tests:

```rust
#[cfg(test)]
mod tests {
    use rb_sys_test_helpers::ruby_test;
    use rb_sys::{rb_num2fix, rb_int2big, FIXNUM_P};

    #[ruby_test]
    fn test_something() {
        // Your test code here will have a valid Ruby VM (hint: this works with
        // the `magnus` crate, too!)
        //
        // ...

        let int = unsafe { rb_num2fix(1) };
        let big = unsafe { rb_int2big(9999999) };

        assert!(FIXNUM_P(int));
        assert!(!FIXNUM_P(big));
    }
}
```

## How It Works

The `ruby_test` macro sets up a Ruby VM before running your test and tears it down afterward. This allows you to interact with Ruby from your Rust code during tests without having to set up the VM yourself.

The test helpers are compatible with both `rb-sys` for low-level C API access and `magnus` for higher-level Ruby interactions.

## Common Testing Patterns

### Testing Value Conversions

```rust
#[ruby_test]
fn test_value_conversion() {
    use rb_sys::{rb_cObject, rb_funcall, rb_str_new_cstr, rb_iv_set};
    use std::ffi::CString;

    unsafe {
        let obj = rb_cObject;
        let name = CString::new("test").unwrap();
        let value = rb_str_new_cstr(name.as_ptr());
        
        rb_iv_set(obj, b"@name\0".as_ptr() as *const _, value);
        
        let result = rb_funcall(obj, b"instance_variable_get\0".as_ptr() as *const _, 1, 
                                rb_str_new_cstr(b"@name\0".as_ptr() as *const _));
                                
        assert_eq!(value, result);
    }
}
```

### Testing with Magnus

```rust
#[ruby_test]
fn test_with_magnus() {
    use magnus::{Ruby, RString, Value};
    
    let ruby = unsafe { Ruby::get().unwrap() };
    let string = RString::new(ruby, "Hello, world!").unwrap();
    
    assert_eq!(string.to_string().unwrap(), "Hello, world!");
}
```

## Testing Multiple Ruby Versions

To test against multiple Ruby versions, you can use environment variables and CI configuration:

```yaml
# .github/workflows/test.yml
jobs:
  test:
    strategy:
      matrix:
        ruby: ['2.7', '3.0', '3.1', '3.2', '3.3']
    steps:
      - uses: actions/checkout@v4
      - uses: oxidize-rb/actions/setup-ruby-and-rust@v1
        with:
          ruby-version: ${{ matrix.ruby }}
      - run: cargo test
```

Your tests will run against each Ruby version in the matrix, helping you ensure compatibility.

## Integration with Other Test Frameworks

The `ruby_test` attribute works with common Rust test frameworks like `proptest` and `quickcheck`:

```rust
#[ruby_test]
fn test_with_proptest() {
    use proptest::prelude::*;
    
    proptest!(|(s in "[a-zA-Z0-9]*")| {
        let ruby = unsafe { Ruby::get().unwrap() };
        let ruby_string = RString::new(ruby, &s).unwrap();
        assert_eq!(ruby_string.to_string().unwrap(), s);
    });
}
```