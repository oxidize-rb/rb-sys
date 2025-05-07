# Ruby Classes and Modules

This chapter covers how to define and work with Ruby classes and modules from Rust. It explains different approaches for
creating Ruby objects, defining methods, and organizing your code.

## Defining Modules

Modules in Ruby are used to namespace functionality and define mixins. Here's how to create and use modules in your Rust
extension:

### Creating a Basic Module

```rust
use magnus::{define_module, prelude::*, Error, Ruby};

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    // Create a top-level module
    let module = ruby.define_module("MyExtension")?;

    // Define a method on the module
    module.define_singleton_method("version", function!(|| "1.0.0", 0))?;

    // Create a nested module
    let utils = module.define_module("Utils")?;
    utils.define_singleton_method("helper", function!(|| "Helper function", 0))?;

    Ok(())
}
```

This creates a module structure that would look like this in Ruby:

```ruby
module MyExtension
  def self.version
    "1.0.0"
  end

  module Utils
    def self.helper
      "Helper function"
    end
  end
end
```

### Module Constants

You can define constants in your modules:

```rust
use magnus::{define_module, Module, Ruby, Error, Value, Symbol};

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Config")?;

    // Define constants
    module.const_set::<_, _, Value>(ruby, "VERSION", "1.0.0")?;
    module.const_set::<_, _, Value>(ruby, "MAX_CONNECTIONS", 100)?;
    module.const_set::<_, _, Value>(ruby, "DEFAULT_MODE", Symbol::new("production"))?;

    Ok(())
}
```

### Using Module Attributes

To maintain module state, a common pattern is storing attributes in the module itself:

```rust
use magnus::{define_module, function, prelude::*, Error, Module, Ruby};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

// Store a counter in a static atomic
static REQUEST_COUNT: AtomicUsize = AtomicUsize::new(0);

// Store configuration in a mutex
static CONFIG: Mutex<Option<String>> = Mutex::new(None);

fn increment_counter() -> usize {
    REQUEST_COUNT.fetch_add(1, Ordering::SeqCst)
}

fn get_config() -> Result<String, Error> {
    match CONFIG.lock().unwrap().clone() {
        Some(config) => Ok(config),
        None => Ok("default".to_string()),
    }
}

fn set_config(value: String) -> Result<String, Error> {
    let mut config = CONFIG.lock().unwrap();
    *config = Some(value.clone());
    Ok(value)
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Stats")?;

    module.define_singleton_method("increment", function!(increment_counter, 0))?;
    module.define_singleton_method("count", function!(|| REQUEST_COUNT.load(Ordering::SeqCst), 0))?;

    // Configuration methods
    module.define_singleton_method("config", function!(get_config, 0))?;
    module.define_singleton_method("config=", function!(set_config, 1))?;

    Ok(())
}
```

## Creating Ruby Classes from Rust Structs

Magnus provides several ways to define Ruby classes that wrap Rust structures. The approach you choose depends on your
specific needs.

### Using the TypedData Trait (Full Control)

For full control over memory management and Ruby integration:

```rust
use magnus::{class, define_class, method, prelude::*, DataTypeFunctions, TypedData, Error, Ruby};

// Define a Rust struct
#[derive(Debug, TypedData)]
#[magnus(class = "MyExtension::Point", free_immediately, size)]
struct Point {
    x: f64,
    y: f64,
}

// Implement required trait
impl DataTypeFunctions for Point {}

// Implement methods
impl Point {
    fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }

    fn x(&self) -> f64 {
        self.x
    }

    fn y(&self) -> f64 {
        self.y
    }

    fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    fn to_s(&self) -> String {
        format!("Point({}, {})", self.x, self.y)
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("MyExtension")?;
    let class = module.define_class("Point", ruby.class_object())?;

    // Define the constructor
    class.define_singleton_method("new", function!(|x: f64, y: f64| {
        Point::new(x, y)
    }, 2))?;

    // Define instance methods
    class.define_method("x", method!(Point::x, 0))?;
    class.define_method("y", method!(Point::y, 0))?;
    class.define_method("distance", method!(Point::distance, 1))?;
    class.define_method("to_s", method!(Point::to_s, 0))?;

    Ok(())
}
```

### Using the Wrap Macro (Simplified Approach)

For a simpler approach with less boilerplate:

```rust
use magnus::{define_class, function, method, prelude::*, Error, Ruby};

// Define a Rust struct
struct Rectangle {
    width: f64,
    height: f64,
}

// Use the wrap macro to handle the Ruby class mapping
#[magnus::wrap(class = "MyExtension::Rectangle")]
impl Rectangle {
    // Constructor
    fn new(width: f64, height: f64) -> Self {
        Rectangle { width, height }
    }

    // Instance methods
    fn width(&self) -> f64 {
        self.width
    }

    fn height(&self) -> f64 {
        self.height
    }

    fn area(&self) -> f64 {
        self.width * self.height
    }

    fn perimeter(&self) -> f64 {
        2.0 * (self.width + self.height)
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("MyExtension")?;
    let class = module.define_class("Rectangle", ruby.class_object())?;

    // Register class methods and instance methods
    class.define_singleton_method("new", function!(Rectangle::new, 2))?;
    class.define_method("width", method!(Rectangle::width, 0))?;
    class.define_method("height", method!(Rectangle::height, 0))?;
    class.define_method("area", method!(Rectangle::area, 0))?;
    class.define_method("perimeter", method!(Rectangle::perimeter, 0))?;

    Ok(())
}
```

### Using RefCell for Mutable Rust Objects

For Ruby objects that need interior mutability:

```rust
use std::cell::RefCell;
use magnus::{define_class, function, method, prelude::*, Error, Ruby};

struct Counter {
    count: usize,
}

#[magnus::wrap(class = "MyExtension::Counter")]
struct MutCounter(RefCell<Counter>);

impl MutCounter {
    fn new(initial: usize) -> Self {
        MutCounter(RefCell::new(Counter { count: initial }))
    }

    fn count(&self) -> usize {
        self.0.borrow().count
    }

    fn increment(&self) -> usize {
        let mut counter = self.0.borrow_mut();
        counter.count += 1;
        counter.count
    }

    fn increment_by(&self, n: usize) -> usize {
        let mut counter = self.0.borrow_mut();
        counter.count += n;
        counter.count
    }

    // AVOID this pattern which can cause BorrowMutError
    fn bad_increment_method(&self) -> Result<usize, Error> {
        // Don't do this - it keeps the borrowing active while trying to borrow_mut
        if self.0.borrow().count > 10 {
            // This will panic with "already borrowed: BorrowMutError"
            self.0.borrow_mut().count += 100;
        } else {
            self.0.borrow_mut().count += 1;
        }

        Ok(self.0.borrow().count)
    }

    // CORRECT pattern - complete the first borrow before starting the second
    fn good_increment_method(&self) -> Result<usize, Error> {
        // Copy the value first
        let current_count = self.0.borrow().count;

        // Then the first borrow is dropped and we can borrow_mut safely
        if current_count > 10 {
            self.0.borrow_mut().count += 100;
        } else {
            self.0.borrow_mut().count += 1;
        }

        Ok(self.0.borrow().count)
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("MyExtension")?;
    let class = module.define_class("Counter", ruby.class_object())?;

    class.define_singleton_method("new", function!(MutCounter::new, 1))?;
    class.define_method("count", method!(MutCounter::count, 0))?;
    class.define_method("increment", method!(MutCounter::increment, 0))?;
    class.define_method("increment_by", method!(MutCounter::increment_by, 1))?;
    class.define_method("good_increment", method!(MutCounter::good_increment_method, 0))?;

    Ok(())
}
```

## Implementing Ruby Methods

Magnus provides flexible macros to help define methods with various signatures.

### Function vs Method Macros

Magnus provides two primary macros for defining callable Ruby code:

1. `function!` - For singleton/class methods and module functions
2. `method!` - For instance methods when you need access to the Rust object (`&self`)

Here's how to use each:

```rust
use magnus::{function, method, define_class, prelude::*, Error, Ruby};

struct Calculator {}

#[magnus::wrap(class = "Calculator")]
impl Calculator {
    // Constructor - a class method
    fn new() -> Self {
        Calculator {}
    }

    // Regular instance method that doesn't raise exceptions
    fn add(&self, a: i64, b: i64) -> i64 {
        a + b
    }

    // Method that needs the Ruby interpreter to raise an exception
    fn divide(ruby: &Ruby, _rb_self: &Self, a: i64, b: i64) -> Result<i64, Error> {
        if b == 0 {
            return Err(Error::new(
                ruby.exception_zero_div_error(),
                "Division by zero"
            ));
        }
        Ok(a / b)
    }

    // Class method that doesn't need a Calculator instance
    fn version() -> &'static str {
        "1.0.0"
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let class = ruby.define_class("Calculator", ruby.class_object())?;

    // Register the constructor with function!
    class.define_singleton_method("new", function!(Calculator::new, 0))?;

    // Register a class method with function!
    class.define_singleton_method("version", function!(Calculator::version, 0))?;

    // Register instance methods with method!
    class.define_method("add", method!(Calculator::add, 2))?;
    class.define_method("divide", method!(Calculator::divide, 2))?;

    Ok(())
}
```

### Method Signature Patterns

There are several common method signature patterns depending on what your method needs to do:

#### Basic Method (no Ruby access, no exceptions)

```rust
fn add(&self, a: i64, b: i64) -> i64 {
    a + b
}
```

#### Method that Raises Exceptions

```rust
fn divide(ruby: &Ruby, _rb_self: &Self, a: i64, b: i64) -> Result<i64, Error> {
    if b == 0 {
        return Err(Error::new(
            ruby.exception_zero_div_error(),
            "Division by zero"
        ));
    }
    Ok(a / b)
}
```

#### Method that Needs to Access Self by Value

```rust
// Usually for cloning or consuming self
fn clone_and_modify(rb_self: Value) -> Result<Value, Error> {
    let ruby = unsafe { Ruby::get_unchecked() };
    let obj = ruby.class_object::<Calculator>()?.new_instance(())?;
    // Modify obj...
    Ok(obj)
}
```

#### Method with Ruby Block

```rust
fn with_retries(ruby: &Ruby, _rb_self: &Self, max_retries: usize, block: Proc) -> Result<Value, Error> {
    let mut retries = 0;
    loop {
        match block.call(ruby, ()) {
            Ok(result) => return Ok(result),
            Err(e) if retries < max_retries => {
                retries += 1;
                // Maybe backoff or log error
            },
            Err(e) => return Err(e),
        }
    }
}
```

## Class Inheritance and Mixins

Ruby supports a rich object model with single inheritance and multiple module inclusion. Magnus allows you to replicate
this model in your Rust extension.

### Creating a Subclass

```rust
use magnus::{Module, class, define_class, method, prelude::*, Error, Ruby};

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    // Get the parent class (Ruby's built-in Array)
    let array_class = ruby.class_object::<RArray>()?;

    // Create a subclass
    let sorted_array = ruby.define_class("SortedArray", array_class)?;

    // Override the << (push) method to keep the array sorted
    sorted_array.define_method("<<", method!(|ruby, rb_self: Value, item: Value| {
        let array = RArray::from_value(rb_self)?;
        array.push(ruby, item)?;

        // Call sort! to keep the array sorted
        array.funcall(ruby, "sort!", ())?;

        Ok(rb_self) // Return self for method chaining
    }, 1))?;

    Ok(())
}
```

### Including Modules (Mixins)

```rust
use magnus::{Module, class, define_class, define_module, method, prelude::*, Error, Ruby};

fn make_comparable(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("MyComparable")?;

    // Define methods for the module
    module.define_method("<=>", method!(|_ruby, rb_self: Value, other: Value| {
        // Implementation of the spaceship operator for comparison
        let self_num: Result<i64, _> = rb_self.try_convert();
        let other_num: Result<i64, _> = other.try_convert();

        match (self_num, other_num) {
            (Ok(a), Ok(b)) => Ok(a.cmp(&b) as i8),
            _ => Ok(nil()),
        }
    }, 1))?;

    // Define methods that depend on <=>
    module.define_method("==", method!(|ruby, rb_self: Value, other: Value| {
        let result: i8 = rb_self.funcall(ruby, "<=>", (other,))?;
        Ok(result == 0)
    }, 1))?;

    module.define_method(">", method!(|ruby, rb_self: Value, other: Value| {
        let result: i8 = rb_self.funcall(ruby, "<=>", (other,))?;
        Ok(result > 0)
    }, 1))?;

    module.define_method("<", method!(|ruby, rb_self: Value, other: Value| {
        let result: i8 = rb_self.funcall(ruby, "<=>", (other,))?;
        Ok(result < 0)
    }, 1))?;

    Ok(())
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    // Create our module
    make_comparable(ruby)?;

    // Create a class
    let score = ruby.define_class("Score", ruby.class_object())?;

    // Define methods
    score.define_singleton_method("new", function!(|value: i64| {
        let obj = RObject::new(ruby.class_object::<Score>())?;
        obj.ivar_set(ruby, "@value", value)?;
        Ok(obj)
    }, 1))?;

    score.define_method("value", method!(|ruby, rb_self: Value| {
        rb_self.ivar_get::<_, i64>(ruby, "@value")
    }, 0))?;

    // Include our module
    let comparable = ruby.define_module("MyComparable")?;
    score.include_module(ruby, comparable)?;

    Ok(())
}
```

## Working with Singleton Methods

Singleton methods in Ruby are methods attached to individual objects, not to their class. The most common use is
defining class methods, but they can be applied to any object.

### Defining a Class with Both Instance and Singleton Methods

```rust
use magnus::{class, define_class, function, method, prelude::*, Error, Ruby, Value};

#[magnus::wrap(class = "Logger")]
struct Logger {
    level: String,
}

impl Logger {
    fn new(level: String) -> Self {
        Logger { level }
    }

    fn log(&self, message: String) -> String {
        format!("[{}] {}", self.level, message)
    }

    // Class methods (singleton methods)
    fn default_level() -> &'static str {
        "INFO"
    }

    fn create_default(ruby: &Ruby) -> Result<Value, Error> {
        let class = ruby.class_object::<Logger>()?;
        let default_level = Self::default_level();
        class.new_instance((default_level,))
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let class = ruby.define_class("Logger", ruby.class_object())?;

    // Instance methods
    class.define_singleton_method("new", function!(Logger::new, 1))?;
    class.define_method("log", method!(Logger::log, 1))?;

    // Class methods using function! macro
    class.define_singleton_method("default_level", function!(Logger::default_level, 0))?;
    class.define_singleton_method("create_default", function!(Logger::create_default, 0))?;

    Ok(())
}
```

### Attaching Methods to a Specific Object (True Singleton Methods)

```rust
use magnus::{module, function, prelude::*, Error, Ruby, Value};

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    // Create a single object
    let config = ruby.eval::<Value>("Object.new")?;

    // Define singleton methods directly on that object
    config.define_singleton_method(ruby, "get", function!(|| {
        "Configuration value"
    }, 0))?;

    config.define_singleton_method(ruby, "enabled?", function!(|| {
        true
    }, 0))?;

    // Make it globally accessible
    ruby.define_global_const("CONFIG", config)?;

    Ok(())
}
```

This creates an object that can be used in Ruby like:

```ruby
CONFIG.get          # => "Configuration value"
CONFIG.enabled?     # => true
CONFIG.class        # => Object
```

## Best Practices

1. **Use magnus macros for class definition**: The `wrap` and `TypedData` macros simplify class definition
   significantly.

2. **Consistent naming**: Keep Ruby and Rust naming conventions consistent within their domains (snake_case for Ruby
   methods, CamelCase for Ruby classes).

3. **Layer your API**: Consider providing both low-level and high-level APIs for complex functionality.

4. **Document method signatures**: When using methods that can raise exceptions, document which exceptions can be
   raised.

5. **RefCell borrowing pattern**: Always release a `borrow()` before calling `borrow_mut()` by copying any needed
   values.

6. **Method macro selection**: Use `function!` for singleton methods and `method!` for instance methods.

7. **Include the Ruby parameter**: Always include `ruby: &Ruby` in your method signature if your method might raise
   exceptions or interact with the Ruby runtime.

8. **Reuse existing Ruby patterns**: When designing your API, follow existing Ruby conventions that users will already
   understand.

9. **Cache Ruby classes and modules**: Use `Lazy` to cache frequently accessed classes and modules.

10. **Maintain object hierarchy**: Properly use Ruby's inheritance and module system to organize your code."
