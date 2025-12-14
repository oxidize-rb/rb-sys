# Agent Instructions for rb-sys

## Build/Test/Lint Commands

- **Run all commands via**: `./script/run <cmd>` (sets up dev environment)
- **Run all tests**: `./script/run bundle exec rake test`
- **Run Rust tests only**: `./script/run bundle exec rake test:cargo`
- **Run single crate tests**: `./script/run cargo test -p rb-sys-tests`
- **Run single test**: `./script/run cargo test -p rb-sys-tests test_name`
- **Lint (Rust)**: `./script/run cargo fmt --check && cargo clippy`
- **Lint (Ruby)**: `./script/run bundle exec standardrb`
- **Format all**: `./script/run bundle exec rake fmt`

## Code Style

- **Rust**: Edition 2018, MSRV 1.71. Use `cargo fmt` (default rustfmt). Run `cargo clippy` before commits.
- **Ruby**: StandardRB (based on RuboCop). No explicit style config beyond defaults.
- **Imports**: Group std, external crates, then local modules. Use `pub use` for re-exports in lib.rs.
- **Naming**: Rust snake_case for functions/variables, CamelCase for types. Ruby snake_case throughout.
- **Error handling**: Prefer `Result<T, E>` in Rust. Avoid panics in library code.
- **FFI safety**: Wrap unsafe Ruby C API calls appropriately. Use `#![allow(unused_unsafe)]` in test files.
- **Features**: Gate optional functionality (e.g., `#[cfg(feature = "stable-api")]`).

## Crate Overview

- **rb-sys** (`crates/rb-sys/`): Low-level Rust bindings to Ruby C API via bindgen
- **rb-sys-build** (`crates/rb-sys-build/`): Build system handling bindgen and Ruby headers
- **rb-sys-env** (`crates/rb-sys-env/`): Cargo/rustc config, RbConfig values, Ruby version cfg flags
- **rb-sys-test-helpers** (`crates/rb-sys-test-helpers/`): Test utilities including `#[ruby_test]` macro
- **rb-sys-test-helpers-macros** (`crates/rb-sys-test-helpers-macros/`): Proc-macro impl for test helpers
- **rb-sys-tests** (`crates/rb-sys-tests/`): Internal test suite (not published)
