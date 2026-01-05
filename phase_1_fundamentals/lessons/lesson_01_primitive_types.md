# Lesson 1: Primitive Data Types 🟢

**Duration:** 30-40 minutes
**Difficulty:** 🟢 Easy
**Prerequisites:** Phase 0 (basic Rust setup)

---

## Learning Objectives

By the end of this lesson, you will:
- Understand Rust's type system and why it matters
- Know all primitive data types and when to use each
- Understand the difference between stack and heap allocation conceptually
- Work with strings, integers, floats, and collections
- Use type inference effectively

---

## Concept: Static Typing vs Dynamic Typing

### What is Type Safety?

**The Problem in Dynamically-Typed Languages:**

In Python, you can write code that changes types at runtime:

```python
x = 5
print(x + 2)  # Works: 7

x = "hello"
print(x + 2)  # Runtime error! Can't add string and int
```

This code looks fine until you run it. The error happens at runtime, potentially in production.

**The Solution in Rust:**

Rust requires you to specify (or let the compiler infer) types at compile time:

```rust
let x: i32 = 5;
println!("{}", x + 2);  // Works: 7

let x = "hello";  // Compiler error: can't assign string to i32
println!("{}", x + 2);  // Never reaches here
```

The compiler catches the error **before** your program runs. This is type safety.

### Why This Matters

**Benefits of Static Typing:**
1. **Catch errors early** - at compile time, not runtime
2. **Documentation** - types tell you what a function accepts/returns
3. **Performance** - compiler knows exact memory requirements
4. **Prevents entire classes of bugs** - type mismatches become impossible

**The Tradeoff:**
You must be explicit about types (though inference helps a lot).

---

## Concept: Stack vs Heap Memory

### The Stack (for Primitive Types)

**What is it?**
- Fast memory region organized like a stack (last-in-first-out)
- Size known at compile time
- Automatically freed when variables go out of scope

**Characteristics:**
- Very fast access
- Limited size (typically a few MB)
- Must know size at compile time
- Automatically cleaned up

**Example:**
```rust
{
    let x: i32 = 42;  // On the stack
    let y: bool = true;  // On the stack
}  // x and y automatically freed here
```

Primitive types (`i32`, `bool`, `f64`, etc.) live on the stack.

### The Heap (for Complex Types)

**What is it?**
- Large memory region for variable-size data
- Slower to access than stack
- Manual management (in C++) or automatic (in Rust via ownership)

**Characteristics:**
- Flexible size (can grow as needed)
- Slower access than stack
- Size known at runtime
- Requires explicit freeing

**Example:**
```rust
{
    let s = String::from("hello");  // String data on heap
}  // Rust automatically frees heap memory here
```

Types like `String`, `Vec<T>`, and collections live on the heap.

### Why This Distinction Matters

When you pass a value to a function, Rust needs to know:
- **For primitives:** Just copy the bits (cheap, on stack)
- **For collections:** Move ownership or borrow the data (prevent double-free)

This is why borrowing and ownership exist—they manage heap data safely.

---

## Primitive Type: Integers

### Integer Types Overview

Integers come in two categories and multiple sizes:

| Type | Size | Range | Use Case |
|------|------|-------|----------|
| **i8** | 8 bits | -128 to 127 | Small negative numbers |
| **i16** | 16 bits | -32,768 to 32,767 | Moderate ranges |
| **i32** | 32 bits | -2.1B to 2.1B | **Default choice** |
| **i64** | 64 bits | -9.2Q to 9.2Q | Very large numbers, timestamps |
| **i128** | 128 bits | Huge range | Cryptography |
| **isize** | Platform | Varies | Pointer-sized, indexing |
| **u8** | 8 bits | 0 to 255 | Bytes, small counts |
| **u16** | 16 bits | 0 to 65,535 | Ports, colors, capacity |
| **u32** | 32 bits | 0 to 4.2B | Large non-negative counts |
| **u64** | 64 bits | 0 to 18.4Q | IDs, very large counts |
| **u128** | 128 bits | Huge range | Special cases |
| **usize** | Platform | Varies | Array lengths, indices |

### Understanding Signed vs Unsigned

**Signed (i)** - Can be negative or positive:
```rust
let temperature: i32 = -5;   // OK: negative
let balance: i32 = 100;      // OK: positive
```

**Unsigned (u)** - Only non-negative:
```rust
let age: u8 = 25;            // OK: positive
// let result: u8 = -5;      // ERROR: can't be negative
```

**When to Use:**
- **Signed (i32):** When values can be negative (temperatures, differences, balances)
- **Unsigned (u32/u64):** When values are always positive (counts, ages, IDs)
- **Default:** Use `i32` unless you have a specific reason (often it's fine)

### For the Binary Protocol Project

The protocol uses specific integer types:
- **u8** for version, message type, individual bytes
- **u16** for payload length (can be 0-65535 bytes)
- **u32/u64** for timestamps or large identifiers

### Integer Literals and Readability

```rust
// Basic integers
let x = 5;           // Type inferred as i32
let x = 5i32;        // Explicit type suffix
let x: i32 = 5;      // Explicit type annotation

// Underscores for readability (ignored by compiler)
let million = 1_000_000;  // Much easier to read!
let binary = 0b1010_0011;
let hex = 0xFF_AA_BB;

// Different bases
let decimal = 98_222;     // Normal
let hex = 0xff;           // Hexadecimal (0x prefix)
let octal = 0o77;         // Octal (0o prefix)
let binary = 0b1111_0000; // Binary (0b prefix)
let byte = b'A';          // Byte literal (b prefix) = 65
```

### Common Integer Operations

```rust
// Arithmetic
let sum = 5 + 6;           // 11
let diff = 6 - 5;          // 1
let product = 5 * 6;       // 30
let quotient = 20 / 5;     // 4 (integer division!)
let remainder = 20 % 3;    // 2

// Comparisons
let is_equal = 5 == 5;     // true
let is_greater = 5 > 3;    // true
let is_less = 5 < 10;      // true

// Bitwise (useful for binary protocols)
let a = 0b1010;            // 10 in binary
let b = 0b0011;            // 3 in binary
let and_result = a & b;    // 0b0010 = 2 (bitwise AND)
let or_result = a | b;     // 0b1011 = 11 (bitwise OR)
let xor_result = a ^ b;    // 0b1001 = 9 (bitwise XOR)
let not_result = !a;       // Bitwise NOT
let shifted = a >> 1;      // Right shift: 0b0101 = 5
let shifted = a << 1;      // Left shift: 0b10100 = 20
```

### Type Casting (Converting Between Integer Types)

```rust
let x: i32 = 5;
let y: u8 = x as u8;       // Convert i32 to u8
let z: i64 = x as i64;     // Convert i32 to i64

// Warning: casting can lose precision
let big: u32 = 300;
let small: u8 = big as u8; // Result: 44 (wraps around!)
```

---

## Primitive Type: Floating-Point Numbers

Floating-point numbers represent decimals:

| Type | Size | Precision |
|------|------|-----------|
| **f32** | 32 bits | ~6 decimal digits |
| **f64** | 64 bits | ~15 decimal digits (**default**) |

### When to Use Floats

```rust
let pi: f64 = 3.14159265359;     // Mathematical constant
let temperature: f32 = 98.6;     // Body temperature
let ratio: f64 = 16.5 / 2.5;    // Division result
```

### Important: Floating-Point Limitations

**Never compare floats with == for equality:**

```rust
let x = 0.1 + 0.2;
let y = 0.3;

// ❌ DON'T DO THIS:
// if x == y { ... }  // Might be false due to rounding!

// ✓ Instead, check if difference is small:
if (x - y).abs() < 0.0001 { /* close enough */ }
```

**Why?** Floating-point arithmetic has rounding errors. 0.1 + 0.2 ≠ 0.3 exactly in binary.

### Float Operations

```rust
let x = 5.0_f64;
let y = 2.0;

let sum = x + y;           // 7.0
let diff = x - y;          // 3.0
let product = x * y;       // 10.0
let quotient = x / y;      // 2.5
let remainder = x % y;     // 1.0

// Math methods
let absolute = -3.5_f64.abs();  // 3.5
let rounded = 3.7_f64.round();  // 4.0
let sqrt = 16.0_f64.sqrt();     // 4.0
let squared = 3.0_f64.powi(2);  // 9.0 (power of integer)
let power = 2.0_f64.powf(3.5);  // 2^3.5
```

---

## Primitive Type: Booleans

Booleans represent true/false values:

```rust
let is_ready: bool = true;
let is_empty: bool = false;

// Usually result from comparisons
let is_adult = age >= 18;    // boolean result
let is_valid = x > 0 && x < 100;  // compound boolean
```

### Boolean Logic

```rust
// AND: both must be true
let x = true && true;    // true
let x = true && false;   // false

// OR: at least one must be true
let x = true || false;   // true
let x = false || false;  // false

// NOT: negates the value
let x = !true;           // false
let x = !false;          // true

// Compound
let result = (x > 5) && (y < 10) || (z == 0);
```

### Using Booleans in Control Flow

```rust
if is_ready {
    println!("Go!");
} else {
    println!("Wait");
}
```

---

## Primitive Type: Characters

A `char` is a single Unicode character:

```rust
let letter: char = 'A';
let digit: char = '5';
let symbol: char = '!';
let emoji: char = '🦀';      // Rust mascot!

// Note the quotes:
let c = 'A';    // char (single quotes)
let s = "A";    // &str string (double quotes)
```

### Important: char vs &str

This is a common point of confusion:

```rust
// ❌ WRONG: single character with double quotes
let c = "A";    // Type is &str (string), not char!

// ✓ CORRECT: single character with single quotes
let c = 'A';    // Type is char

// ✓ CORRECT: multiple characters with double quotes
let s = "Hello"; // Type is &str (string slice)
```

### Unicode Support

```rust
let characters = vec!['a', 'b', 'c', '€', '北', '🦀'];
for ch in characters {
    println!("{}", ch);
}
```

### Character Escapes

```rust
let newline = '\n';           // Line break
let tab = '\t';               // Tab character
let backslash = '\\';         // Literal backslash
let quote = '\'';             // Single quote
let unicode = '\u{1F980}';    // Crab emoji via Unicode code point
```

---

## Type: Strings - &str vs String

This is one of the most important distinctions in Rust:

### &str (String Slice)

**What it is:** An immutable reference to string data

```rust
let greeting: &str = "Hello";  // String literal in binary
let borrowed: &str = &my_string;  // Borrow from String
```

**Characteristics:**
- Immutable (can't modify)
- Fixed size (known at compile time)
- Lives on stack (if literal) or points to heap data
- Very efficient
- The type of string literals

```rust
// String literals are &str type
let text = "Hello";  // Type: &str

// Borrowed from owned String
let owned = String::from("Hello");
let borrowed: &str = &owned;  // Type: &str
```

### String (Owned String)

**What it is:** A growable, mutable collection of characters on the heap

```rust
let mut message = String::from("Hello");
message.push_str(" World");
println!("{}", message);  // "Hello World"
```

**Characteristics:**
- Mutable (can modify)
- Growable (can add characters)
- Owns data on heap
- More memory overhead
- More flexible

### When to Use Each

**Use &str when:**
- You just need to read string data
- You're passing to functions that don't modify
- You want efficiency
- **Function parameters:** Accept `&str` by default

**Use String when:**
- You need to modify the string
- You own the data and control its lifetime
- You're building strings dynamically
- **Return types:** Return owned `String` when creating new data

### String Operations

```rust
let mut s = String::from("hello");

// Add to end
s.push_str(" world");     // Append &str
s.push('!');              // Append char

// Get length
println!("{}", s.len());  // 12 bytes

// Convert to &str
let slice: &str = &s;     // Borrow as &str

// Iterate
for ch in s.chars() {
    println!("{}", ch);
}

// Split
let parts: Vec<&str> = s.split(' ').collect();

// Replace
let replaced = s.replace("world", "Rust");
```

### String Indexing (Important!)

```rust
let s = "Hello";

// ❌ DON'T do this - strings don't support indexing by []
// let first = s[0];  // ERROR!

// ✓ Instead, use chars() or other methods
let first = s.chars().next();          // Option<char>
let as_bytes = s.as_bytes();           // &[u8]
let first_byte = as_bytes[0];          // Byte value

// ✓ Or slice a range
let substring = &s[0..2];              // "He"
```

Why no indexing? Because UTF-8 characters can be multiple bytes, and indexing would be ambiguous.

---

## Collection: Vec<T> - Vectors

A vector is a growable, contiguous array on the heap:

```rust
let mut numbers: Vec<i32> = vec![1, 2, 3];
numbers.push(4);
println!("{:?}", numbers);  // [1, 2, 3, 4]
```

### Creating Vectors

```rust
// Using the vec! macro with initial values
let v1 = vec![1, 2, 3];           // Type inferred: Vec<i32>
let v2 = vec![1; 5];              // [1, 1, 1, 1, 1] - 5 ones

// Using Vec::new() with explicit type
let v3: Vec<i32> = Vec::new();    // Empty vector
let v4 = Vec::new();               // ERROR: can't infer type

// Collecting from iterator
let numbers: Vec<i32> = (1..5).collect();  // [1, 2, 3, 4]
```

### Adding and Removing Elements

```rust
let mut v = vec![1, 2, 3];

v.push(4);              // Add to end: [1, 2, 3, 4]
let last = v.pop();     // Remove from end: Some(4)
v.insert(1, 99);        // Insert at index: [1, 99, 2, 3]
v.remove(1);            // Remove at index: [1, 2, 3]
v.clear();              // Remove all: []
```

### Accessing Elements

```rust
let v = vec![10, 20, 30];

let first = v[0];           // 10 (panics if out of bounds)
match v.get(1) {
    Some(val) => println!("{}", val),  // 20
    None => println!("No element"),
}

let last = v.last();        // Some(30)
let first = v.first();      // Some(10)
```

### Iterating Over Vectors

```rust
let v = vec![1, 2, 3];

// Immutable: read-only
for item in &v {
    println!("{}", item);
}

// Mutable: can modify each item
for item in &mut v {
    *item *= 2;  // Dereference and multiply by 2
}

// Ownership: consumes the vector
for item in v {
    println!("{}", item);
}
// v is no longer valid after this

// With index
for (i, item) in v.iter().enumerate() {
    println!("Item {}: {}", i, item);
}
```

### Vector Capacity

```rust
let mut v = Vec::new();

println!("len: {}, capacity: {}", v.len(), v.capacity());  // 0, 0

v.extend_from_slice(&[1, 2, 3]);
println!("len: {}, capacity: {}", v.len(), v.capacity());  // 3, 4

// Vectors grow strategically to avoid constant reallocation
// Capacity ≥ length always
```

---

## Type Inference

Rust's type inference allows you to omit types when the compiler can figure them out:

### When Inference Works

```rust
let x = 5;              // Inferred as i32 (default integer)
let y = 5.0;            // Inferred as f64 (default float)
let z = true;           // Inferred as bool
let s = "hello";        // Inferred as &str

let numbers = vec![1, 2, 3];  // Inferred as Vec<i32>
```

### When You Must Be Explicit

```rust
// ❌ Can't infer: no context
let numbers = vec![];   // ERROR: what type?

// ✓ Must be explicit
let numbers: Vec<i32> = vec![];

// ❌ Can't infer: ambiguous type
let x = 5;
let y: u8 = x;          // Type mismatch

// ✓ Explicit from the start
let x: u32 = 5;
let y: u8 = x as u8;    // Cast if needed
```

### Function Parameters Require Types

```rust
// ❌ Can't omit parameter types
fn add(x, y) {
    x + y
}

// ✓ Parameters must be explicit
fn add(x: i32, y: i32) -> i32 {
    x + y
}
```

---

## Code Examples

### Example 1: Working with Different Types

```rust
fn main() {
    // Integers
    let age: u8 = 25;
    let temperature: i32 = -5;
    let big_number: u64 = 1_000_000_000;

    println!("Age: {}", age);
    println!("Temperature: {}°C", temperature);
    println!("Large: {}", big_number);

    // Floats
    let pi: f64 = 3.14159;
    let weight: f32 = 72.5;
    println!("Pi: {}", pi);
    println!("Weight: {} kg", weight);

    // Booleans
    let is_ready = true;
    let is_error = age > 100;
    println!("Ready: {}, Error: {}", is_ready, is_error);

    // Characters and Strings
    let initial = 'R';
    let name: &str = "Rust";
    println!("{}. {}", initial, name);
}
```

Output:
```
Age: 25
Temperature: -5°C
Large: 1000000000
Pi: 3.14159
Weight: 72.5 kg
Ready: true, Error: false
R. Rust
```

### Example 2: Strings and Collections

```rust
fn main() {
    // Working with &str (immutable string slice)
    let greeting: &str = "Hello";
    println!("Greeting: {}", greeting);
    println!("Length: {}", greeting.len());

    // Building a String (owned, mutable)
    let mut message = String::from("Hello");
    message.push(' ');
    message.push_str("Rust");
    message.push('!');
    println!("Message: {}", message);

    // Vectors (growable arrays)
    let mut numbers = vec![1, 2, 3];
    numbers.push(4);
    numbers.push(5);

    for num in &numbers {
        println!("Number: {}", num);
    }

    println!("Total numbers: {}", numbers.len());
}
```

Output:
```
Greeting: Hello
Length: 5
Message: Hello Rust!
Number: 1
Number: 2
Number: 3
Number: 4
Number: 5
Total numbers: 5
```

### Example 3: Binary Protocol Data

```rust
fn main() {
    // Simulating protocol message header
    let version: u8 = 1;
    let message_type: u8 = 5;
    let payload_length: u16 = 100;

    println!("Protocol Message");
    println!("  Version: {}", version);
    println!("  Type: {}", message_type);
    println!("  Payload: {} bytes", payload_length);

    // Simulating payload as bytes
    let payload: Vec<u8> = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F];  // "Hello" in ASCII
    println!("  Payload hex: ");
    for byte in &payload {
        print!("{:02X} ", byte);
    }
    println!();

    // Convert to text
    let text = String::from_utf8_lossy(&payload);
    println!("  Payload text: {}", text);
}
```

Output:
```
Protocol Message
  Version: 1
  Type: 5
  Payload: 100 bytes
  Payload hex:
48 65 6C 6C 6F
  Payload text: Hello
```

---

## Common Mistakes for Python Developers

### Mistake 1: Forgetting Type Inference Has Limits

**❌ Wrong:**
```rust
let x = [];  // ERROR: can't infer type
```

**✓ Correct:**
```rust
let x: Vec<i32> = vec![];  // Be explicit with empty collections
```

### Mistake 2: Confusing Char and String

**❌ Wrong:**
```rust
let c = "A";  // This is &str, not char!
c.some_char_method();  // ERROR
```

**✓ Correct:**
```rust
let c = 'A';  // Single quotes for char
let s = "ABC";  // Double quotes for &str
```

### Mistake 3: Mixing Types in Collections

**❌ Wrong:**
```rust
let items = vec![1, "two", 3];  // Mixed types!
```

**✓ Correct:**
```rust
let items = vec![1, 2, 3];  // Consistent type
```

### Mistake 4: Not Understanding String Types

**❌ Wrong:**
```rust
fn process(s: String) { }
let text = "hello";
process(text);  // ERROR: passing &str to String parameter
```

**✓ Correct:**
```rust
fn process(s: &str) { }  // Accept &str, more flexible
let text = "hello";
process(text);  // OK: &str matches
```

### Mistake 5: Assuming Float Equality Works

**❌ Wrong:**
```rust
if 0.1 + 0.2 == 0.3 { }  // Unreliable!
```

**✓ Correct:**
```rust
if (0.1_f64 + 0.2 - 0.3).abs() < 0.0001 { }  // Check closeness
```

---

## Why This Matters

Understanding primitive types is foundational because:

1. **Safety:** Type checking prevents entire classes of bugs
2. **Performance:** The compiler optimizes based on exact types
3. **Clarity:** Types document what values a variable holds
4. **Foundation:** All complex types are built from primitives

For the binary protocol project specifically:
- **u8** represents individual bytes (version, type, checksum)
- **u16** represents two-byte big-endian values (length)
- **Vec<u8>** represents the payload (variable-length binary data)
- **Proper types prevent errors** like accidentally treating bytes as text

---

## Practice Exercises

### Exercise 1: Type Exploration

```rust
fn main() {
    // Try these and observe:
    let x = 42;        // What type is inferred?
    let y = 42u32;     // Explicitly u32
    let z = 42i8;      // Explicitly i8

    // Can you do this?
    // let result = x + z;  // Why or why not?
}
```

### Exercise 2: String Operations

```rust
fn main() {
    let mut text = String::from("Rust");

    // Add to the string
    text.push('!');

    // Create a reference
    let reference: &str = &text;

    // Try modifying through reference
    // reference.push('?');  // Does this work? Why?

    println!("{}", text);
}
```

### Exercise 3: Vectors

```rust
fn main() {
    let mut numbers = vec![1, 2, 3];

    // Add some numbers
    numbers.push(4);
    numbers.push(5);

    // Try to access
    println!("{}", numbers[0]);  // First
    println!("{:?}", numbers.get(10));  // Out of bounds

    // Iterate
    for n in &numbers {
        println!("{}", n * 2);
    }
}
```

---

## Key Takeaways

✓ Rust is **statically typed** - types are known at compile time
✓ **Primitives** (i32, f64, bool, char) live on the stack (fast)
✓ **Collections** (String, Vec) live on the heap (flexible size)
✓ **&str** is a borrowed string slice; **String** is owned and mutable
✓ **Use i32/f64 by default** unless you have specific size requirements
✓ **Type inference works** when context is clear, explicit types required otherwise
✓ **Understand stack vs heap** - it drives ownership and borrowing rules
✓ **String literals** ("text") are &str; building strings requires String type

---

## Next Steps

Now that you understand primitive types, you're ready for:
- **Lesson 2:** Creating Functions - how to work with these types in functions
- **Lesson 3:** Creating Structs - grouping types into custom structures

**Quiz Yourself:**
- What's the difference between i32 and u32?
- Why can't you use floating-point equality checks?
- What's the difference between "hello" and String::from("hello")?
- Why does Rust need both stack and heap?
