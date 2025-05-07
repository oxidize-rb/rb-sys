# rb-sys Crate Features

The `rb-sys` crate provides battle-tested Rust bindings for the Ruby C API. It uses the [`rust-bindgen`](https://github.com/rust-lang/rust-bindgen) crate to generate bindings from the `ruby.h` header.

## Usage Notice

This is a very low-level library. If you are looking to write a gem in Rust, you should probably use the [Magnus](https://github.com/matsadler/magnus) crate with the `rb-sys-interop` feature, which provides a higher-level, more ergonomic API.

If you need raw/unsafe bindings to libruby, then this crate is for you!

## Writing a Ruby Gem

Ruby gems require boilerplate to be defined to be usable from Ruby. `rb-sys` makes this process painless by doing the work for you. Simply enable the `gem` feature:

```toml
[dependencies]
rb-sys = "0.9"
```

Under the hood, this ensures the crate does not link libruby (unless on Windows) and defines a `ruby_abi_version` function for Ruby 3.2+.

## Embedding libruby in Your Rust App

**IMPORTANT**: If you are authoring a Ruby gem, you do not need to enable this feature.

If you need to link libruby (i.e., you are initializing a Ruby VM in your Rust code), you can enable the `link-ruby` feature:

```toml
[dependencies]
rb-sys = { version = "0.9", features = ["link-ruby"] }
```

## Static libruby

You can also force static linking of libruby:

```toml
[dependencies]
rb-sys = { version = "0.9", features = ["ruby-static"] }
```

Alternatively, you can set the `RUBY_STATIC=true` environment variable.

## Available Features

The `rb-sys` crate provides several features that can be enabled in your `Cargo.toml`:

| Feature | Description |
|---------|-------------|
| `global-allocator` | Report Rust memory allocations to the Ruby GC (_recommended_) |
| `ruby-static` | Link the static version of libruby |
| `link-ruby` | Link libruby (typically used for embedding, not for extensions) |
| `bindgen-rbimpls` | Include the Ruby impl types in bindings |
| `bindgen-deprecated-types` | Include deprecated Ruby methods in bindings |
| `gem` | Set up the crate for use in a Ruby gem (default feature) |
| `stable-api` | Use the stable API (C level) if available for your Ruby version |

## Example Cargo.toml

```toml
[dependencies]
rb-sys = { version = "0.9", features = ["global-allocator", "stable-api"] }
```

## Ruby Version Compatibility

`rb-sys` is compatible with Ruby 2.6 and later. The crate detects the Ruby version at compile time and adapts the bindings accordingly.

For Ruby 3.2 and later, `rb-sys` provides a `ruby_abi_version` function that is required for native extensions.

## Integration with Magnus

If you're building a Ruby extension, it's recommended to use the [Magnus](https://github.com/matsadler/magnus) crate on top of `rb-sys`. Magnus provides a high-level, safe API for interacting with Ruby:

```toml
[dependencies]
magnus = { version = "0.7", features = ["rb-sys"] }
rb-sys = "0.9"
```