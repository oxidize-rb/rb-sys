# Working with Ruby Objects

## Basic Type Conversions

When writing Ruby extensions in Rust, one of the most common tasks is converting between Ruby and Rust types. The magnus
crate provides a comprehensive set of conversion functions for this purpose.

### Primitive Types

```rust
use magnus::{RString, Ruby, Value, Integer, Float, Boolean};

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), magnus::Error> {
    // Convert Rust types to Ruby
    let rb_string: RString = Ruby::str_new(ruby, "Ruby!");  // Rust &str to Ruby String
    let rb_int: Integer = Integer::from_i64(42);            // Rust i64 to Ruby Integer
    let rb_float: Float = Float::from_f64(3.14159);         // Rust f64 to Ruby Float
    let rb_bool: Boolean = Boolean::from(true);             // Rust bool to Ruby true/false

    // Convert Ruby types to Rust
    let rust_string: String = rb_string.to_string()?;       // Ruby String to Rust String
    let rust_int: i64 = rb_int.to_i64()?;                   // Ruby Integer to Rust i64
    let rust_float: f64 = rb_float.to_f64()?;               // Ruby Float to Rust f64
    let rust_bool: bool = rb_bool.to_bool();                // Ruby true/false to Rust bool

    Ok(())
}
```

### Checking Types

When working with Ruby objects, you often need to check their types:

```rust
use magnus::{RString, Ruby, Value, check_type};

fn process_value(ruby: &Ruby, val: Value) -> Result<(), magnus::Error> {
    if val.is_nil() {
        println!("Got nil");
    } else if let Ok(s) = RString::try_convert(val) {
        println!("Got string: {}", s.to_string()?);
    } else if check_type::<Integer>(val) {
        println!("Got integer: {}", Integer::from_value(val)?.to_i64()?);
    } else {
        println!("Got some other type");
    }

    Ok(())
}
```

## Strings, Arrays, and Hashes

### Working with Ruby Strings

Ruby strings are encoded and have more complex behavior than Rust strings:

```rust
use magnus::{RString, Ruby, Encoding};

fn string_operations(ruby: &Ruby) -> Result<(), magnus::Error> {
    // Create a new Ruby string
    let hello = RString::new(ruby, "Hello");

    // Concatenate strings
    let world = RString::new(ruby, " World!");
    let message = hello.concat(ruby, world)?;

    // Get the encoding
    let encoding = message.encoding();
    println!("String encoding: {}", encoding.name());

    // Convert to different encoding
    let utf16 = Encoding::find("UTF-16BE").unwrap();
    let utf16_str = message.encode(ruby, utf16, None)?;

    // Get bytes
    let bytes = message.as_bytes();
    println!("Bytes: {:?}", bytes);

    // Create from bytes with specific encoding
    let latin1 = Encoding::find("ISO-8859-1").unwrap();
    let bytes = [72, 101, 108, 108, 111]; // "Hello" in ASCII/Latin1
    let latin1_str = RString::from_slice(ruby, &bytes, Some(latin1));

    Ok(())
}
```

### Working with Ruby Arrays

Ruby arrays can hold any kind of Ruby object:

```rust
use magnus::{RArray, Ruby, Value};

fn array_operations(ruby: &Ruby) -> Result<(), magnus::Error> {
    // Create a new empty array
    let array = RArray::new(ruby);

    // Push elements
    array.push(ruby, 1)?;
    array.push(ruby, "two")?;
    array.push(ruby, 3.0)?;

    // Get length
    let length = array.len();
    println!("Array length: {}", length);

    // Access elements
    let first: i64 = array.get(0)?;
    let second: String = array.get(1)?;
    let third: f64 = array.get(2)?;

    // Iterate through elements
    for i in 0..array.len() {
        let item: Value = array.get(i)?;
        println!("Item {}: {:?}", i, item);
    }

    // Another way to iterate
    array.each(|val| {
        println!("Item: {:?}", val);
        Ok(())
    })?;

    // Create an array from Rust Vec
    let numbers = vec![1, 2, 3, 4, 5];
    let rb_array = RArray::from_iter(ruby, numbers);

    // Convert to a Rust Vec
    let vec: Vec<i64> = rb_array.to_vec()?;

    Ok(())
}
```

### Working with Ruby Hashes

Ruby hashes are similar to Rust's HashMap but can use any Ruby object as keys:

```rust
use magnus::{RHash, Value, Symbol, Ruby};

fn hash_operations(ruby: &Ruby) -> Result<(), magnus::Error> {
    // Create a new hash
    let hash = RHash::new(ruby);

    // Add key-value pairs
    hash.aset(ruby, "name", "Alice")?;
    hash.aset(ruby, Symbol::new("age"), 30)?;
    hash.aset(ruby, 1, "one")?;

    // Get values
    let name: String = hash.get(ruby, "name")?;
    let age: i64 = hash.get(ruby, Symbol::new("age"))?;
    let one: String = hash.get(ruby, 1)?;

    // Check if key exists
    if hash.has_key(ruby, "name")? {
        println!("Has key 'name'");
    }

    // Delete a key
    hash.delete(ruby, 1)?;

    // Iterate over key-value pairs
    hash.foreach(|k, v| {
        println!("Key: {:?}, Value: {:?}", k, v);
        Ok(())
    })?;

    // Convert to a Rust HashMap (if keys and values are convertible)
    let map: std::collections::HashMap<String, String> = hash.to_hash()?;

    Ok(())
}
```

## Handling nil Values

Ruby's `nil` is a special value that requires careful handling:

```rust
use magnus::{Value, Ruby, RNil};

fn handle_nil(ruby: &Ruby, val: Value) -> Result<(), magnus::Error> {
    // Check if a value is nil
    if val.is_nil() {
        println!("Value is nil");
    }

    // Get nil
    let nil = ruby.nil();

    // Options and nil
    let maybe_string: Option<String> = val.try_convert()?;
    match maybe_string {
        Some(s) => println!("Got string: {}", s),
        None => println!("No string (was nil or couldn't convert)"),
    }

    // Explicitly return nil from a function
    fn returns_nil() -> RNil {
        RNil::get()
    }

    Ok(())
}
```

## Converting Between Ruby and Rust Types

Magnus provides powerful type conversion traits that make it easy to convert between Ruby and Rust types.

### From Rust to Ruby (TryConvert)

```rust
use magnus::{Value, Ruby, TryConvert, Error};

// Convert custom Rust types to Ruby objects
struct Person {
    name: String,
    age: u32,
}

impl TryConvert for Person {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let ruby = unsafe { Ruby::get_unchecked() };
        let hash = RHash::try_convert(val)?;

        let name: String = hash.get(ruby, "name")?;
        let age: u32 = hash.get(ruby, "age")?;

        Ok(Person { name, age })
    }
}

// Usage
fn process_person(val: Value) -> Result<(), Error> {
    let person: Person = val.try_convert()?;
    println!("Person: {} ({})", person.name, person.age);
    Ok(())
}
```

### From Ruby to Rust (IntoValue)

```rust
use magnus::{Value, Ruby, IntoValue, Error};

struct Point {
    x: f64,
    y: f64,
}

impl IntoValue for Point {
    fn into_value_with(self, ruby: &Ruby) -> Result<Value, Error> {
        let hash = RHash::new(ruby);
        hash.aset(ruby, "x", self.x)?;
        hash.aset(ruby, "y", self.y)?;
        Ok(hash.as_value())
    }
}

// Usage
fn create_point(ruby: &Ruby) -> Result<Value, Error> {
    let point = Point { x: 10.5, y: 20.7 };
    point.into_value_with(ruby)
}
```

## Creating Ruby Objects from Rust

### Creating Simple Objects

```rust
use magnus::{RObject, Ruby, Value, class, method};

fn create_objects(ruby: &Ruby) -> Result<(), magnus::Error> {
    // Create a basic Ruby Object
    let obj = RObject::new(ruby, ruby.class_object())?;

    // Instantiate a specific class
    let time_class = ruby.class_object::<Time>()?;
    let now = time_class.funcall(ruby, "now", ())?;

    // Create a Date object
    let date_class = class::object("Date")?;
    let today = date_class.funcall(ruby, "today", ())?;

    // Call methods on the object
    let formatted: String = today.funcall(ruby, "strftime", ("%Y-%m-%d",))?;

    Ok(())
}
```

### Creating Objects with Instance Variables

```rust
use magnus::{RObject, Ruby, Symbol};

fn create_with_ivars(ruby: &Ruby) -> Result<(), magnus::Error> {
    // Create a Ruby object
    let obj = RObject::new(ruby, ruby.class_object())?;

    // Set instance variables
    obj.ivar_set(ruby, "@name", "Alice")?;
    obj.ivar_set(ruby, "@age", 30)?;

    // Get instance variables
    let name: String = obj.ivar_get(ruby, "@name")?;
    let age: i64 = obj.ivar_get(ruby, "@age")?;

    // Alternatively, use symbols
    let name_sym = Symbol::new("@name");
    let name_value = obj.ivar_get(ruby, name_sym)?;

    Ok(())
}
```

### Working with Ruby Methods

```rust
use magnus::{RObject, Ruby, prelude::*};

fn call_methods(ruby: &Ruby) -> Result<(), magnus::Error> {
    let array_class = ruby.class_object::<RArray>()?;

    // Creating an array with methods
    let array = array_class.funcall(ruby, "new", (5, "hello"))?;

    // Call methods with different argument patterns
    array.funcall(ruby, "<<", ("world",))?; // One argument
    array.funcall(ruby, "insert", (1, "inserted"))?; // Multiple arguments

    // Call with a block using a closure
    let mapped = array.funcall_with_block(ruby, "map", (), |arg| {
        if let Ok(s) = String::try_convert(arg) {
            Ok(s.len())
        } else {
            Ok(0)
        }
    })?;

    // Methods with keyword arguments
    let hash_class = ruby.class_object::<RHash>()?;
    let merge_opts = [(
        Symbol::new("overwrite"),
        true
    )];
    let hash = RHash::new(ruby);
    let other = RHash::new(ruby);
    hash.funcall_kw(ruby, "merge", (other,), merge_opts)?;

    Ok(())
}
```

## Advanced Techniques

### Handling Arbitrary Ruby Values

Sometimes you need to work with Ruby values without knowing their type in advance:

```rust
use magnus::{Value, Ruby, CheckType, Error};

fn describe_value(val: Value) -> Result<String, Error> {
    let ruby = unsafe { Ruby::get_unchecked() };

    if val.is_nil() {
        return Ok("nil".to_string());
    }

    if let Ok(s) = String::try_convert(val) {
        return Ok(format!("String: {}", s));
    }

    if let Ok(i) = i64::try_convert(val) {
        return Ok(format!("Integer: {}", i));
    }

    if let Ok(f) = f64::try_convert(val) {
        return Ok(format!("Float: {}", f));
    }

    if val.respond_to(ruby, "each")? {
        return Ok("Enumerable object".to_string());
    }

    // Get the class name
    let class_name: String = val.class().name();
    Ok(format!("Object of class: {}", class_name))
}
```

### Working with Duck Types

Ruby often uses duck typing rather than relying on concrete classes:

```rust
use magnus::{Error, Value, Ruby};

fn process_enumerable(ruby: &Ruby, val: Value) -> Result<Value, Error> {
    // Check if the object responds to 'each'
    if !val.respond_to(ruby, "each")? {
        return Err(Error::new(
            ruby.exception_type_error(),
            "Expected an object that responds to 'each'"
        ));
    }

    // We can now safely call 'map' which most enumerables support
    val.funcall_with_block(ruby, "map", (), |item| {
        if let Ok(n) = i64::try_convert(item) {
            Ok(n * 2)
        } else {
            Ok(item)  // Pass through unchanged if not a number
        }
    })
}
```

## Best Practices

1. **Always Handle Errors**: Type conversions can fail, wrap them in proper error handling.

2. **Use try_convert**: Prefer `try_convert` over direct conversions to safely handle type mismatches.

3. **Remember Boxing Rules**: All Ruby objects are reference types, while many Rust types are value types.

4. **Be Careful with Magic Methods**: Some Ruby methods like `method_missing` might not behave as expected when called
   from Rust.

5. **Cache Ruby Objects**: If you're repeatedly using the same Ruby objects (like classes or symbols), consider caching
   them using `Lazy` or similar mechanisms.

6. **Check for nil**: Always check for nil values before attempting conversions that don't handle nil.

7. **Use Type Annotations**: Explicitly specifying types when converting Ruby values to Rust can make your code clearer
   and avoid potential runtime errors.

8. **Pass Ruby State**: Always pass the `Ruby` instance through your functions when needed rather than using
   `Ruby::get()` repeatedly, as this is more performant and clearer about dependencies.
