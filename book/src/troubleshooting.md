# Troubleshooting Guide

This chapter provides solutions to common issues encountered when developing Ruby extensions with rb-sys and magnus.

## Getting Started Issues

### Installation Problems

#### Missing libclang

**Problem**: Build fails with errors related to missing libclang:
```
error: failed to run custom build command for `bindgen v0.64.0`
...
could not find libclang: "couldn't find any valid shared libraries matching: ['libclang.so', ...]"
```

**Solutions**:
1. Add libclang to your Gemfile:
   ```ruby
   gem "libclang", "~> 14.0"
   ```

2. On Linux, install libclang through your package manager:
   ```bash
   # Debian/Ubuntu
   apt-get install libclang-dev
   
   # Fedora/RHEL
   dnf install clang-devel
   ```

3. On macOS:
   ```bash
   brew install llvm
   export LLVM_CONFIG=$(brew --prefix llvm)/bin/llvm-config
   ```

#### Ruby Headers Not Found

**Problem**: Build fails with error about missing Ruby headers:
```
error: failed to run custom build command for `rb-sys v0.9.78`
...
fatal error: ruby.h: No such file or directory
```

**Solutions**:
1. Ensure you have Ruby development headers installed:
   ```bash
   # Debian/Ubuntu
   apt-get install ruby-dev
   
   # Fedora/RHEL
   dnf install ruby-devel
   ```

2. If using rbenv/rvm/asdf, make sure you've installed the development headers:
   ```bash
   # For rbenv
   RUBY_CONFIGURE_OPTS=--enable-shared rbenv install 3.0.0
   
   # For rvm
   rvm install 3.0.0 --with-openssl-dir=$(brew --prefix openssl)
   ```

3. Check your build.rs file includes:
   ```rust
   let _ = rb_sys_env::activate()?;
   ```

## Compilation Issues

### Cargo Build Errors

#### Version Compatibility

**Problem**: Build fails with version incompatibility errors:
```
error: failed to select a version for the requirement `rb-sys = "^0.9.80"`
```

**Solution**: 
1. Check your magnus and rb-sys versions are compatible:
   ```toml
   # Cargo.toml
   [dependencies]
   magnus = "0.7"     # Check latest compatible version
   rb-sys = "0.9.80"  # Check latest version
   ```

2. Update rb-sys in your Gemfile:
   ```ruby
   gem 'rb_sys', '~> 0.9.80'
   ```

#### Linking Issues

**Problem**: Build fails with undefined references:
```
error: linking with `cc` failed: exit status: 1
...
undefined reference to `rb_define_module`
```

**Solutions**:
1. Ensure your build.rs is correctly set up:
   ```rust
   fn main() -> Result<(), Box<dyn std::error::Error>> {
       let _ = rb_sys_env::activate()?;
       Ok(())
   }
   ```

2. Verify Ruby version compatibility with rb-sys version
3. Ensure you have Ruby development headers installed

#### Build Script Errors

**Problem**: Build script execution fails:
```
error: failed to run custom build command for `rb-sys v0.9.78`
```

**Solutions**:
1. Check permissions and environment variables:
   ```bash
   # Set necessary environment variables
   export RUBY_ROOT=$(rbenv prefix)
   export PATH=$RUBY_ROOT/bin:$PATH
   ```

2. If using Docker, ensure the build environment has Ruby installed and configured

## Runtime Issues

### Ruby Object Management

#### Segmentation Faults

**Problem**: Ruby process crashes with a segmentation fault:
```
[BUG] Segmentation fault
ruby 3.0.0p0 (2020-12-25 revision 95aff21468) [x86_64-linux]
```

**Solutions**:
1. Ensure Ruby objects are correctly protected from garbage collection:
   ```rust
   // Using Magnus (preferred)
   let obj = RObject::new(ruby, ruby.class_object())?;
   
   // With raw rb-sys
   unsafe {
       let obj = rb_sys::rb_obj_alloc(rb_sys::rb_cObject);
       rb_sys::rb_gc_register_mark_object(obj);
       // Use obj...
       rb_sys::rb_gc_unregister_mark_object(obj);
   }
   ```

2. Check all pointers are valid before dereferencing
3. Use TypedData with proper mark implementation

#### BorrowMutError Panics

**Problem**: Your extension panics with a "already borrowed" error when using RefCell:
```
thread '<unnamed>' panicked at 'already borrowed: BorrowMutError'
```

**Solutions**:
1. Fix borrow order - always complete immutable borrows before mutable borrows:
   ```rust
   // WRONG - causes BorrowMutError
   if self.0.borrow().value > 10 {
       self.0.borrow_mut().value = 0; // Error: still borrowed from the if condition
   }
   
   // RIGHT - copy values then borrow mutably
   let current = self.0.borrow().value; // Complete this borrow
   if current > 10 {
       self.0.borrow_mut().value = 0; // Now safe to borrow mutably
   }
   ```

2. Consider restructuring your data to avoid nested borrows
3. Use separate methods for reading and writing

### Ruby/Rust Type Conversion Issues

#### Unexpected Nil Values

**Problem**: Your extension crashes when encountering nil:
```
TypeError: no implicit conversion of nil into String
```

**Solutions**:
1. Always check for nil before conversion:
   ```rust
   fn process_string(val: Value) -> Result<String, Error> {
       if val.is_nil() {
           // Handle nil case
           return Ok("default".to_string());
       }
       
       let string = RString::try_convert(val)?;
       Ok(string.to_string()?)
   }
   ```

2. Use Option for conversions that might return nil:
   ```rust
   let maybe_string: Option<String> = val.try_convert()?;
   match maybe_string {
       Some(s) => process_string(s),
       None => handle_nil_case(),
   }
   ```

#### Type Errors

**Problem**: Function fails with type mismatch errors:
```
TypeError: wrong argument type Integer (expected String)
```

**Solutions**:
1. Add explicit type checking before conversion:
   ```rust
   fn process_value(ruby: &Ruby, val: Value) -> Result<(), Error> {
       if !val.is_kind_of(ruby, ruby.class_object::<RString>()?) {
           return Err(Error::new(
               ruby.exception_type_error(),
               format!("Expected String, got {}", val.class().name())
           ));
       }
       
       let string = RString::try_convert(val)?;
       // Process string...
       Ok(())
   }
   ```

2. Use try_convert with proper error handling:
   ```rust
   match RString::try_convert(val) {
       Ok(string) => {
           // Process string...
           Ok(())
       },
       Err(_) => {
           Err(Error::new(
               ruby.exception_type_error(),
               "Expected String argument"
           ))
       }
   }
   ```

## Memory and Performance Issues

### Memory Leaks

**Problem**: Your extension gradually consumes more memory over time.

**Solutions**:
1. Ensure Ruby objects are properly released when using raw rb-sys:
   ```rust
   unsafe {
       // Register the object to protect it from GC
       rb_sys::rb_gc_register_mark_object(obj);
       
       // Use the object...
       
       // Unregister when done (important!)
       rb_sys::rb_gc_unregister_mark_object(obj);
   }
   ```

2. Implement proper mark methods for TypedData:
   ```rust
   #[derive(TypedData)]
   #[magnus(class = "MyClass", free_immediately, mark)]
   struct MyObject {
       references: Vec<Value>,
   }
   
   impl DataTypeFunctions for MyObject {
       fn mark(&self, marker: &Marker) {
           for reference in &self.references {
               marker.mark(*reference);
           }
       }
   }
   ```

3. Use ruby_memcheck to detect leaks (see [Debugging chapter](debugging.md))

### Global VM Lock (GVL) Issues

**Problem**: CPU-intensive operations block the Ruby VM.

**Solutions**:
1. Release the GVL during CPU-intensive work:
   ```rust
   use rb_sys::rb_thread_call_without_gvl;
   use std::{ffi::c_void, ptr::null_mut};
   
   pub fn nogvl<F, R>(func: F) -> R
   where
       F: FnOnce() -> R,
       R: Send + 'static,
   {
       struct CallbackData<F, R> {
           func: Option<F>,
           result: Option<R>,
       }
       
       extern "C" fn callback<F, R>(data: *mut c_void) -> *mut c_void
       where
           F: FnOnce() -> R,
           R: Send + 'static,
       {
           let data = unsafe { &mut *(data as *mut CallbackData<F, R>) };
           if let Some(func) = data.func.take() {
               data.result = Some(func());
           }
           null_mut()
       }
       
       let mut data = CallbackData {
           func: Some(func),
           result: None,
       };
       
       unsafe {
           rb_thread_call_without_gvl(
               Some(callback::<F, R>),
               &mut data as *mut _ as *mut c_void,
               None,
               null_mut(),
           );
       }
       
       data.result.unwrap()
   }
   ```

2. Only release the GVL for operations that don't interact with Ruby objects:
   ```rust
   // Safe to run without GVL - pure computation
   let result = nogvl(|| {
       compute_intensive_function(input_data)
   });
   
   // NOT safe to run without GVL - interacts with Ruby
   // let result = nogvl(|| {
   //     ruby_object.some_method() // WRONG - will crash
   // });
   ```

## Cross-Platform Issues

### Platform-Specific Build Problems

#### Windows Issues

**Problem**: Build fails on Windows with linking errors.

**Solutions**:
1. Ensure you have the correct toolchain installed:
   ```bash
   rustup target add x86_64-pc-windows-msvc
   ```

2. Add platform-specific configuration in Cargo.toml:
   ```toml
   [target.'cfg(windows)'.dependencies]
   winapi = { version = "0.3", features = ["everything"] }
   ```

3. Use conditional compilation for platform-specific code:
   ```rust
   #[cfg(windows)]
   fn platform_specific() {
       // Windows-specific code
   }
   
   #[cfg(unix)]
   fn platform_specific() {
       // Unix-specific code
   }
   ```

#### macOS Issues

**Problem**: Build fails on macOS with architecture or linking issues.

**Solutions**:
1. Specify architecture when needed:
   ```bash
   RUBY_CONFIGURE_OPTS="--with-arch=x86_64,arm64" rbenv install 3.0.0
   ```

2. Fix linking for universal binaries:
   ```rust
   // build.rs
   #[cfg(target_os = "macos")]
   {
       println!("cargo:rustc-link-arg=-arch");
       println!("cargo:rustc-link-arg=arm64");
       println!("cargo:rustc-link-arg=-arch");
       println!("cargo:rustc-link-arg=x86_64");
   }
   ```

### Cargo.toml Configuration Issues

#### Feature Flag Problems

**Problem**: Build fails because of conflicting or missing feature flags.

**Solutions**:
1. Check for feature flag issues in dependencies:
   ```toml
   [dependencies]
   magnus = { version = "0.7", features = ["rb-sys"] }
   rb-sys = { version = "0.9.80", features = ["stable-api"] }
   ```

2. Use debug prints in build.rs to check feature detection:
   ```rust
   fn main() {
       println!("cargo:warning=Ruby version: {}", rb_sys_env::ruby_major_version());
       
       #[cfg(feature = "some-feature")]
       println!("cargo:warning=some-feature is enabled");
   }
   ```

## Ruby Integration Issues

### Method Definition Problems

**Problem**: Ruby method definitions don't work as expected.

**Solutions**:
1. Check method arity and macro usage:
   ```rust
   // For instance methods (that use &self)
   class.define_method("instance_method", method!(MyClass::instance_method, 1))?;
   
   // For class/module methods (no &self)
   class.define_singleton_method("class_method", function!(class_method, 1))?;
   ```

2. Verify method signatures:
   ```rust
   // Instance method
   fn instance_method(&self, arg: Value) -> Result<Value, Error> {
       // Method implementation...
   }
   
   // Class method
   fn class_method(arg: Value) -> Result<Value, Error> {
       // Method implementation...
   }
   
   // Method with ruby
   fn method_with_ruby(ruby: &Ruby, arg: Value) -> Result<Value, Error> {
       // Method implementation...
   }
   ```

### Module/Class Hierarchy Issues

**Problem**: Ruby modules or classes aren't defined correctly.

**Solutions**:
1. Check the correct nesting of defines:
   ```rust
   // Define a module and a nested class
   let module = ruby.define_module("MyModule")?;
   let class = module.define_class("MyClass", ruby.class_object())?;
   
   // Define a nested module
   let nested = module.define_module("Nested")?;
   ```

2. Verify class inheritance:
   ```rust
   // Get the correct superclass
   let superclass = ruby.class_object::<RObject>()?;
   
   // Define a class with the superclass
   let class = ruby.define_class("MyClass", superclass)?;
   ```

## Debugging Ruby Exceptions

### Custom Exception Handling

**Problem**: Ruby exceptions aren't properly caught or raised.

**Solutions**:
1. Define and use custom exception classes:
   ```rust
   fn init(ruby: &Ruby) -> Result<(), Error> {
       let module = ruby.define_module("MyModule")?;
       
       // Define custom exceptions
       let std_error = ruby.exception_standard_error();
       let custom_error = module.define_class("CustomError", std_error)?;
       
       Ok(())
   }
   
   fn raise_custom_error(ruby: &Ruby) -> Result<(), Error> {
       Err(Error::new(
           ruby.class_path_to_value("MyModule::CustomError"),
           "Something went wrong"
       ))
   }
   ```

2. Catch specific exception types:
   ```rust
   fn handle_exceptions(ruby: &Ruby, val: Value) -> Result<Value, Error> {
       let result = val.funcall(ruby, "may_raise", ());
       
       match result {
           Ok(v) => Ok(v),
           Err(e) if e.is_kind_of(ruby, ruby.exception_zero_div_error()) => {
               // Handle division by zero
               Ok(ruby.integer_from_i64(0))
           },
           Err(e) => Err(e), // Re-raise other exceptions
       }
   }
   ```

## Additional Resources

- **Official Documentation**: [rb-sys](https://github.com/oxidize-rb/rb-sys) and [magnus](https://github.com/matsadler/magnus)
- **Examples**: Check the [examples directory](https://github.com/oxidize-rb/rb-sys/tree/main/examples) in rb-sys
- **Community Support**: [Join the Slack channel](https://join.slack.com/t/oxidize-rb/shared_invite/zt-16zv5tqte-Vi7WfzxCesdo2TqF_RYBCw)
- **Further Reading**: See the [Debugging chapter](debugging.md) for more detailed debugging techniques