# `rb-sys-test-helpers`

Helpers for testing Ruby extensions from Rust

## Usage

Add this to your `Cargo.toml`:

```toml
[dev-dependencies]
rb-sys-env = { version = "0.1" }
rb-sys-test-helpers = { version = "0.2" }
```

Then, in your crate's `build.rs`:

```rust,ignore
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = rb_sys_env::activate()?;

    Ok(())
}
```

Then, you can use the `with_ruby_vm` function in your tests:

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

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
