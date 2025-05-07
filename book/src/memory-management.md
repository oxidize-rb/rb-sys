{{#title Ruby on Rust: Memory Management & Safety}}

# Memory Management & Safety

One of the most important aspects of writing Ruby extensions is proper memory management. This chapter covers how Ruby's garbage collector interacts with Rust objects and how to ensure your extensions don't leak memory or cause segmentation faults.

<div class="warning">

Improper memory management is the leading cause of crashes and security vulnerabilities in native extensions. Rust's safety guarantees help prevent many common issues, but you still need to carefully manage the boundary between Ruby and Rust memory.

</div>

## Ruby's Garbage Collection System

Ruby uses a mark-and-sweep garbage collector to manage memory. Understanding how it works is essential for writing safe extensions:

1. **Marking Phase**: Ruby traverses all visible objects, marking them as "in use"
2. **Sweeping Phase**: Objects that weren't marked are considered garbage and freed

<div class="note">

When you create Rust objects that reference Ruby objects, you need to tell Ruby's GC about these references to prevent premature garbage collection. The `TypedData` trait and `mark` method provide the mechanism to do this.

</div>

## TypedData and DataTypeFunctions

Magnus provides a `TypedData` trait and `DataTypeFunctions` trait for managing Ruby objects that wrap Rust structs. This is the recommended way to handle complex objects in Rust.

### Basic TypedData Example

Here's how to define a simple Ruby object that wraps a Rust struct:

```rust
use magnus::{prelude::*, Error, Ruby, TypedData, DataTypeFunctions};

// Define your Rust struct
#[derive(TypedData)]
#[magnus(class = "MyExtension::Counter", free_immediately)]
struct Counter {
    count: i64,
}

// Implement required functions
impl DataTypeFunctions for Counter {}

// Implement methods for your struct
impl Counter {
    fn new(initial_value: i64) -> Self {
        Counter { count: initial_value }
    }
    
    fn increment(&mut self, amount: i64) -> i64 {
        self.count += amount;
        self.count
    }
    
    fn value(&self) -> i64 {
        self.count
    }
}

// Register with Ruby
fn init(ruby: &Ruby) -> Result<(), Error> {
    let class = ruby.define_class("Counter", ruby.class_object())?;
    
    class.define_singleton_method("new", function!(|initial: i64| {
        Ok(class.wrap(Counter::new(initial)))
    }, 1))?;
    
    class.define_method("increment", method!(Counter::increment, 1))?;
    class.define_method("value", method!(Counter::value, 0))?;
    
    Ok(())
}
```

### Implementing GC Marking

When your Rust struct holds references to Ruby objects, you need to implement the `mark` method to tell Ruby's GC about those references. Here's a simple example:

```rust
use magnus::{
    prelude::*, Error, Ruby, Value, TypedData, DataTypeFunctions,
    gc::Marker, typed_data::Obj
};

// A struct that holds references to Ruby objects
#[derive(TypedData)]
#[magnus(class = "MyExtension::Person", free_immediately, mark)]
struct Person {
    // Reference to a Ruby string (their name)
    name: Value,
    // Reference to Ruby array (their hobbies)
    hobbies: Value,
    // Optional reference to another Person (their friend)
    friend: Option<Obj<Person>>,
}

// Implement DataTypeFunctions with mark method
impl DataTypeFunctions for Person {
    // This is called during GC mark phase
    fn mark(&self, marker: &Marker) {
        // Mark the Ruby objects we reference
        marker.mark(self.name);
        marker.mark(self.hobbies);
        
        // If we have a friend, mark them too
        if let Some(ref friend) = self.friend {
            marker.mark(*friend);
        }
    }
}

impl Person {
    fn new(name: Value, hobbies: Value) -> Self {
        Self { 
            name, 
            hobbies, 
            friend: None,
        }
    }
    
    fn add_friend(&mut self, friend: Obj<Person>) {
        self.friend = Some(friend);
    }
    
    fn name(&self) -> Value {
        self.name
    }
    
    fn hobbies(&self) -> Value {
        self.hobbies
    }
    
    fn friend(&self) -> Option<Obj<Person>> {
        self.friend.clone()
    }
}

// Register with Ruby
fn init(ruby: &Ruby) -> Result<(), Error> {
    let class = ruby.define_class("Person", ruby.class_object())?;
    
    class.define_singleton_method("new", function!(|name: Value, hobbies: Value| {
        Ok(class.wrap(Person::new(name, hobbies)))
    }, 2))?;
    
    class.define_method("name", method!(Person::name, 0))?;
    class.define_method("hobbies", method!(Person::hobbies, 0))?;
    class.define_method("friend", method!(Person::friend, 0))?;
    class.define_method("add_friend", method!(Person::add_friend, 1))?;
    
    Ok(())
}
```

In this example:

1. The `Person` struct holds references to Ruby objects (`name` and `hobbies`) and another wrapped Rust object (`friend`)
2. We implement the `mark` method to tell Ruby's GC about all these references
3. During garbage collection, Ruby will know not to collect these objects as long as the `Person` is alive

## A Real-World Example: Trap from wasmtime-rb

Here's a slightly simplified version of a real-world example from the wasmtime-rb project:

```rust
use magnus::{
    prelude::*, method, Error, Ruby, TypedData, DataTypeFunctions,
    typed_data::Obj, Symbol
};

// A struct representing a WebAssembly trap (error)
#[derive(TypedData)]
#[magnus(class = "Wasmtime::Trap", size, free_immediately)]
pub struct Trap {
    trap: wasmtime::Trap,
    wasm_backtrace: Option<wasmtime::WasmBacktrace>,
}

// No references to Ruby objects, so mark is empty
impl DataTypeFunctions for Trap {}

impl Trap {
    pub fn new(trap: wasmtime::Trap, wasm_backtrace: Option<wasmtime::WasmBacktrace>) -> Self {
        Self {
            trap,
            wasm_backtrace,
        }
    }

    // Return a text description of the trap error
    pub fn message(&self) -> String {
        self.trap.to_string()
    }

    // Return the wasm backtrace if available
    pub fn wasm_backtrace_message(&self) -> Option<String> {
        self.wasm_backtrace.as_ref().map(|bt| format!("{bt}"))
    }

    // Return the trap code as a Ruby symbol
    pub fn code(&self) -> Result<Option<Symbol>, Error> {
        match self.trap {
            wasmtime::Trap::StackOverflow => Ok(Some(Symbol::new("STACK_OVERFLOW"))),
            wasmtime::Trap::MemoryOutOfBounds => Ok(Some(Symbol::new("MEMORY_OUT_OF_BOUNDS"))),
            // More cases...
            _ => Ok(Some(Symbol::new("UNKNOWN"))),
        }
    }

    // Custom inspect method
    pub fn inspect(rb_self: Obj<Self>) -> Result<String, Error> {
        Ok(format!(
            "#<Wasmtime::Trap:0x{:016x} @trap_code={}>",
            rb_self.as_raw(),
            rb_self.code()?.map_or("nil".to_string(), |s| s.to_string())
        ))
    }
}

// Register with Ruby
pub fn init(ruby: &Ruby) -> Result<(), Error> {
    let class = ruby.define_class("Trap", ruby.class_object())?;
    
    class.define_method("message", method!(Trap::message, 0))?;
    class.define_method("wasm_backtrace_message", method!(Trap::wasm_backtrace_message, 0))?;
    class.define_method("code", method!(Trap::code, 0))?;
    class.define_method("inspect", method!(Trap::inspect, 0))?;
    
    Ok(())
}
```

This example shows:

1. A Rust struct that wraps WebAssembly-specific types 
2. Methods that convert Rust values to Ruby-friendly types
3. A simple implementation of `DataTypeFunctions` (since there are no Ruby object references to mark)

## More Complex Example: Memory References

Let's look at a more complex scenario involving memory management:

```rust
use magnus::{
    prelude::*, gc::Marker, Error, Ruby, TypedData, DataTypeFunctions,
    typed_data::Obj, value::Opaque, RString
};

// Represents WebAssembly memory
#[derive(TypedData)]
#[magnus(class = "Wasmtime::Memory", free_immediately, mark)]
struct Memory {
    // Reference to a store (context) object
    store: StoreContext,
    // The actual WebAssembly memory
    memory: WasmMemory,
}

impl DataTypeFunctions for Memory {
    fn mark(&self, marker: &Marker) {
        // Mark the store so it stays alive
        self.store.mark(marker);
    }
}

// A guard that ensures memory access is safe
struct MemoryGuard {
    // Reference to Memory object
    memory: Opaque<Obj<Memory>>,
    // Size when created, to detect resizing
    original_size: u64,
}

impl MemoryGuard {
    fn new(memory: Obj<Memory>) -> Result<Self, Error> {
        let original_size = memory.size()?;
        
        Ok(Self {
            memory: memory.into(),
            original_size,
        })
    }
    
    fn get(&self) -> Result<&Memory, Error> {
        let ruby = Ruby::get().unwrap();
        let mem = ruby.get_inner_ref(&self.memory);
        
        // Check that memory size hasn't changed
        if mem.size()? != self.original_size {
            return Err(Error::new(
                magnus::exception::runtime_error(),
                "memory was resized, reference is no longer valid"
            ));
        }
        
        Ok(mem)
    }
    
    fn mark(&self, marker: &Marker) {
        marker.mark(self.memory);
    }
}

// A slice of WebAssembly memory
#[derive(TypedData)]
#[magnus(class = "Wasmtime::MemorySlice", free_immediately, mark)]
struct MemorySlice {
    guard: MemoryGuard,
    offset: usize,
    size: usize,
}

impl DataTypeFunctions for MemorySlice {
    fn mark(&self, marker: &Marker) {
        // Mark the memory guard, which marks the memory object
        self.guard.mark(marker);
    }
}

impl MemorySlice {
    fn new(memory: Obj<Memory>, offset: usize, size: usize) -> Result<Self, Error> {
        let guard = MemoryGuard::new(memory)?;
        
        // Validate the slice is in bounds
        let mem = guard.get()?;
        if offset + size > mem.data_size()? {
            return Err(Error::new(
                magnus::exception::range_error(),
                "memory slice out of bounds"
            ));
        }
        
        Ok(Self {
            guard,
            offset,
            size,
        })
    }
    
    // Read the slice as a Ruby string (efficiently, without copying)
    fn to_str(&self) -> Result<RString, Error> {
        let ruby = unsafe { Ruby::get_unchecked() };
        let mem = self.guard.get()?;
        let data = mem.data()?;
        
        // Extract the relevant slice
        let slice = &data[self.offset..self.offset + self.size];
        
        // Create a Ruby string directly from the slice (zero-copy)
        Ok(ruby.str_from_slice(slice))
    }
    
    // Read the slice as a UTF-8 string (with validation)
    fn to_utf8_str(&self) -> Result<RString, Error> {
        let ruby = unsafe { Ruby::get_unchecked() };
        let mem = self.guard.get()?;
        let data = mem.data()?;
        
        // Extract the relevant slice
        let slice = &data[self.offset..self.offset + self.size];
        
        // Validate UTF-8 and create a Ruby string
        match std::str::from_utf8(slice) {
            Ok(s) => Ok(RString::new(s)),
            Err(e) => Err(Error::new(
                magnus::exception::encoding_error(),
                format!("invalid UTF-8: {}", e)
            ))
        }
    }
}
```

This more advanced example demonstrates:

1. **Guarded Resource Access**: The `MemoryGuard` ensures memory operations are safe by checking for resizing
2. **Proper GC Integration**: Both structs implement marking to ensure referenced objects aren't collected
3. **Efficient String Creation**: Using `str_from_slice` to create strings directly from memory without extra copying
4. **Error Handling**: All operations that might fail return meaningful errors
5. **Resource Validation**: The code validates bounds before accessing memory

## Common Memory Management Pitfalls

<div class="warning">

These pitfalls can lead to crashes, memory leaks, or undefined behavior in your Ruby extensions. Understanding and avoiding them is crucial for writing reliable code.

</div>

### 1. Forgetting to Mark References

If your Rust struct holds Ruby objects but doesn't implement marking, those objects might be collected while still in use:

```rust,hidelines=#
# use magnus::{gc::Marker, DataTypeFunctions, Value};
# 
// BAD: No marking implementation
struct BadExample {
    ruby_object: Value,  // This reference won't be marked by GC
}

# impl DataTypeFunctions for BadExample {
#     // Missing mark implementation!
# }
# 
// GOOD: Proper marking
struct GoodExample {
    ruby_object: Value,
}

impl DataTypeFunctions for GoodExample {
    fn mark(&self, marker: &Marker) {
        marker.mark(self.ruby_object);
    }
}

# // A more complex example with multiple references
# struct ComplexExample {
#     ruby_strings: Vec<Value>,
#     ruby_hash: Value,
#     ruby_object: Value,
# }
# 
# impl DataTypeFunctions for ComplexExample {
#     fn mark(&self, marker: &Marker) {
#         // Mark each string in the vector
#         for string in &self.ruby_strings {
#             marker.mark(*string);
#         }
#         
#         // Mark the hash and object
#         marker.mark(self.ruby_hash);
#         marker.mark(self.ruby_object);
#     }
# }
```

<div class="tip">

Click the eye icon (<i class="fa fa-eye"></i>) to see an additional example of marking multiple references in a more complex struct.

</div>

### 2. Creating Cyclic References

Cyclic references (A references B, which references A) can lead to memory leaks. Consider using weak references or redesigning your object graph.

### 3. Inefficient String Creation

<div class="note">

String handling is often a performance bottleneck in Ruby extensions. Using the right APIs can significantly improve performance.

</div>

Creating strings inefficiently can significantly impact performance:

```rust,hidelines=#
# use magnus::{Error, RString, Ruby};
# 
// BAD: Creates unnecessary temporary Rust String
fn inefficient_string(data: &[u8]) -> Result<RString, Error> {
    let temp_string = String::from_utf8(data.to_vec())?; // Unnecessary allocation
    Ok(RString::new(&temp_string))  // Another copy
}

// GOOD: Direct creation from slice
fn efficient_string(ruby: &Ruby, data: &[u8]) -> RString {
    ruby.str_from_slice(data)  // No extra copies
}

# // ALSO GOOD: Creating from string slice when UTF-8 is confirmed
# fn from_str(ruby: &Ruby, s: &str) -> RString {
#     RString::new(s)
# }
# 
# // ALSO GOOD: Creating binary string with capacity then filling
# fn build_string(ruby: &Ruby, size: usize) -> RString {
#     let mut string = RString::with_capacity(size);
#     // Fill the string directly...
#     string
# }
```

<div class="tip">

Both memory usage and performance are significantly improved by avoiding unnecessary allocations and copies. The eye icon (<i class="fa fa-eye"></i>) reveals additional efficient string handling examples.

</div>

### 4. Not Handling Exceptions Properly

Ruby exceptions can disrupt the normal flow of your code. Ensure resources are cleaned up even when exceptions occur.

## RefCell and Interior Mutability

When creating Ruby objects with Rust, you'll often need to use interior mutability patterns. The most common approach is using `RefCell` to allow your Ruby objects to be mutated even when users hold immutable references to them.

### Understanding RefCell and Borrowing

Rust's `RefCell` allows mutable access to data through shared references, but enforces Rust's borrowing rules at runtime. This is perfect for Ruby extension objects, where Ruby owns the object and we interact with it via method calls.

A common pattern is to wrap your Rust struct in a `RefCell`:

```rust
use std::cell::RefCell;
use magnus::{prelude::*, Error, Ruby};

struct Counter {
    count: i64,
}

#[magnus::wrap(class = "MyExtension::Counter")]
struct MutCounter(RefCell<Counter>);

impl MutCounter {
    fn new(initial: i64) -> Self {
        Self(RefCell::new(Counter { count: initial }))
    }
    
    fn count(&self) -> i64 {
        self.0.borrow().count
    }
    
    fn increment(&self) -> i64 {
        let mut counter = self.0.borrow_mut();
        counter.count += 1;
        counter.count
    }
}
```

### The BorrowMutError Problem

A common mistake when using `RefCell` is trying to borrow mutably when you already have an active immutable borrow. This leads to a `BorrowMutError` panic:

```rust
// BAD - will panic with "already borrowed: BorrowMutError"
fn buggy_add(&self, val: i64) -> Result<i64, Error> {
    // First borrow is still active when we try to borrow_mut below
    if let Some(sum) = self.0.borrow().count.checked_add(val) {
        self.0.borrow_mut().count = sum; // ERROR - already borrowed above
        Ok(sum)
    } else {
        Err(Error::new(
            ruby.exception_range_error(),
            "result too large"
        ))
    }
}
```

The problem is that the `borrow()` in the `if` condition is still active when we try to use `borrow_mut()` in the body. Rust's borrow checker would catch this at compile time for normal references, but `RefCell` defers this check to runtime, resulting in a panic.

### The Solution: Complete Borrows Before Mutating

The solution is to complete all immutable borrows before starting mutable ones:

```rust
// GOOD - copy the value first to complete the borrow
fn safe_add(&self, val: i64) -> Result<i64, Error> {
    // Get the current count, completing this borrow
    let current_count = self.0.borrow().count;
    
    // Now we can safely borrow mutably
    if let Some(sum) = current_count.checked_add(val) {
        self.0.borrow_mut().count = sum; // Safe now
        Ok(sum)
    } else {
        Err(Error::new(
            ruby.exception_range_error(),
            "result too large"
        ))
    }
}
```

By copying `count` to a local variable, we complete the immutable borrow before starting the mutable one, avoiding the runtime panic.

### Complex Example with Multiple Operations

When working with more complex data structures:

```rust
struct Game {
    players: Vec<String>,
    current_player: usize,
    score: i64,
}

#[magnus::wrap(class = "MyGame")]
struct MutGame(RefCell<Game>);

impl MutGame {
    fn new() -> Self {
        Self(RefCell::new(Game {
            players: Vec::new(),
            current_player: 0,
            score: 0,
        }))
    }
    
    // INCORRECT: Multiple borrows that will cause issues
    fn buggy_next_player_scores(&self, points: i64) -> Result<String, Error> {
        let game = self.0.borrow();
        if game.players.is_empty() {
            return Err(Error::new(
                magnus::exception::runtime_error(),
                "No players in game"
            ));
        }
        
        // This would panic - we're still borrowing game
        let mut game_mut = self.0.borrow_mut();
        game_mut.score += points;
        let player = game_mut.current_player;
        game_mut.current_player = (player + 1) % game_mut.players.len();
        
        Ok(format!("{} scored {} points! New total: {}", 
            game_mut.players[player], points, game_mut.score))
    }
    
    // CORRECT: Copy all needed data before releasing the borrow
    fn safe_next_player_scores(&self, points: i64) -> Result<String, Error> {
        // Read all the data we need first
        let player_name: String;
        let new_player_index: usize;
        let new_score: i64;
        
        {
            // Create a block scope to ensure the borrow is dropped
            let game = self.0.borrow();
            if game.players.is_empty() {
                return Err(Error::new(
                    magnus::exception::runtime_error(),
                    "No players in game"
                ));
            }
            
            player_name = game.players[game.current_player].clone();
            new_player_index = (game.current_player + 1) % game.players.len();
            new_score = game.score + points;
        } // borrow is dropped here
        
        // Now we can borrow mutably
        let mut game = self.0.borrow_mut();
        game.score = new_score;
        game.current_player = new_player_index;
        
        Ok(format!("{} scored {} points! New total: {}", 
            player_name, points, new_score))
    }
}
```

### Using Temporary Variables Instead of Block Scopes

If you prefer, you can use temporary variables instead of block scopes:

```rust
fn add_player(&self, player: String) -> Result<usize, Error> {
    // Get the current number of players first
    let player_count = self.0.borrow().players.len();
    
    // Now we can mutate
    let mut game = self.0.borrow_mut();
    game.players.push(player);
    
    Ok(player_count + 1) // Return new count
}
```

### RefCell Best Practices

1. **Complete All Borrows**: Always complete immutable borrows before starting mutable borrows.

2. **Use Block Scopes or Variables**: Either use block scopes to limit borrow lifetimes or copy needed values to local variables.

3. **Minimize Borrow Scope**: Keep the scope of borrows as small as possible.

4. **Clone When Necessary**: If you need to keep references to data while mutating other parts, clone the data you need to keep.

5. **Consider Data Design**: Structure your data to minimize the need for complex borrowing patterns.

6. **Error When Conflicting**: If you can't resolve a borrowing conflict cleanly, make the operation an error rather than trying to force it.

## Best Practices

1. **Use TypedData and DataTypeFunctions**: They provide a safe framework for memory management
2. **Always Implement Mark Methods**: Mark all Ruby objects your struct references
3. **Validate Assumptions**: Check that resources are valid before using them
4. **Use Zero-Copy APIs**: Leverage APIs like `str_from_slice` to avoid unnecessary copying
5. **Use Guards for Changing Data**: Validate assumptions before accessing data that might change
6. **Test Thoroughly with GC Stress**: Run tests with `GC.stress = true` to expose memory issues
7. **Handle RefCell Borrowing Carefully**: Complete all immutable borrows before starting mutable ones to avoid runtime panics

By following these practices, you can write Ruby extensions in Rust that are both memory-safe and efficient.

## Next Steps

In the next chapter, we'll explore performance optimization techniques that leverage Rust's strengths while maintaining memory safety.