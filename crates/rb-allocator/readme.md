# rb-allocator

[![Join the discussion](https://img.shields.io/badge/slack-chat-blue.svg)](https://join.slack.com/t/oxidize-rb/shared_invite/zt-16zv5tqte-Vi7WfzxCesdo2TqF_RYBCw)

A wrapper around the [Rust system allocator](https://doc.rust-lang.org/std/alloc/struct.System.html) which reports
memory usage back to the Ruby GC. This means that Ruby will have an accurate representation of Rust's memory usage, and
thus know when to GC.

## Usage

1. Add the following to your `Cargo.toml`:

   ```toml
   [dependencies]
   rb-allocator = "0.9.2"
   ```

2. Use the provided `ruby_global_allocator!` macro

   ```rust
   extern crate rb_allocator;

   use rb_allocator::*;

   ruby_global_allocator!();
   ```

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
