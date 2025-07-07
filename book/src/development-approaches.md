# Development Approaches

When building Ruby extensions with Rust and rb-sys, you have two main approaches to choose from:

1. **Direct rb-sys usage**: Working directly with Ruby's C API through the rb-sys bindings
2. **Higher-level wrappers**: Using libraries like Magnus that build on top of rb-sys

This chapter will help you understand when to use each approach and how to mix them when needed.

## Direct rb-sys Usage

The rb-sys crate provides low-level bindings to Ruby's C API. This approach gives you complete control over how your
Rust code interacts with Ruby.

### When to Use Direct rb-sys

- When you need maximum control over Ruby VM interaction
- For specialized extensions that need access to low-level Ruby internals
- When performance is absolutely critical and you need to eliminate any overhead
- When implementing functionality not yet covered by higher-level wrappers

### Example: Simple Extension with Direct rb-sys

Here's a simple example of a Ruby extension using direct rb-sys:

```rust
use rb_sys::{
    rb_define_module, rb_define_module_function, rb_str_new_cstr,
    rb_string_value_cstr, VALUE
};
use std::ffi::CString;
use std::os::raw::c_char;

// Helper macro for creating C strings
macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const c_char
    };
}

// Reverse a string
unsafe extern "C" fn reverse(_: VALUE, s: VALUE) -> VALUE {
    let c_str = rb_string_value_cstr(&s);
    let rust_str = std::ffi::CStr::from_ptr(c_str).to_str().unwrap();
    let reversed = rust_str.chars().rev().collect::<String>();

    let c_string = CString::new(reversed).unwrap();
    rb_str_new_cstr(c_string.as_ptr())
}

// Module initialization function
#[no_mangle]
pub extern "C" fn Init_string_utils() {
    unsafe {
        let module = rb_define_module(cstr!("StringUtils"));

        rb_define_module_function(
            module,
            cstr!("reverse"),
            Some(reverse as unsafe extern "C" fn(VALUE, VALUE) -> VALUE),
            1,
        );
    }
}
```

### Using rb_thread_call_without_gvl for Performance

When performing computationally intensive operations, it's important to release Ruby's Global VM Lock (GVL) to allow
other threads to run. The `rb_thread_call_without_gvl` function provides this capability:

```rust
use magnus::{Error, Ruby, RString};
use rb_sys::rb_thread_call_without_gvl;
use std::{ffi::c_void, panic::{self, AssertUnwindSafe}, ptr::null_mut};

/// Execute a function without holding the Global VM Lock (GVL).
/// This allows other Ruby threads to run while performing CPU-intensive tasks.
///
/// # Safety
///
/// The passed function must not interact with the Ruby VM or Ruby objects
/// as it runs without the GVL, which is required for safe Ruby operations.
///
/// # Returns
///
/// Returns the result of the function or a magnus::Error if the function panics.
pub fn nogvl<F, R>(func: F) -> Result<R, Error>
where
    F: FnOnce() -> R,
    R: Send + 'static,
{
    struct CallbackData<F, R> {
        func: Option<F>,
        result: Option<Result<R, String>>, // Store either the result or a panic message
    }

    extern "C" fn call_without_gvl<F, R>(data: *mut c_void) -> *mut c_void
    where
        F: FnOnce() -> R,
        R: Send + 'static,
    {
        // Safety: We know this pointer is valid because we just created it below
        let data = unsafe { &mut *(data as *mut CallbackData<F, R>) };

        // Use take() to move out of the Option, ensuring we don't try to run the function twice
        if let Some(func) = data.func.take() {
            // Use panic::catch_unwind to prevent Ruby process termination if the Rust code panics
            match panic::catch_unwind(AssertUnwindSafe(func)) {
                Ok(result) => data.result = Some(Ok(result)),
                Err(panic_info) => {
                    // Convert panic info to a string message
                    let panic_msg = if let Some(s) = panic_info.downcast_ref::<&'static str>() {
                        s.to_string()
                    } else if let Some(s) = panic_info.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "Unknown panic occurred in Rust code".to_string()
                    };

                    data.result = Some(Err(panic_msg));
                }
            }
        }

        null_mut()
    }

    // Create a data structure to pass the function and receive the result
    let mut data = CallbackData {
        func: Some(func),
        result: None,
    };

    unsafe {
        // Release the GVL and call our function
        rb_thread_call_without_gvl(
            Some(call_without_gvl::<F, R>),
            &mut data as *mut _ as *mut c_void,
            None,  // No unblock function
            null_mut(),
        );
    }

    // Extract the result or create an error if the function failed
    match data.result {
        Some(Ok(result)) => Ok(result),
        Some(Err(panic_msg)) => {
            // Convert the panic message to a Ruby RuntimeError
            let ruby = unsafe { Ruby::get_unchecked() };
            Err(Error::new(
                ruby.exception_runtime_error(),
                format!("Rust panic in nogvl: {}", panic_msg)
            ))
        },
        None => {
            // This should never happen if the callback runs, but handle it anyway
            let ruby = unsafe { Ruby::get_unchecked() };
            Err(Error::new(
                ruby.exception_runtime_error(),
                "nogvl function was not executed"
            ))
        }
    }
}

// For checking large inputs
pub fn nogvl_if_large<F, R>(input_len: usize, func: F) -> Result<R, Error>
where
    F: FnOnce() -> R,
    R: Send + 'static,
{
    const MAX_INPUT_LEN: usize = 8192; // Threshold for using GVL release

    if input_len > MAX_INPUT_LEN {
        nogvl(func)
    } else {
        // If the input is small, just run the function directly
        // but still wrap the result in a Result for consistency
        match panic::catch_unwind(AssertUnwindSafe(func)) {
            Ok(result) => Ok(result),
            Err(_) => {
                let ruby = unsafe { Ruby::get_unchecked() };
                Err(Error::new(
                    ruby.exception_runtime_error(),
                    "Rust panic in small input path"
                ))
            }
        }
    }
}

// Example: Using with Magnus API
fn compress(ruby: &Ruby, data: RString) -> Result<RString, Error> {
    let data_bytes = data.as_bytes();
    let data_len = data_bytes.len();

    // Use nogvl_if_large with proper error handling
    let compressed_bytes = nogvl_if_large(data_len, || {
        // CPU-intensive operation here that returns a Vec<u8>
        compression_algorithm(data_bytes)
    })?; // Propagate any errors

    // Create new Ruby string with compressed data
    let result = RString::from_slice(ruby, &compressed_bytes);
    Ok(result)
}

// Example: Registering the method
#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Compression")?;

    // Using method! for defining instance methods
    module.define_singleton_method("compress", function!(compress, 1))?;

    Ok(())
}
```

### How Direct rb-sys Works

When using rb-sys directly:

1. You define C-compatible functions with the `extern "C"` calling convention
2. You manually convert between Ruby's `VALUE` type and Rust types
3. You're responsible for memory management and type safety
4. You must use the `#[no_mangle]` attribute on the initialization function so Ruby can find it
5. All interactions with Ruby data happen through raw pointers and unsafe code

## Higher-level Wrappers (Magnus)

Magnus provides a more ergonomic, Rust-like API on top of rb-sys. It handles many of the unsafe aspects of Ruby
integration for you.

### When to Use Magnus

- For most standard Ruby extensions where ease of development is important
- When you want to avoid writing unsafe code
- When you want idiomatic Rust error handling
- For extensions with complex type conversions
- When working with Ruby classes and objects in an object-oriented way

### Example: Simple Extension with Magnus

Let's look at a simple example using Magnus, based on real-world usage patterns:

```rust
use magnus::{function, prelude::*, Error, Ruby};

fn hello(subject: String) -> String {
    format!("Hello from Rust, {subject}!")
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("StringUtils")?;
    module.define_singleton_method("hello", function!(hello, 1))?;
    Ok(())
}
```

Looking at a more complex example from a real-world project (lz4-flex-rb):

```rust
use magnus::{function, prelude::*, Error, RModule, Ruby};

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Lz4Flex")?;

    // Define error classes
    let base_error = module.define_error("Error", magnus::exception::standard_error())?;
    let _ = module.define_error("EncodeError", base_error)?;
    let _ = module.define_error("DecodeError", base_error)?;

    // Define methods
    module.define_singleton_method("compress", function!(compress, 1))?;
    module.define_singleton_method("decompress", function!(decompress, 1))?;

    // Define aliases
    module.singleton_class()?.define_alias("deflate", "compress")?;
    module.singleton_class()?.define_alias("inflate", "decompress")?;

    // Define nested module
    let varint_module = module.define_module("VarInt")?;
    varint_module.define_singleton_method("compress", function!(compress_varint, 1))?;
    varint_module.define_singleton_method("decompress", function!(decompress_varint, 1))?;

    Ok(())
}
```

### How Magnus Works

Magnus builds on top of rb-sys and provides:

1. Automatic type conversions between Ruby and Rust
2. Rust-like error handling with `Result` types
3. Memory safety through RAII patterns
4. More ergonomic APIs for defining modules, classes, and methods
5. A more familiar development experience for Rust programmers

## When to Choose Each Approach

### Choose Direct rb-sys When:

- **Performance is absolutely critical**: You need to eliminate every bit of overhead
- **You need low-level control**: Your extension needs to do things not possible with Magnus
- **GVL management is important**: You need fine-grained control over when to release the GVL
- **Compatibility with older Ruby versions**: You need version-specific behavior

### Choose Magnus When:

- **Developer productivity is important**: You want to write less code
- **Memory safety is a priority**: You want Rust's safety guarantees
- **You're working with complex Ruby objects**: You need convenient methods for Ruby class integration
- **Error handling is complex**: You want to leverage Rust's error handling

## Mixing Approaches

You can also mix the two approaches when appropriate. Magnus provides access to the underlying rb-sys functionality when
needed:

```rust
use magnus::{function, prelude::*, Error, Ruby};
use rb_sys;
use std::os::raw::c_char;

fn high_level() -> String {
    "High level".to_string()
}

unsafe extern "C" fn low_level(_: rb_sys::VALUE) -> rb_sys::VALUE {
    // Direct rb-sys implementation
    let c_string = std::ffi::CString::new("Low level").unwrap();
    rb_sys::rb_str_new_cstr(c_string.as_ptr())
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("MixedExample")?;

    // Use Magnus for most things
    module.define_singleton_method("high_level", function!(high_level, 0))?;

    // Use rb-sys directly for special cases
    unsafe {
        rb_sys::rb_define_module_function(
            module.as_raw(),
            cstr!("low_level"),
            Some(low_level as unsafe extern "C" fn(rb_sys::VALUE) -> rb_sys::VALUE),
            0,
        );
    }

    Ok(())
}

// Helper macro for C strings
macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const c_char
    };
}
```

### Enabling rb-sys Feature in Magnus

To access rb-sys through Magnus, enable the `rb-sys` feature:

```toml
# Cargo.toml
[dependencies]
magnus = { version = "0.7", features = ["rb-sys"] }
```

### Common Mixing Patterns

1. **Use Magnus for most functionality, rb-sys for specific optimizations**:
   - Define your public API using Magnus for safety and ease
   - Drop down to rb-sys in critical performance paths, especially when using `nogvl`

2. **Use rb-sys for core functionality, Magnus for complex conversions**:
   - Build core functionality with rb-sys for maximum control
   - Use Magnus for handling complex Ruby objects or collections

3. **Start with Magnus, optimize with rb-sys over time**:
   - Begin development with Magnus for rapid progress
   - Profile your code and replace hot paths with direct rb-sys

## Real-World Examples

Let's look at how real projects decide between these approaches:

### Blake3-Ruby (Direct rb-sys)

Blake3-Ruby is a cryptographic hashing library that uses direct rb-sys to achieve maximum performance:

```rust
// Based on blake3-ruby
use rb_sys::{
    rb_define_module, rb_define_module_function, rb_string_value_cstr,
    rb_str_new_cstr, VALUE,
};

#[no_mangle]
pub extern "C" fn Init_blake3_ext() {
    unsafe {
        // Create module and class hierarchy
        let digest_module = /* ... */;
        let blake3_class = /* ... */;

        // Define methods directly using rb-sys for maximum performance
        rb_define_module_function(
            blake3_class,
            cstr!("digest"),
            Some(rb_blake3_digest as unsafe extern "C" fn(VALUE, VALUE) -> VALUE),
            1,
        );

        // More method definitions...
    }
}

unsafe extern "C" fn rb_blake3_digest(_klass: VALUE, string: VALUE) -> VALUE {
    // Extract data from Ruby VALUE
    let data_ptr = rb_string_value_cstr(&string);
    let data_len = /* ... */;

    // Release GVL for CPU-intensive operation
    let hash = nogvl(|| {
        blake3::hash(/* ... */)
    });

    // Return result as Ruby string
    rb_str_new_cstr(/* ... */)
}
```

### LZ4-Flex-RB (Mixed Approach)

The LZ4-Flex-RB gem demonstrates a more sophisticated approach mixing Magnus with direct rb-sys calls:

```rust
// Based on lz4-flex-rb
use magnus::{function, prelude::*, Error, RModule, Ruby};
use rb_sys::{rb_str_locktmp, rb_str_unlocktmp, rb_thread_call_without_gvl};

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Lz4Flex")?;

    // High-level API using Magnus
    module.define_singleton_method("compress", function!(compress, 1))?;
    module.define_singleton_method("decompress", function!(decompress, 1))?;

    Ok(())
}

// Functions that mix high-level Magnus with low-level rb-sys
fn compress(input: LockedRString) -> Result<RString, Error> {
    let bufsize = get_maximum_output_size(input.len());
    let mut output = RStringMut::buf_new(bufsize);

    // Use nogvl_if_large to release GVL for large inputs
    let outsize = nogvl_if_large(input.len(), || {
        lz4_flex::block::compress_into(input.as_slice(), output.as_mut_slice())
    }).map_err(|e| Error::new(encode_error_class(), e.to_string()))?;

    output.set_len(outsize);
    Ok(output.into_inner())
}

// Helper for locked RString (uses rb-sys directly)
struct LockedRString(RString);

impl LockedRString {
    fn new(string: RString) -> Self {
        unsafe { rb_str_locktmp(string.as_raw()) };
        Self(string)
    }

    fn as_slice(&self) -> &[u8] {
        // Implementation using rb-sys functions
    }
}

impl Drop for LockedRString {
    fn drop(&mut self) {
        unsafe { rb_str_unlocktmp(self.0.as_raw()) };
    }
}
```
