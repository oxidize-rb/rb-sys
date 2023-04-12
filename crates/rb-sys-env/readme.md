# `rb-sys-env`

Helpers to integrate `rb-sys` into your high-level Ruby bindings library.

## Features

- Provides the neccesary Cargo configuration to ensure that Rust crates compile properly across all platforms
- Sets useful rustc-cfg flags that you can use from your crate
- Exposes all `RbConfig::CONFIG` values from rb-sys
- Provides a `test_helpers::with_ruby_vm` function that you can use to run tests that require a Ruby VM

## Usage

Add this to your `Cargo.toml`:

```toml
[build-dependencies]
rb-sys-env = "0.1"
```

Then, in your crate's `build.rs`:

```rust
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _rb_env = rb_sys_env::activate()?;

    Ok(())
}
```

### Test helpers

This crate also provides a `test_helpers` module that you can use to run tests that require a Ruby VM.

Add this to your `Cargo.toml`:

```toml
[dev-dependencies]
rb-sys-env = { version = "0.1", features = ["test-helpers"] }
```

Then, use the `with_ruby_vm` function in your tests:

```rust
#[cfg(test)]
mod tests {
    use rb_sys_env::test_helpers::with_ruby_vm;

    #[test]
    fn test_something() {
        with_ruby_vm(|| {
            // Your test code here (hint: this works with the `magnus` crate, too!)
        });
    }
}
```

## Available `rustc-cfg`

Here is an example of the `rustc-cfg` flags that are set by this crate:

- `#[cfg(ruby_have_ruby_re_h)]`
- `#[cfg(ruby_use_rgengc)]`
- `#[cfg(ruby_use_symbol_as_method_name)]`
- `#[cfg(ruby_have_ruby_util_h)]`
- `#[cfg(ruby_have_ruby_oniguruma_h)]`
- `#[cfg(ruby_have_ruby_defines_h)]`
- `#[cfg(ruby_use_flonum)]`
- `#[cfg(ruby_have_ruby_onigmo_h)]`
- `#[cfg(ruby_use_unaligned_member_access)]`
- `#[cfg(ruby_use_transient_heap)]`
- `#[cfg(ruby_have_ruby_atomic_h)]`
- `#[cfg(ruby_have_rb_scan_args_optional_hash)]`
- `#[cfg(ruby_have_rb_data_type_t_parent)]`
- `#[cfg(ruby_have_ruby_debug_h)]`
- `#[cfg(ruby_have_ruby_encoding_h)]`
- `#[cfg(ruby_have_ruby_ruby_h)]`
- `#[cfg(ruby_have_ruby_intern_h)]`
- `#[cfg(ruby_use_mjit)]`
- `#[cfg(ruby_have_rb_data_type_t_function)]`
- `#[cfg(ruby_have_rb_fd_init)]`
- `#[cfg(ruby_have_rb_reg_new_str)]`
- `#[cfg(ruby_have_rb_io_t)]`
- `#[cfg(ruby_have_ruby_memory_view_h)]`
- `#[cfg(ruby_have_ruby_version_h)]`
- `#[cfg(ruby_have_ruby_st_h)]`
- `#[cfg(ruby_have_ruby_thread_native_h)]`
- `#[cfg(ruby_have_ruby_random_h)]`
- `#[cfg(ruby_have_ruby_regex_h)]`
- `#[cfg(ruby_have_rb_define_alloc_func)]`
- `#[cfg(ruby_have_ruby_fiber_scheduler_h)]`
- `#[cfg(ruby_have_ruby_missing_h)]`
- `#[cfg(ruby_have_rb_ext_ractor_safe)]`
- `#[cfg(ruby_have_ruby_thread_h)]`
- `#[cfg(ruby_have_ruby_vm_h)]`
- `#[cfg(ruby_use_rincgc)]`
- `#[cfg(ruby_have_ruby_ractor_h)]`
- `#[cfg(ruby_have_ruby_io_h)]`
- `#[cfg(ruby_3)]`
- `#[cfg(ruby_3_1)]`
- `#[cfg(ruby_3_1_2)]`
- `#[cfg(ruby_gte_2_2)]`
- `#[cfg(ruby_gt_2_2)]`
- `#[cfg(ruby_gte_2_3)]`
- `#[cfg(ruby_gt_2_3)]`
- `#[cfg(ruby_gte_2_4)]`
- `#[cfg(ruby_gt_2_4)]`
- `#[cfg(ruby_gte_2_5)]`
- `#[cfg(ruby_gt_2_5)]`
- `#[cfg(ruby_gte_2_6)]`
- `#[cfg(ruby_gt_2_6)]`
- `#[cfg(ruby_gte_2_7)]`
- `#[cfg(ruby_gt_2_7)]`
- `#[cfg(ruby_gte_3_0)]`
- `#[cfg(ruby_gt_3_0)]`
- `#[cfg(ruby_lte_3_1)]`
- `#[cfg(ruby_3_1)]`
- `#[cfg(ruby_eq_3_1)]`
- `#[cfg(ruby_gte_3_1)]`
- `#[cfg(ruby_lt_3_2)]`
- `#[cfg(ruby_lte_3_2)]`
- `#[cfg(ruby_lt_3_3)]`
- `#[cfg(ruby_lte_3_3)]`
- `#[cfg(ruby_gte_1)]`
- `#[cfg(ruby_gt_1)]`
- `#[cfg(ruby_gte_2)]`
- `#[cfg(ruby_gt_2)]`
- `#[cfg(ruby_lte_3)]`
- `#[cfg(ruby_3)]`
- `#[cfg(ruby_eq_3)]`
- `#[cfg(ruby_gte_3)]`
- `#[cfg(ruby_lt_4)]`
- `#[cfg(ruby_lte_4)]`

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
