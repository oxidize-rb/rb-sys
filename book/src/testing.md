{{#title Ruby on Rust: Testing Extensions}}

# Testing Extensions

Testing is a critical part of developing Ruby extensions. This chapter covers strategies for testing your Rust code that interfaces with Ruby, from unit tests to integration tests and CI workflows.

<div class="warning">

Testing is particularly important for Ruby extensions because segmentation faults, memory leaks, and other low-level issues can crash the entire Ruby VM. Untested extensions can lead to hard-to-debug production crashes.

</div>

## Unit Testing Rust Code

### The Challenge of Testing Ruby Extensions

Testing Rust code that interacts with Ruby presents unique challenges:

1. **Ruby VM Initialization**: The Ruby VM must be properly initialized before tests run.
2. **Thread Safety**: Ruby's VM has thread-specific state that must be managed.
3. **Exception Handling**: Ruby exceptions need to be properly caught and converted to Rust errors.
4. **Memory Management**: Memory allocated by Ruby needs to be protected from garbage collection during tests.

<div class="note">

rb-sys provides specialized tools to overcome these challenges, particularly the `#[ruby_test]` macro which handles Ruby VM initialization and thread management automatically.

</div>

### Complete Test Setup Guide

Setting up proper testing for Ruby extensions requires several components working together. This guide provides a comprehensive setup that you can adapt to your project.

#### Required Dependencies

Your `Cargo.toml` needs to be configured with the appropriate dependencies:

```toml
[package]
name = "my_extension"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

# Main dependencies
[dependencies]
magnus = "0.6" # For high-level Ruby API
rb-sys = "0.9"  # Required for rb_sys_test_helpers to work

# Test dependencies
[dev-dependencies]
rb-sys-env = "0.1"          # For Ruby environment detection
rb-sys-test-helpers = "0.2" # For Ruby VM test helpers
```

The key points:
- Include `rb-sys` as a regular dependency (not just a dev-dependency)
- Both `rb-sys-env` and `rb-sys-test-helpers` are needed for tests

#### Setting Up build.rs

Create a `build.rs` file in your project root with the following content:

```rust
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // This activates rb-sys-env for both normal builds and tests
    let _ = rb_sys_env::activate()?;

    // Any additional build configuration can go here

    Ok(())
}
```

The `rb_sys_env::activate()` function:
- Sets up Cargo configuration based on the detected Ruby environment
- Exposes Ruby version information as feature flags (e.g., `ruby_gte_3_0`, `ruby_use_flonum`) 
- Ensures proper linking to the Ruby library

#### Importing Test Helpers

In your test module, import the necessary components:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rb_sys_test_helpers::ruby_test;
    use magnus::{Ruby, Error};

    // Your test functions go here...
}
```

### The #[ruby_test] Macro

<div class="tip">

The `#[ruby_test]` macro is the simplest and most reliable way to test Ruby extensions in Rust. It handles all the complexities of VM initialization and thread management.

</div>

The simplest way to test Ruby extensions is with the `#[ruby_test]` macro, which wraps your test functions to ensure they run within a properly initialized Ruby VM:

```rust,hidelines=#
# // Complete example of using the ruby_test macro
use rb_sys::*;
use rb_sys_test_helpers::ruby_test;

#[ruby_test]
fn test_string_manipulation() {
    unsafe {
        // Create a Ruby string
        let rb_str = rb_utf8_str_new_cstr("hello\0".as_ptr() as _);
        
        // Append to the string
        let rb_str = rb_str_cat(rb_str, " world\0".as_ptr() as _, 6);
        
        // Convert to Rust string for assertion
        let mut rb_str_val = rb_str;
        let result_ptr = rb_string_value_cstr(&mut rb_str_val);
        let result = std::ffi::CStr::from_ptr(result_ptr)
            .to_string_lossy()
            .to_string();
        
        assert_eq!(result, "hello world");
    }
}

# // You can add options to the macro
# #[ruby_test(gc_stress)]
# fn test_with_gc_stress() {
#     // This test will run with GC stress enabled
#     // Ruby's garbage collector will run more frequently
#     // to help catch memory management issues
#     unsafe {
#         let rb_str = rb_utf8_str_new_cstr("test\0".as_ptr() as _);
#         rb_gc_guard!(rb_str); // Protect from GC
#         rb_gc(); // Force garbage collection
#         // If rb_str was not protected, it might be collected here
#     }
# }
#
# // Version-specific tests using rb-sys-env features
# #[ruby_test]
# fn test_with_version_conditionals() {
#     // This block only runs on Ruby 3.0 or newer
#     #[cfg(ruby_gte_3_0)]
#     {
#         // Test Ruby 3.0+ specific features
#     }
#     
#     // This block only runs on Ruby 2.7
#     #[cfg(all(ruby_gte_2_7, ruby_lt_3_0))]
#     {
#         // Test Ruby 2.7 specific features
#     }
#     
#     // This block runs if float values are stored
#     // as immediate values (Ruby implementation detail)
#     #[cfg(ruby_use_flonum)]
#     {
#         // Test flonum implementation
#     }
# }
```

<div class="note">

The `#[ruby_test]` macro:
1. Ensures the Ruby VM is initialized once and only once
2. Runs all tests on the same OS thread
3. Catches and propagates Ruby exceptions as Rust errors
4. Performs GC after each test to catch memory management issues

Click the eye icon (<i class="fa fa-eye"></i>) to view additional examples of the macro options and version-specific testing.

</div>

The `#[ruby_test]` macro:
1. Ensures the Ruby VM is initialized once and only once
2. Runs all tests on the same OS thread
3. Catches and propagates Ruby exceptions as Rust errors
4. Performs GC after each test to catch memory management issues

### Using Magnus with #[ruby_test]

<div class="tip">

Magnus provides a much more ergonomic Rust API for working with Ruby. Combined with the `#[ruby_test]` macro, it makes testing Ruby extensions much simpler and safer.

</div>

One of the great advantages of the `#[ruby_test]` macro is that it works seamlessly with Magnus, providing a much more ergonomic way to test Ruby integrations:

```rust,hidelines=#
# // Complete example of using Magnus with ruby_test
use magnus::{RString, Ruby};
use rb_sys_test_helpers::ruby_test;

#[ruby_test]
fn test_with_magnus() {
    // Get the Ruby interpreter - no unsafe required when using Magnus!
    let ruby = Ruby::get().unwrap();
    
    // Create a Ruby string with Magnus
    let hello = RString::new(ruby, "Hello, ");
    
    // Append to the string
    let message = hello.concat(ruby, "World!");
    
    // Convert to Rust string for assertion - easy with Magnus
    let result = message.to_string().unwrap();
    
    assert_eq!(result, "Hello, World!");
}

# // Testing more complex Ruby interactions
# #[ruby_test]
# fn test_ruby_class_interaction() {
#     let ruby = Ruby::get().unwrap();
#     
#     // Define a Ruby class for testing
#     let test_class = ruby.define_class("TestClass", ruby.class_object()).unwrap();
#     
#     // Define a method on the class
#     test_class.define_method("double", 
#         magnus::method!(|_rb_self, num: i64| -> i64 { num * 2 }, 1)
#     ).unwrap();
#     
#     // Use Ruby's eval to test the class
#     let result: i64 = ruby.eval("TestClass.new.double(21)").unwrap();
#     
#     assert_eq!(result, 42);
# }
```

<div class="note">

Magnus makes it much easier to interact with Ruby objects in a safe and idiomatic way. Using Magnus with the `#[ruby_test]` macro gives you the best of both worlds:
- Magnus's safe, high-level API
- The `#[ruby_test]` macro's robust Ruby VM management

Click the eye icon (<i class="fa fa-eye"></i>) to see examples of more complex Ruby class interactions.

</div>

Magnus makes it much easier to interact with Ruby objects in a safe and idiomatic way. Using Magnus with the `#[ruby_test]` macro gives you the best of both worlds:
- Magnus's safe, high-level API
- The `#[ruby_test]` macro's robust Ruby VM management

Here's another example showing how to work with Ruby classes and methods using Magnus:

```rust
use magnus::{class, eval, method, prelude::*, Module, RClass, Ruby};
use rb_sys_test_helpers::ruby_test;

#[ruby_test]
fn test_ruby_class_interaction() {
    let ruby = Ruby::get().unwrap();
    
    // Define a Ruby class for testing
    let test_class = ruby.define_class("TestClass", ruby.class_object()).unwrap();
    
    // Define a method on the class
    test_class.define_method("double", method!(|ruby, num: i64| -> i64 {
        num * 2
    })).unwrap();
    
    // Create an instance and call the method
    let result: i64 = eval!(ruby, "TestClass.new.double(21)").unwrap();
    
    assert_eq!(result, 42);
}
```

### Testing with GC Stress

To detect subtle memory management issues, you can enable GC stress testing:

```rust
#[ruby_test(gc_stress)]
fn test_gc_interactions() {
    unsafe {
        // Create a Ruby string
        let s = rb_str_new_cstr("hello world\0".as_ptr() as _);
        
        // Get a pointer to the string's contents
        let s_ptr = RSTRING_PTR(s);
        
        // Protect s from garbage collection
        rb_gc_guard!(s);
        
        // Now we can safely use s_ptr, even though GC might run
        let t = rb_str_new_cstr("prefix: \0".as_ptr() as _);
        let result = rb_str_cat_cstr(t, s_ptr);
        
        // More code...
    }
}
```

With Magnus, the same test is more straightforward:

```rust
use magnus::{RString, Ruby};
use rb_sys_test_helpers::ruby_test;

#[ruby_test(gc_stress)]
fn test_gc_interactions_with_magnus() {
    let ruby = Ruby::get().unwrap();
    
    // Create first string
    let s = RString::new(ruby, "hello world");
    
    // Magnus handles GC protection automatically!
    
    // Create second string and concatenate
    let t = RString::new(ruby, "prefix: ");
    let result = t.concat(ruby, &s);
    
    assert_eq!(result.to_string().unwrap(), "prefix: hello world");
}
```

The `gc_stress` option forces Ruby's garbage collector to run frequently during the test, which helps expose bugs related to:
- Objects not being properly protected from GC
- Dangling pointers
- Invalid memory access

### Handling Ruby Exceptions

Ruby exceptions can be caught and converted to Rust errors using the `protect` function:

```rust
#[ruby_test]
fn test_exception_handling() {
    use rb_sys_test_helpers::protect;
    
    // This code will raise a Ruby exception
    let result = unsafe {
        protect(|| {
            rb_sys::rb_raise(rb_sys::rb_eRuntimeError, "Test error\0".as_ptr() as _);
            // This will never be reached
            "success"
        })
    };
    
    // Verify we got an error
    assert!(result.is_err());
    
    // Check the error message
    let error = result.unwrap_err();
    assert!(error.message().unwrap().contains("Test error"));
}
```

With Magnus, exception handling is more natural:

```rust
use magnus::{eval, Ruby, Error};
use rb_sys_test_helpers::ruby_test;

#[ruby_test]
fn test_exception_handling_with_magnus() {
    let ruby = Ruby::get().unwrap();
    
    // Evaluate Ruby code that raises an exception
    let result: Result<String, Error> = eval!(ruby, "raise 'Test error'");
    
    // Verify we got an error
    assert!(result.is_err());
    
    // Magnus errors contain the Ruby exception
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Test error"));
}
```

### Version-Specific Tests

rb-sys-env provides feature flags that allow you to write version-specific tests:

```rust
#[ruby_test]
fn test_version_specific_features() {
    // This test will only run on Ruby 3.0 or higher
    #[cfg(ruby_gte_3_0)]
    {
        // Test Ruby 3.0+ specific features
        unsafe {
            // Example: using Ractor API which is only available in Ruby 3.0+
            #[cfg(ruby_have_ruby_ractor_h)]
            let is_ractor_supported = rb_sys::rb_ractor_main_p() != 0;
            
            // ...
        }
    }
    
    // This block will only run on Ruby 2.7
    #[cfg(all(ruby_gte_2_7, ruby_lt_3_0))]
    {
        // Test Ruby 2.7 specific features
        // ...
    }
}
```

With Magnus:

```rust
use magnus::{Ruby, eval};
use rb_sys_test_helpers::ruby_test;

#[ruby_test]
fn test_version_specific_features_with_magnus() {
    let ruby = Ruby::get().unwrap();
    
    // This test will only run on Ruby 3.0 or higher
    #[cfg(ruby_gte_3_0)]
    {
        // Test Ruby 3.0+ specific features
        #[cfg(ruby_have_ruby_ractor_h)]
        let is_ractor_supported: bool = eval!(ruby, "defined?(Ractor) != nil").unwrap();
        
        #[cfg(ruby_have_ruby_ractor_h)]
        assert!(is_ractor_supported);
    }
}
```

Available version flags include:
- `ruby_gte_X_Y`: Ruby version >= X.Y
- `ruby_lt_X_Y`: Ruby version < X.Y
- `ruby_eq_X_Y`: Ruby version == X.Y
- `ruby_have_FEATURE`: Specific Ruby API feature is available

### Test Helpers and Macros

rb-sys-test-helpers includes several macros to simplify common testing patterns:

```rust
// Convert a Ruby string to a Rust String for testing
#[ruby_test]
fn test_with_helper_macros() {
    use rb_sys_test_helpers::rstring_to_string;
    
    unsafe {
        let rb_str = rb_utf8_str_new_cstr("hello world\0".as_ptr() as _);
        let rust_str = rstring_to_string!(rb_str);
        
        assert_eq!(rust_str, "hello world");
    }
}
```

### Manual Ruby VM Setup

For more complex test scenarios, you can manually initialize the Ruby VM:

```rust
use rb_sys_test_helpers::{with_ruby_vm, protect};

#[test]
fn test_complex_scenario() {
    with_ruby_vm(|| {
        // Multiple operations that need a Ruby VM
        let result1 = unsafe {
            protect(|| {
                // First operation...
                42
            })
        };
        
        let result2 = unsafe {
            protect(|| {
                // Second operation...
                "success"
            })
        };
        
        assert_eq!(result1.unwrap(), 42);
        assert_eq!(result2.unwrap(), "success");
    }).unwrap();
}
```

With Magnus, the same approach but more ergonomically:

```rust
use magnus::{eval, Ruby};
use rb_sys_test_helpers::with_ruby_vm;

#[test]
fn test_complex_scenario_with_magnus() {
    with_ruby_vm(|| {
        let ruby = Ruby::get().unwrap();
        
        // First operation
        let result1: i64 = eval!(ruby, "21 * 2").unwrap();
        
        // Second operation
        let result2: String = eval!(ruby, "'suc' + 'cess'").unwrap();
        
        assert_eq!(result1, 42);
        assert_eq!(result2, "success");
    }).unwrap();
}
```

## Debugging Failed Tests

When your tests fail, debugging tools can help identify the root cause. LLDB is particularly useful for debugging memory issues, segmentation faults, and other low-level problems.

### Using LLDB to Debug Tests

LLDB is a powerful debugger that works well with Rust and Ruby code. Here's how to use it with your tests:

1. First, compile your extension with debug symbols:

   ```bash
   RUSTFLAGS="-g" bundle exec rake compile
   ```

2. Run your test with LLDB:

   ```bash
   lldb -- ruby -Ilib -e "require 'my_extension'; require_relative 'test/test_my_extension.rb'"
   ```

3. At the LLDB prompt, set breakpoints in your Rust code:

   ```
   (lldb) breakpoint set --name MutCalculator::divide
   ```

4. Run the program:

   ```
   (lldb) run
   ```

5. When the breakpoint is hit, you can:
   - Examine variables: `frame variable`
   - Print expressions: `p self` or `p val`
   - Step through code: `next` (over) or `step` (into)
   - Continue execution: `continue`
   - Show backtrace: `bt`

### LLDB Commands for Ruby Extensions

Some LLDB commands that are particularly useful for Ruby extensions:

```
# To print a Ruby string VALUE
(lldb) p rb_string_value_cstr(&my_rb_string_val)

# To check if a VALUE is nil
(lldb) p RB_NIL_P(my_value)

# To get the Ruby class name of an object
(lldb) p rb_class2name(rb_class_of(my_value))

# To check Ruby exception information
(lldb) p rb_errinfo()
```

### Debugging Memory Issues

For memory-related issues:

1. Set a breakpoint around where objects are created
2. Set a breakpoint where the crash occurs
3. When hitting the first breakpoint, note memory addresses
4. When hitting the second breakpoint, check if those addresses are still valid

```bash
# Example debugging session for memory issues
$ lldb -- ruby -Ilib -e "require 'my_extension'; MyExtension.test_method"
(lldb) breakpoint set --name MutPoint::new
(lldb) breakpoint set --name MutPoint::add_x
(lldb) run

# When first breakpoint hits
(lldb) frame variable
(lldb) p self
(lldb) continue

# When second breakpoint hits
(lldb) frame variable
(lldb) p self
```

### Debugging RefCell Borrow Errors

For diagnosing `BorrowMutError` panics:

1. Set a breakpoint right before the borrow operation:

   ```
   (lldb) breakpoint set --file lib.rs --line 123
   ```

2. When it hits, check the status of the RefCell:

   ```
   (lldb) p self.0
   ```

3. Step through the code and watch when borrows occur:

   ```
   (lldb) next
   ```

### Further Information

For more comprehensive debugging setup including VSCode integration and debugging the Ruby C API, see the [Debugging & Troubleshooting](debugging.md) chapter.

### Common Testing Patterns and Anti-Patterns

When testing Ruby extensions, several patterns emerge that can help you write more effective tests, along with anti-patterns to avoid.

#### Pattern: Proper Method Invocation

```rust
// ✅ GOOD: Using associated function syntax for methods with Ruby/self parameters
let result = MutCalculator::divide(&ruby, &calc, 6.0, 2.0);

// ❌ BAD: This won't compile - can't call as instance method
// let result = calc.divide(&ruby, 6.0, 2.0);
```

#### Pattern: Complete RefCell Borrows

```rust
// ✅ GOOD: Complete the borrow before attempting to borrow mutably
let current_x = self.0.borrow().x;  // First borrow completes here
if let Some(sum) = current_x.checked_add(val) {
    self.0.borrow_mut().x = sum;    // Safe to borrow mutably now
}

// ❌ BAD: Will panic with "already borrowed: BorrowMutError"
// if let Some(sum) = self.0.borrow().x.checked_add(val) {
//     self.0.borrow_mut().x = sum;  // Error: still borrowed from the if condition
// }
```

#### Pattern: Ruby Error Checking

Testing error handling is crucial for Ruby extensions. Here's how to properly test different exception scenarios:

```rust
// ✅ GOOD: Verify specific Ruby exception types
let result = MutCalculator::divide(&ruby, &calc, 6.0, 0.0);
assert!(result.is_err());
let err = result.unwrap_err();
assert!(err.is_kind_of(ruby, ruby.exception_zero_div_error()));
assert!(err.message().unwrap().contains("Division by zero"));

// ❌ BAD: Just checking for any error without specific type
// assert!(result.is_err());
```

##### Testing Different Ruby Exception Types

```rust
// Testing for ArgumentError
fn test_argument_error() -> Result<(), Error> {
    let ruby = Ruby::get()?;
    let calc = Calculator::new();
    
    // Function that raises ArgumentError on negative input
    let result = Calculator::sqrt(&ruby, &calc, -1.0);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(err.is_kind_of(ruby, ruby.exception_arg_error()));
    assert!(err.message().unwrap().contains("must be positive"));
    
    Ok(())
}

// Testing for RangeError
fn test_range_error() -> Result<(), Error> {
    let ruby = Ruby::get()?;
    let calc = Calculator::new();
    
    // Function that raises RangeError on large values
    let result = Calculator::factorial(&ruby, &calc, 100);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(err.is_kind_of(ruby, ruby.exception_range_error()));
    
    Ok(())
}

// Testing for TypeError
fn test_type_error() -> Result<(), Error> {
    let ruby = Ruby::get()?;
    
    // Use eval to create a type error situation
    let result: Result<i64, Error> = ruby.eval("'string' + 5");
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(err.is_kind_of(ruby, ruby.exception_type_error()));
    
    Ok(())
}
```

##### Testing Ruby Exceptions Using eval

You can also test how Ruby exceptions are raised and handled using `eval`:

```rust
#[ruby_test]
fn test_ruby_exceptions_with_eval() -> Result<(), Error> {
    let ruby = Ruby::get()?;
    
    // Set up our extension
    let module = ruby.define_module("MyModule")?;
    let calc_class = module.define_class("Calculator", ruby.class_object())?;
    calc_class.define_singleton_method("new", function!(Calculator::new, 0))?;
    calc_class.define_method("divide", method!(Calculator::divide, 2))?;
    
    // Test division by zero from Ruby code
    let result: Result<f64, Error> = ruby.eval("MyModule::Calculator.new.divide(10, 0)");
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(err.is_kind_of(ruby, ruby.exception_zero_div_error()));
    
    Ok(())
}
```

##### Verifying Custom Exception Types

For custom exception classes:

```rust
#[ruby_test]
fn test_custom_exception() -> Result<(), Error> {
    let ruby = Ruby::get()?;
    
    // Create a custom exception class
    let module = ruby.define_module("MyModule")?;
    let custom_error = module.define_class("CustomError", ruby.exception_standard_error())?;
    
    // Define a method that raises our custom error
    let obj = ruby.eval::<Value>("Object.new")?;
    obj.define_singleton_method(ruby, "raise_custom", 
        function!(|ruby: &Ruby| -> Result<(), Error> {
            Err(Error::new(
                ruby.class_path_to_value("MyModule::CustomError"),
                "Custom error message"
            ))
        }, 0)
    )?;
    
    // Call the method and verify the exception
    let result: Result<(), Error> = ruby.eval("Object.new.raise_custom");
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(err.is_kind_of(ruby, custom_error));
    assert!(err.message().unwrap().contains("Custom error"));
    
    Ok(())
}
```

#### Pattern: Proper Memory Management

```rust
// ✅ GOOD: Test with GC stress to catch memory issues
#[ruby_test(gc_stress)]
fn test_memory_management() {
    // Test code here will run with GC stress enabled
}

// ✅ GOOD: Ensure objects used in raw C API are protected
unsafe {
    let rb_str = rb_utf8_str_new_cstr("hello\0".as_ptr() as _);
    let rb_str = rb_gc_guard!(rb_str);  // Protected from GC
}

// ❌ BAD: Using raw pointers without protection
// unsafe {
//     let rb_str = rb_utf8_str_new_cstr("hello\0".as_ptr() as _);
//     // rb_str could be collected here if GC runs
// }
```

#### Pattern: Version-Specific Testing

```rust
// ✅ GOOD: Conditional tests based on Ruby version
#[ruby_test]
fn test_features() {
    #[cfg(ruby_gte_3_0)]
    {
        // Test Ruby 3.0+ specific features
    }
    
    #[cfg(not(ruby_gte_3_0))]
    {
        // Test for older Ruby versions
    }
}

// ❌ BAD: Runtime checks for version
// if ruby_version() >= (3, 0, 0) {
//     // Test Ruby 3.0+ specific features
// }
```

### Testing Best Practices

<div class="warning">

Failing to follow these practices can result in segmentation faults, memory leaks, and other serious issues that may only appear in production environments with specific data or Ruby versions.

</div>

1. **Use `#[ruby_test]` for most tests**: This macro handles Ruby VM setup automatically.
2. **Consider Magnus for cleaner tests**: Magnus offers a much more ergonomic API than raw rb-sys.
3. **Enable `gc_stress` for memory management tests**: This helps catch GC-related bugs early.
4. **Always protect raw Ruby pointers**: Use `rb_gc_guard!` when you need to use raw pointers.
5. **Catch exceptions properly**: Don't let Ruby exceptions crash your tests.
6. **Use conditional compilation for version-specific tests**: Leverage the version flags from rb-sys-env.
7. **Test edge cases**: Nil values, empty strings, large numbers, etc.
8. **Use helper macros**: Convert between Ruby and Rust types using provided helpers.

<div class="tip">

**Code Example: Testing With Best Practices**

```rust,hidelines=#
# use magnus::{class, eval, function, method, prelude::*, Error, Ruby, Value};
# use rb_sys_test_helpers::ruby_test;
# use std::cell::RefCell;
# 
# // Define a struct with interior mutability
# struct Counter {
#     count: i64,
# }
# 
# #[magnus::wrap(class = "MyExtension::Counter")]
# struct MutCounter(RefCell<Counter>);
# 
# impl MutCounter {
#     fn new(initial: i64) -> Self {
#         Self(RefCell::new(Counter { count: initial }))
#     }
#     
#     fn count(&self) -> i64 {
#         self.0.borrow().count
#     }
#     
#     fn increment(&self) -> i64 {
#         let mut counter = self.0.borrow_mut();
#         counter.count += 1;
#         counter.count
#     }
#     
#     // Method that uses Ruby VM to potentially raise exceptions
#     fn add_checked(ruby: &Ruby, rb_self: &Self, val: i64) -> Result<i64, Error> {
#         // ✅ GOOD: Complete borrow before starting a new one
#         let current = rb_self.0.borrow().count;
#         
#         if let Some(sum) = current.checked_add(val) {
#             rb_self.0.borrow_mut().count = sum;
#             Ok(sum)
#         } else {
#             Err(Error::new(
#                 ruby.exception_range_error(),
#                 "Addition would overflow"
#             ))
#         }
#     }
# }
# 
# // Comprehensive test suite following best practices
# #[cfg(test)]
# mod tests {
#     use super::*;
#     
#     // ✅ GOOD: Basic functionality test
#     #[ruby_test]
#     fn test_counter_basic() {
#         let counter = MutCounter::new(0);
#         assert_eq!(counter.count(), 0);
#         assert_eq!(counter.increment(), 1);
#         assert_eq!(counter.increment(), 2);
#     }
#     
#     // ✅ GOOD: Test with Ruby exceptions
#     #[ruby_test]
#     fn test_counter_overflow() {
#         let ruby = Ruby::get().unwrap();
#         let counter = MutCounter::new(i64::MAX);
#         
#         // Test method that might raise Ruby exception
#         let result = MutCounter::add_checked(&ruby, &counter, 1);
#         assert!(result.is_err());
#         
#         // ✅ GOOD: Check specific exception type
#         let err = result.unwrap_err();
#         assert!(err.is_kind_of(ruby, ruby.exception_range_error()));
#     }
#     
#     // ✅ GOOD: GC stress testing to catch memory issues
#     #[ruby_test(gc_stress)]
#     fn test_with_gc_stress() {
#         let ruby = Ruby::get().unwrap();
#         let counter = MutCounter::new(0);
#         
#         // Register Ruby class for testing from Ruby
#         let class = ruby.define_class("Counter", ruby.class_object()).unwrap();
#         class.define_singleton_method("new", function!(MutCounter::new, 1)).unwrap();
#         class.define_method("increment", method!(MutCounter::increment, 0)).unwrap();
#         
#         // Access from Ruby (with GC stress active)
#         let result: i64 = ruby.eval(
#             "counter = Counter.new(5); counter.increment; counter.increment"
#         ).unwrap();
#         
#         assert_eq!(result, 7);
#     }
#     
#     // ✅ GOOD: Version-specific tests
#     #[ruby_test]
#     fn test_version_specific() {
#         #[cfg(ruby_gte_3_0)]
#         {
#             // Test Ruby 3.0+ specific features
#         }
#         
#         #[cfg(all(ruby_gte_2_7, ruby_lt_3_0))]
#         {
#             // Test Ruby 2.7 specific features
#         }
#     }
# }
```

This example illustrates proper handling of RefCell borrowing, Ruby exceptions, GC stress testing, and version-specific tests.

</div>

### Example: Complete Test Module

Here's a complete end-to-end example based on the rusty_calculator extension. This includes the project structure, required files, and comprehensive test module:

#### Project Setup

First, ensure your project has the correct file structure:

```
my_extension/
├── Cargo.toml
├── build.rs
├── src/
│   └── lib.rs
```

#### Cargo.toml

```toml
[package]
name = "my_extension"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
magnus = "0.6"
rb-sys = "0.9"

[dev-dependencies]
rb-sys-env = "0.1"
rb-sys-test-helpers = "0.2"
```

#### build.rs

```rust
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Activate rb-sys-env to set up Ruby environment for both builds and tests
    let _ = rb_sys_env::activate()?;
    
    Ok(())
}
```

#### lib.rs

This example includes a calculator class with a method that can potentially raise a Ruby exception:

```rust
use std::cell::RefCell;
use magnus::{function, method, prelude::*, wrap, Error, Ruby};

// Calculator struct with memory
struct Calculator {
    memory: f64,
}

#[wrap(class = "MyExtension::Calculator")]
struct MutCalculator(RefCell<Calculator>);

impl MutCalculator {
    // Constructor
    fn new() -> Self {
        Self(RefCell::new(Calculator { memory: 0.0 }))
    }
    
    // Basic arithmetic that returns a Result which can generate Ruby exceptions
    fn divide(ruby: &Ruby, _rb_self: &Self, a: f64, b: f64) -> Result<f64, Error> {
        if b == 0.0 {
            return Err(Error::new(
                ruby.exception_zero_div_error(),
                "Division by zero"
            ));
        }
        Ok(a / b)
    }
    
    // Regular instance method
    fn add(&self, a: f64, b: f64) -> f64 {
        a + b
    }
    
    // Memory operations using RefCell
    fn store(&self, value: f64) -> f64 {
        self.0.borrow_mut().memory = value;
        value
    }
    
    fn recall(&self) -> f64 {
        self.0.borrow().memory
    }
}

// Module initialization
#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("MyExtension")?;
    
    // Set up the Calculator class
    let calc_class = module.define_class("Calculator", ruby.class_object())?;
    calc_class.define_singleton_method("new", function!(MutCalculator::new, 0))?;
    calc_class.define_method("divide", method!(MutCalculator::divide, 2))?;
    calc_class.define_method("add", method!(MutCalculator::add, 2))?;
    calc_class.define_method("store", method!(MutCalculator::store, 1))?;
    calc_class.define_method("recall", method!(MutCalculator::recall, 0))?;
    
    Ok(())
}

// Complete test module
#[cfg(test)]
mod tests {
    use super::*;
    use rb_sys_test_helpers::ruby_test;
    
    // Basic functionality test
    #[ruby_test]
    fn test_calculator_basic_operations() {
        let calc = MutCalculator::new();
        
        // Test regular instance method
        assert_eq!(calc.add(2.0, 3.0), 5.0);
        
        // Test memory operations
        assert_eq!(calc.store(42.0), 42.0);
        assert_eq!(calc.recall(), 42.0);
    }
    
    // Test method that raises Ruby exceptions
    #[ruby_test]
    fn test_calculator_divide() {
        let ruby = Ruby::get().unwrap();
        let calc = MutCalculator::new();
        
        // Test normal division - note the function syntax for methods
        // that take ruby and rb_self parameters
        let result = MutCalculator::divide(&ruby, &calc, 10.0, 2.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 5.0);
        
        // Test division by zero
        let result = MutCalculator::divide(&ruby, &calc, 10.0, 0.0);
        assert!(result.is_err());
        
        // Verify specific exception type
        let err = result.unwrap_err();
        assert!(err.is_kind_of(ruby, ruby.exception_zero_div_error()));
        assert!(err.message().unwrap().contains("Division by zero"));
    }
    
    // Test with GC stress for memory issues
    #[ruby_test(gc_stress)]
    fn test_calculator_with_gc_stress() {
        let calc = MutCalculator::new();
        
        // Store and recall with GC stress active
        for i in 0..100 {
            calc.store(i as f64);
            assert_eq!(calc.recall(), i as f64);
        }
        
        // No segfaults or panics means test passed
    }
    
    // Test for Ruby integration using eval
    #[ruby_test]
    fn test_ruby_integration() {
        let ruby = Ruby::get().unwrap();
        
        // Define the calculator class - this simulates what init() does
        let module = ruby.define_module("MyExtension").unwrap();
        let calc_class = module.define_class("Calculator", ruby.class_object()).unwrap();
        calc_class.define_singleton_method("new", function!(MutCalculator::new, 0)).unwrap();
        calc_class.define_method("add", method!(MutCalculator::add, 2)).unwrap();
        
        // Call methods via Ruby's eval
        let result: f64 = ruby.eval("MyExtension::Calculator.new.add(2, 3)").unwrap();
        assert_eq!(result, 5.0);
    }
}
```

This complete example demonstrates:

1. Proper project setup with required dependencies
2. A realistic implementation with potential error conditions
3. Testing various method types (regular instance methods and methods with Ruby state)
4. Testing Ruby exceptions with proper type checking
5. Memory safety testing with GC stress
6. Ruby integration testing via eval

You can adapt this template to your own extension, adding the specific functionality your project requires.
```

## Integration Testing Ruby API

<div class="tip">

Integration tests verify that your extension works correctly when called from Ruby code. Testing both in Rust and Ruby provides the most complete coverage.

</div>

Integration tests verify that your Ruby extension's API works correctly when called from Ruby code. These tests are typically written in Ruby and run using Ruby's test frameworks.

### Setting Up Ruby Tests

Most Ruby gems use Minitest or RSpec for testing. Here's how to set up integration tests with Minitest (which bundler creates by default):

```ruby,hidelines=#
# test/test_my_extension.rb
require "test_helper"

class TestMyExtension < Minitest::Test
  def setup
    # Set up test fixtures
    @calculator = MyExtension::Calculator.new
  end

  def test_basic_addition
    assert_equal 5, @calculator.add(2, 3)
  end

  def test_division_by_zero
    error = assert_raises(ZeroDivisionError) do
      @calculator.divide(10, 0)
    end
    assert_match /division by zero/i, error.message
  end

  def test_nil_handling
    # Test that nil values are properly handled
    assert_nil @calculator.process(nil)
  end
  
# # Test memory management
# def test_gc_safety
#   # Create many objects and force garbage collection
#   1000.times do |i|
#     obj = MyExtension::Calculator.new
#     obj.add(i, i)
#     
#     # Force garbage collection periodically
#     GC.start if i % 100 == 0
#   end
#   
#   # If we reach here without segfaults, the test passes
#   assert true
# end
# 
# # Test edge cases
# def test_edge_cases
#   # Test with extreme values
#   max = (2**60)
#   assert_equal max * 2, @calculator.multiply(max, 2)
#   
#   # Test with different types
#   assert_raises(TypeError) do
#     @calculator.add("string", 1)
#   end
# end
end
```

<div class="note">

Click the eye icon (<i class="fa fa-eye"></i>) to see additional tests for memory management and edge cases.

</div>

### Testing Error Handling

It's particularly important to test how your extension handles error conditions:

```ruby
def test_error_propagation
  # Test that Rust errors properly convert to Ruby exceptions
  error = assert_raises(RangeError) do
    @calculator.factorial(100) # Too large, should raise RangeError
  end
  assert_match /too large/i, error.message
end

def test_invalid_arguments
  # Test type validation
  error = assert_raises(TypeError) do
    @calculator.add("string", 3) # Should raise TypeError
  end
  assert_match /expected numeric/i, error.message
end
```

### Testing Memory Management

Test memory management by creating objects and forcing garbage collection:

```ruby
def test_gc_safety
  # Create many objects and force garbage collection
  1000.times do |i|
    obj = MyExtension::Point.new(i, i)
    
    # Force garbage collection periodically
    GC.start if i % 100 == 0
  end
  
  # If we reach here without segfaults or leaks, the test passes
  assert true
end

def test_object_references
  # Test that nested objects maintain correct references
  parent = MyExtension::Node.new("parent")
  child = MyExtension::Node.new("child")
  
  # Create relationship
  parent.add_child(child)
  
  # Force garbage collection
  GC.start
  
  # Both objects should still be valid
  assert_equal "parent", parent.name
  assert_equal "child", parent.children.first.name
end
```

## Common Testing Patterns

When testing Ruby extensions written in Rust, several patterns emerge that can help ensure correctness and stability.

### Testing Type Conversions

Type conversions between Rust and Ruby are common sources of bugs:

```rust
#[ruby_test]
fn test_type_conversions() {
    let ruby = Ruby::get().unwrap();
    
    // Test Ruby to Rust conversions
    let rb_str = RString::new(ruby, "test");
    let rb_int = Integer::from_i64(42);
    let rb_array = RArray::from_iter(ruby, vec![1, 2, 3]);
    
    // Convert to Rust types
    let rust_str: String = rb_str.to_string().unwrap();
    let rust_int: i64 = rb_int.to_i64().unwrap();
    let rust_vec: Vec<i64> = rb_array.to_vec().unwrap();
    
    // Verify conversions
    assert_eq!(rust_str, "test");
    assert_eq!(rust_int, 42);
    assert_eq!(rust_vec, vec![1, 2, 3]);
    
    // Test Rust to Ruby conversions
    let rust_str = "reverse";
    let rb_str = RString::new(ruby, rust_str);
    assert_eq!(rb_str.to_string().unwrap(), rust_str);
}
```

### Method Invocation Syntax in Tests

When testing Rust methods exposed to Ruby, it's important to understand the different invocation patterns based on the method's signature:

#### Regular Instance Methods

For methods that only take `&self` and don't interact with the Ruby VM:

```rust
// Method definition
fn count(&self) -> isize {
    self.0.borrow().count
}

// In tests - use instance method syntax
#[ruby_test]
fn test_count() {
    let counter = MutCounter::new(0);
    assert_eq!(counter.count(), 0);
}
```

#### Methods with Ruby State

For methods that require the Ruby interpreter (to raise exceptions or interact with Ruby objects):

```rust
// Method definition
fn divide(ruby: &Ruby, _rb_self: &Self, a: f64, b: f64) -> Result<f64, Error> {
    if b == 0.0 {
        return Err(Error::new(
            ruby.exception_zero_div_error(),
            "Division by zero"
        ));
    }
    Ok(a / b)
}

// In tests - use associated function syntax with explicit self parameter
#[ruby_test]
fn test_divide() {
    let ruby = Ruby::get().unwrap();
    let calc = MutCalculator::new();
    
    // CORRECT: Associated function syntax with all parameters
    let result = MutCalculator::divide(&ruby, &calc, 6.0, 2.0);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3.0);
    
    // INCORRECT: This will not compile
    // let result = calc.divide(&ruby, 6.0, 2.0);
}
```

The key difference is that when a method takes `rb_self: &Self` as a parameter (as many methods do that interact with Ruby), it's not a true instance method from Rust's perspective. In tests, you must call these using the associated function syntax, passing in the Ruby interpreter and the self reference explicitly.

### Testing RefCell Borrowing

For extensions that use `RefCell` for interior mutability, test these patterns thoroughly:

```rust
#[ruby_test]
fn test_refcell_borrowing() {
    let ruby = Ruby::get().unwrap();
    let counter = MutCounter::new(0);
    
    // Test regular instance methods
    assert_eq!(counter.count(), 0);
    assert_eq!(counter.increment(), 1);
    assert_eq!(counter.increment(), 2);
    
    // Test methods that use checked operations with the Ruby VM
    // Note the use of associated function syntax here
    let result = MutCounter::add_checked(&ruby, &counter, 10);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 13);
    assert_eq!(counter.count(), 13);
}
```

### GC Stress Testing

Testing with Ruby's garbage collector is essential to ensure your extension doesn't leak memory or access deallocated objects. The `#[ruby_test(gc_stress)]` option helps identify these issues early by running the garbage collector more frequently.

#### Basic GC Stress Testing

```rust
#[ruby_test(gc_stress)]
fn test_gc_integration() {
    let ruby = Ruby::get().unwrap();
    
    // Create objects that should be properly managed
    for i in 0..100 {
        let obj = SomeObject::new(i);
        // obj goes out of scope here, should be collected
    }
    
    // Force garbage collection explicitly
    ruby.gc_start();
    
    // No panics or segfaults means the test passes
}
```

#### Testing with TypedData and Mark Methods

For custom classes that hold Ruby object references, test the `mark` method implementation:

```rust
use magnus::{gc::Marker, TypedData, DataTypeFunctions, Value};

// A struct that holds references to Ruby objects
#[derive(TypedData)]
#[magnus(class = "MyExtension::Container", free_immediately, mark)]
struct Container {
    item: Value,
    metadata: Value,
}

impl DataTypeFunctions for Container {
    fn mark(&self, marker: &Marker) {
        marker.mark(self.item);
        marker.mark(self.metadata);
    }
}

impl Container {
    fn new(item: Value, metadata: Value) -> Self {
        Self { item, metadata }
    }
    
    fn item(&self) -> Value {
        self.item
    }
}

// Test with GC stress
#[ruby_test(gc_stress)]
fn test_container_mark_method() {
    let ruby = Ruby::get().unwrap();
    
    // Create Ruby strings
    let item = RString::new(ruby, "Test Item");
    let metadata = RString::new(ruby, "Item Description");
    
    // Create our container
    let container = Container::new(item.as_value(), metadata.as_value());
    
    // Force garbage collection
    ruby.gc_start();
    
    // The items should still be accessible and not garbage collected
    let retrieved_item = container.item();
    let item_str: String = RString::from_value(retrieved_item).unwrap().to_string().unwrap();
    
    assert_eq!(item_str, "Test Item");
}
```

#### Testing Object References After GC

This test ensures objects referenced by your extension aren't prematurely collected:

```rust
#[ruby_test(gc_stress)]
fn test_object_references_survive_gc() {
    let ruby = Ruby::get().unwrap();
    
    // Create a struct holding references to other objects
    #[derive(TypedData)]
    #[magnus(class = "Node", free_immediately, mark)]
    struct Node {
        value: Value,
        children: Vec<Value>,
    }
    
    impl DataTypeFunctions for Node {
        fn mark(&self, marker: &Marker) {
            marker.mark(self.value);
            for child in &self.children {
                marker.mark(*child);
            }
        }
    }
    
    impl Node {
        fn new(value: Value) -> Self {
            Self { value, children: Vec::new() }
        }
        
        fn add_child(&mut self, child: Value) {
            self.children.push(child);
        }
        
        fn child_values(&self, ruby: &Ruby) -> Result<Vec<String>, Error> {
            let mut result = Vec::new();
            for child in &self.children {
                let str = RString::from_value(*child)?;
                result.push(str.to_string()?);
            }
            Ok(result)
        }
    }
    
    // Create the parent node
    let parent_value = RString::new(ruby, "Parent");
    let mut parent = Node::new(parent_value.as_value());
    
    // Add many child nodes
    for i in 0..20 {
        let child = RString::new(ruby, format!("Child {}", i));
        parent.add_child(child.as_value());
    }
    
    // Run garbage collection multiple times
    for _ in 0..5 {
        ruby.gc_start();
    }
    
    // Verify all children are still accessible
    let child_values = parent.child_values(ruby).unwrap();
    assert_eq!(child_values.len(), 20);
    assert_eq!(child_values[0], "Child 0");
    assert_eq!(child_values[19], "Child 19");
}
```

#### Testing Memory Safety with Raw Pointers

If your extension uses raw C API functions, test with gc_stress and use `rb_gc_guard!`:

```rust
use rb_sys::*;

#[ruby_test(gc_stress)]
fn test_raw_pointer_safety() {
    unsafe {
        // Create Ruby values
        let rb_ary = rb_ary_new();
        
        // IMPORTANT: Protect from GC
        let rb_ary = rb_gc_guard!(rb_ary);
        
        // Add items to the array
        for i in 0..10 {
            let rb_str = rb_utf8_str_new_cstr(format!("item {}\0", i).as_ptr() as _);
            
            // IMPORTANT: Protect each string from GC
            let rb_str = rb_gc_guard!(rb_str);
            
            rb_ary_push(rb_ary, rb_str);
        }
        
        // Force GC
        rb_gc();
        
        // Array should still have 10 elements
        assert_eq!(rb_ary_len(rb_ary), 10);
    }
}
```

## Test Helpers and Utilities

rb-sys-test-helpers provides various utilities to make testing easier.

### Value Conversion Helpers

These macros help with common conversions when testing:

```rust
use rb_sys_test_helpers::{rstring_to_string, rarray_to_vec};

#[ruby_test]
fn test_with_conversion_helpers() {
    unsafe {
        // Create Ruby objects
        let rb_str = rb_utf8_str_new_cstr("hello\0".as_ptr() as _);
        let rb_ary = rb_ary_new();
        rb_ary_push(rb_ary, rb_utf8_str_new_cstr("one\0".as_ptr() as _));
        rb_ary_push(rb_ary, rb_utf8_str_new_cstr("two\0".as_ptr() as _));
        
        // Convert to Rust using helpers
        let rust_str = rstring_to_string!(rb_str);
        let rust_vec = rarray_to_vec!(rb_ary, String);
        
        // Verify conversions
        assert_eq!(rust_str, "hello");
        assert_eq!(rust_vec, vec!["one".to_string(), "two".to_string()]);
    }
}
```

### Exception Handling Helpers

The `protect` function simplifies handling Ruby exceptions:

```rust
use rb_sys_test_helpers::protect;

#[ruby_test]
fn test_exception_handling() {
    // Try an operation that might raise an exception
    let result = unsafe {
        protect(|| {
            // Ruby operation that might raise
            rb_sys::rb_funcall(
                rb_sys::rb_cObject,
                rb_sys::rb_intern("nonexistent_method\0".as_ptr() as _),
                0
            )
        })
    };
    
    // Verify we got an exception
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message().unwrap().contains("undefined method"));
}
```

## CI Testing Workflow

<div class="warning">

CI testing is essential for extensions that will be distributed as gems. Without it, you risk publishing binaries that crash on specific Ruby versions or platforms.

</div>

Setting up continuous integration (CI) testing is crucial for Ruby extension gems. This section covers best practices for testing your extensions in CI environments.

### Basic GitHub Actions Setup

A simple GitHub Actions workflow for a Rust Ruby extension typically includes:

1. Setting up Ruby and Rust environments
2. Running compilation
3. Executing tests
4. Linting the code

```yaml,hidelines=#
# .github/workflows/test.yml
name: Tests

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest]
        ruby: ['3.0', '3.1', '3.2']
        
    steps:
    - uses: actions/checkout@v3
    
    # Use the setup-ruby-and-rust action from oxidize-rb
    - name: Set up Ruby and Rust
      uses: oxidize-rb/actions/setup-ruby-and-rust@v1
      with:
        ruby-version: ${{ matrix.ruby }}
        bundler-cache: true
        cargo-cache: true
    
    # Run tests
    - name: Compile and test
      run: |
        bundle exec rake compile
        bundle exec rake test
        
    # Run Rust tests
    - name: Run Rust tests
      run: cargo test --workspace

# # Windows testing job
# windows:
#   runs-on: windows-latest
#   strategy:
#     matrix:
#       ruby: ['3.1']
#   steps:
#   - uses: actions/checkout@v3
#   - name: Set up Ruby and Rust (Windows)
#     uses: oxidize-rb/actions/setup-ruby-and-rust@v1
#     with:
#       ruby-version: ${{ matrix.ruby }}
#       bundler-cache: true
#       cargo-cache: true
#   - name: Run tests
#     run: |
#       bundle exec rake compile
#       bundle exec rake test
```

<div class="note">

Click the eye icon (<i class="fa fa-eye"></i>) to see a Windows-specific job configuration.

The [oxidize-rb/actions](https://github.com/oxidize-rb/actions) repository provides specialized GitHub Actions for Ruby extensions written in Rust, making setup much simpler.

</div>

### Memory Testing with ruby_memcheck

<div class="tip">

Memory leaks can be particularly difficult to detect in Ruby extensions. Tools like ruby_memcheck help catch these issues early.

</div>

The [ruby_memcheck](https://github.com/Shopify/ruby_memcheck) gem provides a powerful way to detect memory leaks in Ruby extensions. It uses Valgrind under the hood but filters out false positives that are common when running Valgrind on Ruby code.

To use ruby_memcheck, add it to your test workflow:

```ruby,hidelines=#
# Add to your Gemfile
gem 'ruby_memcheck', group: :development

# In your Rakefile:
require 'ruby_memcheck'

test_config = lambda do |t|
  t.libs << "test"
  t.test_files = FileList["test/**/*_test.rb"]
end

namespace :test do
  RubyMemcheck::TestTask.new(valgrind: test_config)
end

# # Advanced configuration
# RubyMemcheck.config do |config|
#   # Adjust valgrind options
#   config.valgrind_options += ["--leak-check=full", "--show-leak-kinds=all"]
#   
#   # Specify custom suppression files
#   config.valgrind_suppression_files << "my_suppressions.supp"
#   
#   # Skip specific Ruby functions
#   config.skipped_ruby_functions << /my_custom_allocator/
# end
```

To run memory tests:
```bash
# Install valgrind first if needed
# sudo apt-get install valgrind  # On Debian/Ubuntu

# Run the tests with memory checking
bundle exec rake test:valgrind
```

<div class="note">

Click the eye icon (<i class="fa fa-eye"></i>) to see advanced configuration options for ruby_memcheck.

For more detailed instructions and configuration options, refer to the [ruby_memcheck documentation](https://github.com/Shopify/ruby_memcheck).

</div>

### Cross-Platform Testing with rb-sys-dock

For testing across different platforms, [rb-sys-dock](https://github.com/oxidize-rb/rb-sys-dock) provides Docker images pre-configured for cross-platform compilation and testing of Rust Ruby extensions.

### Best Practices for CI Testing

<div class="warning">

Without thorough CI testing across all supported platforms and Ruby versions, your extension may work perfectly in your development environment but crash for users with different setups.

</div>

1. **Test Matrix**: Test against multiple Ruby versions, Rust versions, and platforms
2. **Memory Testing**: Include memory leak detection with ruby_memcheck
3. **Linting**: Validate code formatting and catch Rust warnings
4. **Cross-Platform**: Test on all platforms you aim to support
5. **Documentation Verification**: Test code examples in documentation

<div class="tip">

The [oxidize-rb/actions](https://github.com/oxidize-rb/actions) repository provides ready-to-use GitHub Actions for:
- Setting up Ruby and Rust environments
- Building native gems
- Cross-compiling for multiple platforms 
- Running tests and linting checks

Using these specialized actions will save you time and ensure your tests follow best practices.

</div>