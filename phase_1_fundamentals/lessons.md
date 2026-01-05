# Phase 1: Rust Fundamentals - Detailed Lessons

---

## Lesson 1: Primitive Data Types 🟢

### Overview
Rust is a **statically-typed** language, meaning every variable has a type known at compile time. Unlike Python, where you can write `x = 5` and later `x = "hello"`, Rust requires you to specify types—either explicitly or by relying on type inference. This lesson covers the fundamental building blocks: the primitive types.

In Rust, primitives are built-in types that represent basic values. They're stored on the **stack** (fast memory), making them inherently efficient. Understanding primitive types is essential because every more complex type in Rust is built from them.

### Python vs Rust: The Type System

**Python:**
```python
# Python is dynamically typed - types determined at runtime
x = 42
print(type(x))  # <class 'int'>
x = "now a string"
print(type(x))  # <class 'str'>
```

**Rust:**
```rust
// Rust is statically typed - types determined at compile time
let x: i32 = 42;        // explicit type annotation
let x = 42;             // type inference - Rust figures out i32
// x = "hello";         // ERROR! x is i32, not string
```

This might seem restrictive, but it's Rust's superpower: the compiler catches type errors **before your program runs**.

### The Primitive Types

#### Integers: Signed and Unsigned
Integers come in two flavors and multiple sizes:

| Type | Bits | Range | Use Case |
|------|------|-------|----------|
| i8 | 8 | -128 to 127 | Small negative numbers |
| i16 | 16 | -32,768 to 32,767 | Moderate ranges |
| i32 | 32 | -2 billion to 2 billion | **Default for integers** |
| i64 | 64 | -9 quintillion to 9 quintillion | Large numbers, timestamps |
| isize | varies | Platform-dependent | Indexing, pointer sizes |
| u8 | 8 | 0 to 255 | Bytes, small non-negative |
| u16 | 16 | 0 to 65,535 | Network ports, colors |
| u32 | 32 | 0 to 4 billion | Large non-negative counts |
| u64 | 64 | 0 to 18 quintillion | Very large counts, IDs |
| usize | varies | Platform-dependent | Array lengths, indices |

**Why multiple sizes?** Different sizes use different memory and are faster/slower at different tasks. For the binary protocol project, we'll use `u8` for individual bytes and `u16` for message lengths.

#### Floating-Point Numbers
```rust
let pi: f32 = 3.14159;              // 32-bit, similar to Python float (but smaller)
let more_precise: f64 = 3.141592653589793;  // 64-bit, more precision (DEFAULT)
```

**Python Analogy:** Python's `float` is always 64-bit; Rust lets you choose for efficiency.

#### Booleans
```rust
let is_valid: bool = true;
let is_error: bool = false;

// Booleans are often the result of comparisons
let age = 25;
let is_adult = age >= 18;  // true
```

#### Characters
```rust
let letter: char = 'A';
let emoji: char = '🦀';         // Yes, Rust supports Unicode!
let line_break: char = '\n';    // Escape sequences work

// Note: 'x' is a char, "x" is a string
```

**Common Mistake:** Python developers often confuse `char` and `str`. A `char` is **one Unicode character**; a `str` is a sequence.

#### Strings: &str and String

This is crucial and often confusing for Python developers.

**&str (string slice):**
- Fixed-size view into string data
- Lives on the stack (for literals) or borrows from a String
- Immutable
- More efficient when you don't need to modify
```rust
let greeting: &str = "Hello";  // Stored in binary, immutable
let borrowed = &owned_string;  // Borrows from owned String
```

**String (owned string):**
- Dynamic, growable collection of characters
- Stored on the heap
- Mutable
- More flexible
```rust
let mut message = String::from("Hello");
message.push_str(" World");  // Can modify
println!("{}", message);     // Hello World
```

**Why Two Types?**
- `&str` is efficient when you just need to read
- `String` is necessary when you need to build, modify, or own the data
- This distinction enables Rust's memory safety guarantees

For the binary protocol project, payloads will often be `&[u8]` (byte slices) or `Vec<u8>` (owned byte vectors).

#### Type Inference and Annotations

Rust's compiler is smart about inferring types:

```rust
let x = 5;              // Compiler infers i32
let y = 5u8;            // Explicit annotation: u8
let z = 5_i64;          // Underscores for readability
let big = 1_000_000;    // Still i32, but easier to read

// Type inference has limits - sometimes you must be explicit
let numbers = vec![1, 2, 3];  // Vec<i32> inferred
let ambiguous = vec![];       // ERROR: can't infer type without context
let fixed: Vec<i32> = vec![];  // OK: we told Rust what type
```

#### Collections: Vec<T>

Vectors are dynamic arrays - they grow as needed:

```rust
let mut numbers: Vec<i32> = vec![1, 2, 3];
numbers.push(4);
println!("{:?}", numbers);  // [1, 2, 3, 4]

// Access by index
let first = numbers[0];  // 1

// Iterate
for num in &numbers {
    println!("{}", num);
}
```

For the binary protocol, we'll extensively use `Vec<u8>` to represent raw bytes.

### Code Examples

#### Example 1: Basic Type Usage
```rust
fn main() {
    // Integers
    let age: u8 = 25;              // Explicit type - we're storing an age
    let population = 1_000_000i32;  // Suffixed literal

    // Floats
    let temperature: f64 = 98.6;

    // Booleans
    let is_raining = true;

    // Characters and Strings
    let grade: char = 'A';
    let message: &str = "Hello, Rust!";

    // Type inference
    let inferred = 42;  // Rust knows this is i32

    println!("Age: {}", age);
    println!("Message: {}", message);
    println!("Grade: {}", grade);
}
```

#### Example 2: Working with Strings
```rust
fn analyze_string(text: &str) {
    println!("Text: {}", text);
    println!("Length: {}", text.len());

    // Convert to owned String
    let mut owned = String::from(text);
    owned.push('!');
    println!("Modified: {}", owned);
}

fn main() {
    let literal = "Rust";
    analyze_string(literal);  // Pass &str

    let dynamic = String::from("Programming");
    analyze_string(&dynamic);  // Borrow the String as &str
}
```

#### Example 3: Vectors and Collections
```rust
fn main() {
    // Create a vector
    let mut numbers = vec![1, 2, 3, 4, 5];

    // Add elements
    numbers.push(6);

    // Access by index
    println!("First: {}", numbers[0]);

    // Iterate
    for num in &numbers {
        println!("Number: {}", num);
    }

    // Check length
    println!("Count: {}", numbers.len());

    // Common operations
    let sum: i32 = numbers.iter().sum();
    println!("Sum: {}", sum);
}
```

### Common Mistakes for Python Developers

1. **Forgetting Type Inference Has Limits**
   ```rust
   // ❌ ERROR: Can't infer type
   let x = [];

   // ✓ OK: Be explicit
   let x: Vec<i32> = vec![];
   ```

2. **Confusing char and &str**
   ```rust
   // ❌ Wrong
   let c = "A";  // This is &str, not char

   // ✓ Correct
   let c = 'A';  // Single quotes for char
   ```

3. **Type Mismatch in Collections**
   ```rust
   // ❌ ERROR: Mixed types
   let items = vec![1, "two", 3];

   // ✓ Either consistent types or enums (covered later)
   let numbers = vec![1, 2, 3];
   ```

4. **Forgetting that &str and String are Different**
   ```rust
   // ❌ This might fail
   fn double(s: String) -> String { /* ... */ }
   let result = double("literal");  // Passing &str to String parameter

   // ✓ Accept both
   fn double(s: &str) -> String { /* ... */ }
   ```

### Why This Matters

Understanding primitive types is essential because:
- **Safety:** Type checking prevents entire classes of bugs
- **Performance:** Fixed-size types are stored efficiently on the stack
- **Clarity:** Types document what values a variable can hold
- **For the Project:** The binary protocol parser works with bytes (u8), message lengths (u16), and version numbers (u8)

### Key Takeaways

- ✓ Rust is statically-typed; every variable has a type known at compile time
- ✓ Use signed integers (iX) for values that can be negative; unsigned (uX) for non-negative
- ✓ Use `i32` and `f64` as defaults unless you have a specific reason otherwise
- ✓ `&str` is a string slice (immutable reference); `String` is owned and growable
- ✓ `Vec<T>` is a dynamic array; use it when you need a variable-length collection
- ✓ Type inference is convenient but sometimes requires explicit annotations

---

## Lesson 2: Creating Functions 🟢

### Overview
Functions are the fundamental unit of code organization in Rust. Unlike Python's `def`, Rust functions require:
1. **Explicit parameter types**
2. **Explicit return types** (or inference)
3. **Understanding of expressions vs statements**

Functions in Rust are **pure and composable** by default—they don't have hidden side effects, making code easier to reason about.

### Function Syntax

The basic structure:
```rust
fn function_name(param1: Type1, param2: Type2) -> ReturnType {
    // function body
    expression_or_return_value
}
```

**Python Analogy:**
```python
# Python
def greet(name):
    return f"Hello, {name}!"

# Rust
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

### Expressions vs Statements

This is the most important concept for functions. **Statements** end with semicolons; **expressions** don't.

```rust
fn example() {
    let x = 5 + 6;      // Statement: binding a variable (ends with ;)
    {
        let x = 3;
        x + 1           // Expression: evaluates to 4 (NO semicolon)
    }
    // = 4 - the block above is an expression with value 4
}
```

**In function returns:**
```rust
fn add(a: i32, b: i32) -> i32 {
    a + b  // No semicolon - this is the return value (expression)
}

fn add_explicit(a: i32, b: i32) -> i32 {
    return a + b;  // Explicit return (statement - has semicolon)
}

// Both work, but the first is more idiomatic
```

**Common Mistake:** Adding a semicolon accidentally:
```rust
fn broken() -> i32 {
    42;  // Semicolon makes this a statement, returns unit type ()
}
// ERROR: expected `i32`, found `()`

fn fixed() -> i32 {
    42   // No semicolon - returns 42
}
```

### Parameters and Type Annotations

Every parameter must have an explicit type:

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn greet(name: &str, age: u8) -> String {
    format!("{} is {} years old", name, age)
}

fn process(data: Vec<u8>) {
    // No return type means returns ()
}
```

### Return Types

```rust
// No return value (returns unit type ())
fn say_hello() {
    println!("Hello!");
}

// Returns String
fn make_message() -> String {
    String::from("Important message")
}

// Returns bool
fn is_even(n: i32) -> bool {
    n % 2 == 0
}

// Returns multiple values (as tuple - covered later)
fn swap(a: i32, b: i32) -> (i32, i32) {
    (b, a)
}
```

### Function Scope and Variable Shadowing

Variables are scoped to their block:

```rust
fn scope_example() {
    let x = 5;

    {
        let x = x + 1;  // Shadowing: creates new x in this scope
        println!("{}", x);  // 6
    }

    println!("{}", x);  // 5 (original x still exists)
}
```

**Shadowing allows redeclaring variables:** This is often used for transformations:

```rust
fn parse_age(input: &str) -> u8 {
    let input = input.trim();           // &str with whitespace removed
    let input = input.parse::<u8>().unwrap();  // u8 parsed from string
    input
}
```

### Code Examples

#### Example 1: Basic Functions with Different Return Types
```rust
// Function returning nothing
fn print_twice(message: &str) {
    println!("{}", message);
    println!("{}", message);
}

// Function returning a value
fn double(n: i32) -> i32 {
    n * 2
}

// Function with multiple parameters
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Function using expressions
fn absolute_value(n: i32) -> i32 {
    if n >= 0 { n } else { -n }
}

fn main() {
    print_twice("Hello");
    println!("Double of 5: {}", double(5));
    println!("Sum: {}", add(3, 7));
    println!("Absolute: {}", absolute_value(-42));
}
```

#### Example 2: Understanding Expression Returns
```rust
// Implicit return (expression)
fn calculate_discount(price: f64, percent: f64) -> f64 {
    price * (1.0 - percent / 100.0)
}

// Explicit return (statement)
fn calculate_discount_explicit(price: f64, percent: f64) -> f64 {
    return price * (1.0 - percent / 100.0);
}

// Complex expression return
fn process_value(x: i32) -> i32 {
    if x < 0 {
        -x  // Return negation
    } else if x == 0 {
        1   // Return 1
    } else {
        x * 2  // Return doubled
    }
}

fn main() {
    println!("Price after 20% discount: {}", calculate_discount(100.0, 20.0));
    println!("Processed 5: {}", process_value(5));
    println!("Processed -3: {}", process_value(-3));
}
```

#### Example 3: Working with String Types
```rust
// Function that takes &str (more flexible)
fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

// Function that builds and returns a String
fn repeat_string(text: &str, times: usize) -> String {
    let mut result = String::new();
    for _ in 0..times {
        result.push_str(text);
        result.push(' ');
    }
    result
}

// Function combining both
fn summarize(text: &str) -> String {
    let word_count = count_words(text);
    format!("Text has {} words", word_count)
}

fn main() {
    let text = "Hello world";
    println!("Words: {}", count_words(text));
    println!("Repeated: {}", repeat_string("Hi", 3));
    println!("Summary: {}", summarize(text));
}
```

### Common Mistakes for Python Developers

1. **Forgetting Parameter Types**
   ```rust
   // ❌ ERROR: Can't omit types
   fn add(a, b) {
       a + b
   }

   // ✓ Correct
   fn add(a: i32, b: i32) -> i32 {
       a + b
   }
   ```

2. **Semicolon in Return Expression**
   ```rust
   // ❌ Wrong: semicolon turns expression into statement
   fn get_number() -> i32 {
       42;
   }

   // ✓ Correct
   fn get_number() -> i32 {
       42
   }
   ```

3. **Confusing Return Types**
   ```rust
   // ❌ Might not return what you expect
   fn maybe_double(n: i32) -> i32 {
       if n > 10 {
           n * 2
       }
       // Implicitly returns () here when condition is false!
   }

   // ✓ Be explicit
   fn maybe_double(n: i32) -> i32 {
       if n > 10 { n * 2 } else { n }
   }
   ```

4. **Parameter Order Matters**
   ```rust
   fn subtract(a: i32, b: i32) -> i32 {
       a - b
   }

   // Rust doesn't have keyword arguments by default
   // Position matters: subtract(10, 3) returns 7, not 3
   ```

### Why This Matters

Functions are the building blocks of all Rust code. Understanding:
- **Type requirements** ensures your code compiles
- **Expressions vs statements** is critical for returns
- **Parameter types** enables the compiler to catch errors
- **Scope rules** prevents variable confusion

For the binary protocol project, we'll write functions to parse bytes and validate checksums.

### Key Takeaways

- ✓ Function parameters require explicit types
- ✓ Return types must be explicit (unless returning ())
- ✓ Omit the semicolon on the last expression to return it
- ✓ Variable shadowing is allowed and often useful
- ✓ Functions in Rust are expressions-first; understand the difference from statements
- ✓ Use `&str` parameters when you don't need ownership; own `String` when you do

---

## Lesson 3: Creating Structs 🟢

### Overview
A **struct** (structure) is a custom data type that groups related data together. It's similar to a Python dataclass or namedtuple, but with more control and safety. Structs are essential for organizing data in meaningful ways—they're how you represent domain concepts in your code.

For the binary protocol parser, structs will represent protocol messages, headers, and payloads.

### Defining Structs

The basic syntax:
```rust
struct StructName {
    field1: Type1,
    field2: Type2,
}
```

**Example for the binary protocol:**
```rust
struct ProtocolMessage {
    version: u8,
    message_type: u8,
    length: u16,
    payload: Vec<u8>,
    checksum: u8,
}
```

### Creating Instances

```rust
let msg = ProtocolMessage {
    version: 1,
    message_type: 5,
    length: 10,
    payload: vec![72, 101, 108, 108, 111],  // "Hello"
    checksum: 42,
};

// Access fields with dot notation
println!("Version: {}", msg.version);
println!("Type: {}", msg.message_type);
```

### Mutable Structs

By default, struct fields are immutable:
```rust
let mut msg = ProtocolMessage { /* ... */ };
msg.version = 2;  // OK: msg is mutable
```

**Important:** Mutability is all-or-nothing. If you need a mutable struct, the entire struct is mutable.

```rust
// ❌ Can't selectively make one field mutable
let msg = ProtocolMessage { /* ... */ };
// msg.version = 2;  // ERROR: msg is immutable
```

### Shorthand Syntax

When variable names match field names:

```rust
fn create_message(version: u8, message_type: u8) -> ProtocolMessage {
    ProtocolMessage {
        version,              // Same as version: version
        message_type,         // Same as message_type: message_type
        length: 0,
        payload: Vec::new(),
        checksum: 0,
    }
}
```

### Struct Update Syntax

Create a new struct based on an existing one:

```rust
let msg1 = ProtocolMessage { /* ... */ };

let msg2 = ProtocolMessage {
    version: 2,
    ..msg1  // Copy all other fields from msg1
};
```

### Tuple Structs

Structs without named fields:

```rust
struct Color(u8, u8, u8);  // RGB
struct Point(f64, f64);

let red = Color(255, 0, 0);
let origin = Point(0.0, 0.0);

println!("Red component: {}", red.0);  // Access by index
println!("X coordinate: {}", origin.0);
```

Useful for lightweight wrappers or when field meaning is obvious.

### Unit Structs

Structs with no fields:

```rust
struct Marker;  // Useful for type safety without data
```

These might seem useless, but they're powerful for things like custom errors or type-system tricks (covered later).

### Visibility of Struct Fields

By default, struct fields are private (only visible within the module):

```rust
pub struct User {
    pub username: String,       // Public: anyone can read/write
    password: String,           // Private: only within module
}
```

For the binary protocol, most message fields should be public for test access.

### Destructuring Structs

Extract multiple fields at once:

```rust
let ProtocolMessage { version, message_type, .. } = msg;
println!("Version: {}, Type: {}", version, message_type);
```

### Code Examples

#### Example 1: Basic Struct Definition and Usage
```rust
struct Book {
    title: String,
    author: String,
    pages: u16,
    published: u16,
}

fn main() {
    let rust_book = Book {
        title: String::from("The Rust Programming Language"),
        author: String::from("Steve Klabnik"),
        pages: 651,
        published: 2023,
    };

    println!("Title: {}", rust_book.title);
    println!("By: {}", rust_book.author);
    println!("Pages: {}", rust_book.pages);
}
```

#### Example 2: Mutable Struct and Methods
```rust
struct Counter {
    count: i32,
}

fn main() {
    let mut counter = Counter { count: 0 };

    counter.count += 1;
    counter.count += 1;

    println!("Count: {}", counter.count);  // 2
}
```

#### Example 3: Struct Update Syntax
```rust
struct Protocol {
    version: u8,
    message_type: u8,
    flags: u8,
}

fn main() {
    let msg1 = Protocol {
        version: 1,
        message_type: 5,
        flags: 0,
    };

    // Create new struct with one field changed
    let msg2 = Protocol {
        message_type: 10,
        ..msg1
    };

    println!("msg1 type: {}, msg2 type: {}", msg1.message_type, msg2.message_type);
}
```

### Common Mistakes for Python Developers

1. **Thinking Structs Are Always Mutable**
   ```rust
   // ❌ Wrong: forgot mut
   let person = Person { name: "Alice".into(), age: 30 };
   // person.age = 31;  // ERROR: person is immutable

   // ✓ Correct
   let mut person = Person { name: "Alice".into(), age: 30 };
   person.age = 31;  // OK
   ```

2. **Confusing Struct Fields with Class Attributes**
   ```rust
   // In Python, you might expect:
   // person.set_age(31)

   // In Rust, fields are directly assignable (if mutable):
   person.age = 31;  // Direct access
   ```

3. **Forgetting to Specify Types for String**
   ```rust
   // ❌ Wrong: String might be unclear
   struct Person {
       name: String,
       age: u8,
   }

   let person = Person {
       name: "Alice",  // ERROR: &str, not String
       age: 30,
   };

   // ✓ Correct
   let person = Person {
       name: String::from("Alice"),  // Convert &str to String
       age: 30,
   };
   ```

4. **Trying to Have Selective Mutability**
   ```rust
   // ❌ Can't do this in Rust
   let person = Person { /* ... */ };
   // person.age = 31;  // ERROR: person is immutable

   // You need either all mutable or all immutable
   let mut person = Person { /* ... */ };
   person.age = 31;  // OK
   ```

### Why This Matters

Structs enable:
- **Organization:** Group related data meaningfully
- **Type Safety:** The compiler ensures you have all required fields
- **Documentation:** Struct definitions document what data lives together
- **Methods:** Next lesson shows how to attach functions to structs

For the binary protocol project, we'll define structs for:
- Protocol messages (version, type, length, payload, checksum)
- Parsed headers
- Error states

### Key Takeaways

- ✓ Structs group related data with named fields
- ✓ Structs are immutable by default; use `let mut` for mutable structs
- ✓ Access fields with dot notation: `struct_instance.field`
- ✓ Use shorthand syntax when variable names match field names
- ✓ Tuple structs are useful for lightweight wrappers
- ✓ The `..` syntax copies remaining fields from another struct

---

## Lesson 4: Impl Blocks and Methods 🟡

### Overview
An **impl block** is where you define methods for your structs (and other types). A method is a function associated with a type. In Python, you'd put methods inside a class; in Rust, you put them in an `impl` block.

This lesson completes the struct story and enables building clean, object-like APIs.

### Method Basics

The fundamental pattern:
```rust
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }

    fn perimeter(&self) -> u32 {
        2 * (self.width + self.height)
    }
}

fn main() {
    let rect = Rectangle { width: 30, height: 50 };
    println!("Area: {}", rect.area());      // Method call
    println!("Perimeter: {}", rect.perimeter());
}
```

### Three Types of Methods

#### 1. Immutable Borrow: `&self`
Read the struct without modifying:

```rust
impl Rectangle {
    fn describe(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }
}
```

Use `&self` when you only need to read fields.

#### 2. Mutable Borrow: `&mut self`
Modify the struct:

```rust
impl Rectangle {
    fn resize(&mut self, new_width: u32, new_height: u32) {
        self.width = new_width;
        self.height = new_height;
    }
}

let mut rect = Rectangle { width: 30, height: 50 };
rect.resize(40, 60);  // Must be mutable
```

Use `&mut self` when you need to modify fields.

#### 3. Take Ownership: `self`
Consume the struct (can't use it afterward):

```rust
impl Rectangle {
    fn into_tuple(self) -> (u32, u32) {
        (self.width, self.height)  // Consumes self
    }
}

let rect = Rectangle { width: 30, height: 50 };
let dims = rect.into_tuple();
// rect is no longer valid after this!
```

Use `self` when you want to consume the struct or transform it completely.

### Associated Functions

Functions in an `impl` block that don't take `self`:

```rust
impl Rectangle {
    // Associated function (no self parameter)
    fn square(side: u32) -> Rectangle {
        Rectangle { width: side, height: side }
    }
}

// Call with :: syntax
let sq = Rectangle::square(50);
```

Associated functions are like static methods in other languages. Use them for constructors or factory methods.

### Method Chaining

Methods that return `self` enable chaining:

```rust
impl Rectangle {
    fn set_width(mut self, width: u32) -> Rectangle {
        self.width = width;
        self
    }

    fn set_height(mut self, height: u32) -> Rectangle {
        self.height = height;
        self
    }
}

let rect = Rectangle::square(0)
    .set_width(30)
    .set_height(50);
```

This pattern is common in Rust for builders.

### Multiple Impl Blocks

You can have multiple `impl` blocks for the same struct:

```rust
impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }
}

impl Rectangle {
    fn perimeter(&self) -> u32 {
        2 * (self.width + self.height)
    }
}
```

This enables organizing methods logically or implementing different traits (covered later).

### Code Examples

#### Example 1: Basic Methods
```rust
struct BankAccount {
    holder: String,
    balance: f64,
}

impl BankAccount {
    // Constructor-like associated function
    fn new(holder: String) -> BankAccount {
        BankAccount { holder, balance: 0.0 }
    }

    // Read-only method
    fn get_balance(&self) -> f64 {
        self.balance
    }

    // Mutable method
    fn deposit(&mut self, amount: f64) {
        self.balance += amount;
    }

    // Mutable method
    fn withdraw(&mut self, amount: f64) -> bool {
        if amount <= self.balance {
            self.balance -= amount;
            true
        } else {
            false
        }
    }
}

fn main() {
    let mut account = BankAccount::new("Alice".into());
    account.deposit(100.0);

    if account.withdraw(30.0) {
        println!("Withdrew $30");
    }

    println!("Balance: ${}", account.get_balance());
}
```

#### Example 2: Methods with Self
```rust
#[derive(Debug)]
struct Message {
    text: String,
}

impl Message {
    fn new(text: String) -> Self {
        Message { text }
    }

    // Method that modifies self and returns it
    fn uppercase(mut self) -> Self {
        self.text = self.text.to_uppercase();
        self
    }

    // Method that consumes self
    fn into_string(self) -> String {
        self.text
    }
}

fn main() {
    let msg = Message::new("hello".into());
    let upper = msg.uppercase();  // Returns modified Message

    let text = upper.into_string();  // Consumes Message
    println!("{}", text);  // HELLO
    // upper is no longer valid
}
```

#### Example 3: Builder Pattern
```rust
struct RequestBuilder {
    url: String,
    method: String,
    headers: Vec<String>,
}

impl RequestBuilder {
    fn new(url: String) -> Self {
        RequestBuilder {
            url,
            method: "GET".into(),
            headers: Vec::new(),
        }
    }

    fn method(mut self, method: &str) -> Self {
        self.method = method.into();
        self
    }

    fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.push(format!("{}: {}", key, value));
        self
    }

    fn build(self) -> String {
        format!("{} {}", self.method, self.url)
    }
}

fn main() {
    let request = RequestBuilder::new("https://api.example.com".into())
        .method("POST")
        .header("Content-Type", "application/json")
        .build();

    println!("{}", request);
}
```

### Common Mistakes for Python Developers

1. **Forgetting `self` Parameters Are References**
   ```rust
   // ❌ Wrong: methods automatically dereference
   impl Rectangle {
       fn area(self) -> u32 {
           // After this method, rect is consumed!
           self.width * self.height
       }
   }

   // ✓ Better: use &self
   impl Rectangle {
       fn area(&self) -> u32 {
           self.width * self.height
       }
   }
   ```

2. **Confusing Associated Functions with Methods**
   ```rust
   // Associated function (called with ::)
   Rectangle::new()    // :: syntax

   // Method (called with .)
   rect.area()         // . syntax
   ```

3. **Trying to Mutate Without `&mut self`**
   ```rust
   // ❌ Wrong: can't modify with &self
   impl Rectangle {
       fn resize(&self, width: u32) {
           self.width = width;  // ERROR
       }
   }

   // ✓ Correct: use &mut self
   impl Rectangle {
       fn resize(&mut self, width: u32) {
           self.width = width;
       }
   }
   ```

4. **Not Realizing `self` Consumes the Value**
   ```rust
   let rect = Rectangle { width: 30, height: 50 };
   let dims = rect.into_tuple();  // Takes ownership
   // println!("{:?}", rect);  // ERROR: rect is no longer valid
   ```

### Why This Matters

Methods enable:
- **Encapsulation:** Hide implementation details
- **Type Safety:** The compiler ensures correct usage
- **Ergonomics:** `rect.area()` is cleaner than `calculate_area(&rect)`
- **Ownership:** Methods make ownership semantics explicit (which version of self?)

For the binary protocol project, we'll implement:
- Parsers: methods to construct messages from bytes
- Validators: methods to check validity
- Serializers: methods to convert back to bytes

### Key Takeaways

- ✓ Methods are functions in `impl` blocks, associated with a type
- ✓ `&self` for read-only access; `&mut self` for modifications; `self` for consuming
- ✓ Use `::` for associated functions; `.` for methods
- ✓ Multiple `impl` blocks for the same struct are allowed
- ✓ Methods enable cleaner, more ergonomic APIs
- ✓ The builder pattern enables fluent configuration

---

## Lesson 5: Ownership Fundamentals 🟡

### Overview
**Ownership** is Rust's killer feature. It enables memory safety without a garbage collector. Rather than trying to be clever, Rust enforces simple rules at compile time that prevent memory errors:

- Double-frees
- Use-after-free
- Data races
- Buffer overflows

This lesson covers the conceptual foundation. Future lessons cover borrowing and lifetimes.

### The Three Ownership Rules

Memorize these:

1. **Each value has exactly one owner**
2. **When the owner is dropped, the value is freed**
3. **You can transfer ownership (move) to another owner**

That's it. Everything else follows from these three rules.

### Stack vs Heap (Conceptual Understanding)

**Stack:**
- Fast access, but limited size
- Fixed-size data (primitives like i32, u8)
- Organized as a stack; last-in-first-out
- Automatically freed when variables go out of scope

**Heap:**
- Slower access, but flexible size
- Variable-size data (String, Vec<T>)
- Requires explicit management (in languages like C++)
- Rust manages this automatically with ownership

```rust
let x = 5;              // i32 lives on stack
let s = String::from("hello");  // String data lives on heap
// x automatically freed here
// s automatically freed here
```

### Move Semantics

When you assign or pass an owned value, **ownership moves**:

```rust
let s1 = String::from("hello");
let s2 = s1;  // Ownership moves from s1 to s2

// println!("{}", s1);  // ERROR: s1 is no longer valid!
println!("{}", s2);   // OK: s2 owns the string
```

**Why?** If both s1 and s2 could access the same string, when scope ends, both would try to free it (double-free).

```rust
fn takes_ownership(s: String) {
    println!("{}", s);
    // s is dropped here, string is freed
}

let s1 = String::from("hello");
takes_ownership(s1);
// println!("{}", s1);  // ERROR: s1 was moved into function
```

This might seem restrictive, but it's **intentional**. It forces you to think about who owns data.

### Copy Types

Some types don't move; they copy:

```rust
let x = 5;
let y = x;  // Copies, doesn't move
println!("{}", x);  // OK: x is still valid
```

**Copy types:**
- Primitives: i32, u8, bool, char, f64, etc.
- Small fixed-size types

**Non-copy types:**
- String (variable size)
- Vec<T> (variable size)
- Most custom types

### Clone: Explicit Deep Copy

When you need a copy but the type doesn't implement Copy, use `.clone()`:

```rust
let s1 = String::from("hello");
let s2 = s1.clone();  // Explicit copy
println!("{}", s1);  // OK: s1 still valid
println!("{}", s2);  // OK: s2 is independent copy
```

Clone is explicit—it makes the expensive operation obvious in code.

### Ownership Transfer vs Borrowing

Ownership transfer (move):
```rust
let s1 = String::from("hello");
let s2 = s1;  // s1 moves to s2, s1 no longer valid
```

Borrowing (next lesson covers this thoroughly):
```rust
let s1 = String::from("hello");
let s2 = &s1;  // Borrow s1, s1 still owns
println!("{}", s1);  // OK: s1 still valid
```

### Why This Matters

Ownership prevents:
- **Memory leaks:** Every value is automatically freed
- **Double-frees:** Only one owner at a time
- **Use-after-free:** Compiler prevents accessing freed memory
- **Data races:** Ownership rules prevent unsynchronized sharing

For the binary protocol project, ownership will matter when:
- Parsing takes bytes from a buffer
- Returning parsed messages
- Passing data to validation functions

### Code Examples

#### Example 1: Basic Move Semantics
```rust
fn main() {
    let s1 = String::from("Rust");
    let s2 = s1;  // Ownership moves

    // println!("{}", s1);  // ERROR: s1 has lost ownership
    println!("{}", s2);   // OK: s2 owns the string
}
```

#### Example 2: Move Through Function Calls
```rust
fn consume_string(s: String) {
    println!("Received: {}", s);
    // String is dropped here
}

fn main() {
    let message = String::from("Hello, Rust!");
    consume_string(message);  // Ownership moves into function

    // println!("{}", message);  // ERROR: message was moved
}
```

#### Example 3: Clone for Copying
```rust
fn main() {
    let original = vec![1, 2, 3, 4, 5];

    // Method 1: Move (no longer have original)
    let moved = original;
    // println!("{:?}", original);  // ERROR: moved

    // Method 2: Clone (keep both)
    let cloned = original.clone();
    println!("{:?}", original);  // OK: still valid
    println!("{:?}", cloned);    // OK: independent copy
}
```

#### Example 4: Copy vs Clone
```rust
fn main() {
    // Primitives implement Copy
    let num1 = 42;
    let num2 = num1;  // Copied automatically
    println!("{} {}", num1, num2);  // Both valid

    // Strings require clone
    let str1 = String::from("hello");
    let str2 = str1.clone();  // Explicit copy
    println!("{} {}", str1, str2);  // Both valid
}
```

### Common Mistakes for Python Developers

1. **Expecting Python-Style References**
   ```rust
   // In Python, both reference the same object:
   # list1 = [1, 2, 3]
   # list2 = list1
   # Both refer to same list

   // In Rust, ownership moves:
   let vec1 = vec![1, 2, 3];
   let vec2 = vec1;  // Ownership moved
   // vec1 is no longer valid!
   ```

2. **Not Realizing Functions Take Ownership**
   ```rust
   // ❌ Common mistake
   fn process(s: String) { /* ... */ }

   let msg = String::from("Important");
   process(msg);
   // println!("{}", msg);  // ERROR: moved

   // ✓ Better: borrow instead
   fn process(s: &str) { /* ... */ }
   let msg = String::from("Important");
   process(&msg);
   println!("{}", msg);  // OK
   ```

3. **Clone Performance Assumptions**
   ```rust
   // Cloning a large vector is expensive!
   let large_vec = vec![0; 1_000_000];
   let copy = large_vec.clone();  // Allocates 2MB!

   // Usually, you should borrow instead
   fn process(v: &Vec<i32>) { /* ... */ }
   process(&large_vec);  // No allocation
   ```

### Key Takeaways

- ✓ Each value has exactly one owner
- ✓ Ownership can be transferred (moved) but not shared
- ✓ Primitive types implement Copy and don't move
- ✓ Use `.clone()` for explicit copies of non-Copy types
- ✓ When a variable goes out of scope, the value is freed
- ✓ Next lesson: borrowing allows sharing without moving

---

## Lesson 6: References and Borrowing 🟡

### Overview
If ownership is Rust's foundation, **borrowing** is its superpower. Borrowing lets you use a value without owning it. Instead of moving ownership (which prevents the original owner from using the value), you can temporarily lend it.

There are two types of borrows:
- **Immutable borrows** (`&T`): Multiple borrows, read-only
- **Mutable borrows** (`&mut T`): Single borrow, can modify

### Immutable References

Borrow a value without owning it:

```rust
let s = String::from("hello");
let ref1 = &s;  // Borrow (don't own)
let ref2 = &s;  // Can borrow multiple times

println!("{}", s);     // Original still valid!
println!("{}", ref1);  // Reference valid too
println!("{}", ref2);
```

**Key insight:** Many readers are fine; they won't interfere.

### Mutable References

Borrow with permission to modify:

```rust
let mut s = String::from("hello");
let r1 = &mut s;  // Mutable borrow
r1.push_str(" world");

println!("{}", s);  // "hello world"
```

**Crucial rule:** At any moment, you can have EITHER:
- Any number of immutable references, OR
- Exactly ONE mutable reference

```rust
let mut s = String::from("hello");
let r1 = &s;      // Immutable borrow
let r2 = &s;      // OK: another immutable borrow
// let r3 = &mut s;  // ERROR: can't have mutable while immutables exist
```

Why this rule? Mutable access means you could change data while others are reading it—chaos. Immutable references are safe because they can't change anything.

### Functions and Borrowing

**Taking ownership (pass data to function):**
```rust
fn takes_ownership(s: String) {
    println!("{}", s);
}  // s is dropped here

let msg = String::from("hello");
takes_ownership(msg);  // Ownership moves
// msg no longer valid
```

**Borrowing (pass reference to function):**
```rust
fn borrows(s: &str) {
    println!("{}", s);
}  // s is not dropped (it's just a reference)

let msg = String::from("hello");
borrows(&msg);  // Borrow the string
println!("{}", msg);  // msg still valid!
```

**Mutable borrowing (pass mutable reference):**
```rust
fn modify(s: &mut String) {
    s.push_str(" world");
}

let mut msg = String::from("hello");
modify(&mut msg);
println!("{}", msg);  // "hello world"
```

### The Golden Rule

**At any given scope, you can have either:**
- One mutable reference, or
- Any number of immutable references

Not both at the same time.

```rust
let mut s = String::from("hello");

let r1 = &s;     // OK: immutable reference
let r2 = &s;     // OK: another immutable reference
let r3 = &mut s; // ERROR: can't mix mutable and immutable

// Why? r1 and r2 might be used after r3 modifies s!
```

### Dangling References Prevention

Rust prevents you from creating references to freed memory:

```rust
fn make_ref() -> &String {
    let s = String::from("hello");
    &s  // ERROR: s is freed when function returns!
}
```

The compiler catches this. You must return either:
- An owned value (takes responsibility for freeing)
- A reference to data that lives longer

### Code Examples

#### Example 1: Borrowing Prevents Moves
```rust
fn print_length(s: &String) {
    println!("Length: {}", s.len());
}

fn main() {
    let message = String::from("Hello, Rust!");

    print_length(&message);  // Borrow
    print_length(&message);  // Can borrow again

    println!("{}", message); // Original still valid!
}
```

#### Example 2: Mutable Borrowing
```rust
fn append(s: &mut String, suffix: &str) {
    s.push_str(suffix);
}

fn main() {
    let mut text = String::from("Hello");
    append(&mut text, " World");
    println!("{}", text);  // "Hello World"
}
```

#### Example 3: The Borrow Checker Rules
```rust
fn main() {
    let mut s = String::from("hello");

    // Multiple immutable borrows OK
    let r1 = &s;
    let r2 = &s;
    println!("{} {}", r1, r2);

    // Now mutable borrow is OK (immutable refs no longer used)
    let r3 = &mut s;
    r3.push_str(" world");

    println!("{}", r3);  // "hello world"
    // println!("{}", r1);  // ERROR: r1 no longer valid
}
```

#### Example 4: Practical Parser Example
```rust
fn parse_version(data: &[u8]) -> u8 {
    data[0]  // Borrow the slice, don't take ownership
}

fn main() {
    let packet = vec![1, 5, 0, 10, 72, 101, 108, 108, 111];

    let version = parse_version(&packet);  // Borrow as slice
    println!("Version: {}", version);

    println!("Packet: {:?}", packet);  // Still valid!
}
```

### Common Mistakes for Python Developers

1. **Forgetting the `&` in Function Calls**
   ```rust
   // ❌ Wrong: trying to move when you want to borrow
   fn process(s: &str) { /* ... */ }

   let msg = String::from("hello");
   process(msg);  // ERROR: msg is String, not &str

   // ✓ Correct
   process(&msg);  // Borrow as &str
   ```

2. **Confusing & and &mut**
   ```rust
   // ❌ Wrong: using & when you need &mut
   let mut s = String::from("hello");
   let r = &s;
   // r.push_str(" world");  // ERROR: r is immutable reference

   // ✓ Correct
   let r = &mut s;
   r.push_str(" world");  // OK
   ```

3. **Not Realizing Scope of References**
   ```rust
   let mut s = String::from("hello");
   let r1 = &s;
   let r2 = &s;
   println!("{} {}", r1, r2);  // Last use of r1, r2

   // OK to create mutable borrow now (r1, r2 no longer used)
   let r3 = &mut s;
   r3.push_str(" world");
   ```

4. **Trying to Have Multiple Mutable Borrows**
   ```rust
   // ❌ Wrong: two mutable borrows
   let mut s = String::from("hello");
   let r1 = &mut s;
   let r2 = &mut s;  // ERROR

   // ✓ Only one mutable borrow at a time
   let r1 = &mut s;
   r1.push_str(" 1");
   let r2 = &mut s;  // OK: r1 no longer used
   r2.push_str(" 2");
   ```

### Why This Matters

Borrowing enables:
- **Sharing without moving:** Keep using the original value
- **Safe mutation:** `&mut` signals that this function might modify
- **Zero-cost abstractions:** References are just pointers, no copying

For the binary protocol project:
- Parsers will borrow packet bytes: `parse(&packet)`
- Validators will borrow messages: `is_valid(&msg)`
- Error handlers will need references

### Key Takeaways

- ✓ `&T` is an immutable reference; `&mut T` is mutable
- ✓ Multiple immutable references are safe
- ✓ Only one mutable reference at a time
- ✓ Can't mix immutable and mutable references in same scope
- ✓ References don't take ownership; original owner remains valid
- ✓ Use borrows in function parameters to avoid unnecessary moves

---

## Lesson 7: Lifetimes Explained 🟡

### Overview
**Lifetimes** tell the compiler how long references are valid. They ensure you never use a reference to freed memory. Don't worry—lifetimes are usually inferred automatically. This lesson explains what they are and when you need them explicitly.

### Why Lifetimes Exist

A reference must not outlive the data it points to:

```rust
fn make_ref() -> &String {
    let s = String::from("hello");
    &s  // ERROR: s freed when function returns!
}
```

The compiler rejects this because the reference would point to freed memory. Lifetimes are the mechanism that catches this.

### Lifetime Syntax

Lifetimes are written with `'` (apostrophe):

```rust
&'a String       // Reference to String with lifetime 'a
&'static str     // Reference with 'static lifetime (lives for entire program)
&'a mut String   // Mutable reference with lifetime 'a
```

You usually don't write these explicitly—they're inferred. But sometimes the compiler needs help.

### Lifetime Elision

The compiler automatically infers lifetimes in simple cases:

```rust
// These have implicit lifetimes:
fn borrow(s: &String) -> usize { s.len() }
// Really: fn borrow<'a>(s: &'a String) -> usize { s.len() }

// Single input, single output—compiler connects them automatically
```

### When You Need Explicit Lifetimes

When returning a reference, the compiler needs to know: "What is this reference to?"

```rust
// ❌ ERROR: compiler can't infer lifetime of return value
fn first_word(s: &String) -> &str {
    let bytes = s.as_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        if byte == b' ' {
            return &s[0..i];
        }
    }
    &s[..]
}

// ✓ Correct: explicit lifetime shows return borrows from input
fn first_word<'a>(s: &'a String) -> &'a str {
    let bytes = s.as_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        if byte == b' ' {
            return &s[0..i];
        }
    }
    &s[..]
}
```

The `<'a>` says: "This function takes a reference with lifetime 'a and returns a reference with the same lifetime." This ensures the return value doesn't outlive the input.

### Multiple Lifetimes

When multiple references are involved:

```rust
fn merge<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() > s2.len() { s1 } else { s2 }
}
```

This says: "Both inputs and the output share the same lifetime."

If you want different lifetimes:

```rust
fn owned_merge<'a, 'b>(s1: &'a str, s2: &'b str) -> String {
    format!("{}{}", s1, s2)  // Returns owned String, no lifetime needed
}
```

### 'static Lifetime

`'static` means the reference is valid for the entire program:

```rust
let s: &'static str = "Hello, world!";  // String literal
const CONFIG: &'static str = "settings";  // Constants
```

Most references don't have static lifetime—they're scoped to where data exists.

### Code Examples

#### Example 1: Simple Lifetime Inference
```rust
fn main() {
    let s = String::from("Hello");
    let len = get_length(&s);  // Lifetime inferred
    println!("Length: {}", len);
}

// Compiler automatically handles lifetime here
fn get_length(s: &String) -> usize {
    s.len()
}
```

#### Example 2: Explicit Lifetime
```rust
fn first_word<'a>(s: &'a str) -> &'a str {
    let bytes = s.as_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        if byte == b' ' {
            return &s[0..i];
        }
    }
    &s[..]
}

fn main() {
    let s = String::from("hello world");
    let word = first_word(&s);
    println!("First word: {}", word);
}
```

#### Example 3: Multiple Lifetimes
```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

fn main() {
    let s1 = "hello";
    let s2 = "world";
    let result = longest(s1, s2);
    println!("{}", result);
}
```

#### Example 4: Struct with References
```rust
struct Quote<'a> {
    author: &'a str,
    text: &'a str,
}

fn main() {
    let author = "Albert Einstein";
    let text = "Life is like riding a bicycle";

    let quote = Quote { author, text };
    println!("{} - {}", quote.text, quote.author);
}
```

### Common Mistakes for Python Developers

1. **Ignoring Lifetime Errors**
   ```rust
   // ❌ ERROR: trying to return reference to local data
   fn make_string() -> &String {
       let s = String::from("hello");
       &s  // Compiler error!
   }

   // ✓ Return owned value instead
   fn make_string() -> String {
       String::from("hello")
   }
   ```

2. **Forgetting Lifetime Parameters in Structs**
   ```rust
   // ❌ ERROR: struct has reference but no lifetime
   struct Parser {
       data: &[u8],
   }

   // ✓ Add lifetime parameter
   struct Parser<'a> {
       data: &'a [u8],
   }
   ```

3. **Overthinking Lifetimes**
   Most of the time, the compiler infers them. Don't write explicit lifetimes unless the compiler asks.

### Why This Matters

Lifetimes enable:
- **Memory safety:** Compiler prevents use-after-free at compile time
- **Zero overhead:** No runtime cost; checked before execution
- **Flexibility:** You can share references efficiently

For the binary protocol project:
- Parsers will have lifetimes: `parse<'a>(data: &'a [u8])`
- Error types might reference original data
- Tests will verify lifetime correctness

### Key Takeaways

- ✓ Lifetimes ensure references don't outlive their data
- ✓ Write `'a`, `'b`, etc. to name lifetimes
- ✓ Usually inferred automatically (lifetime elision)
- ✓ Only explicit when returning references or in structs with references
- ✓ `'static` is valid for the entire program
- ✓ Multiple references can share lifetimes or have different ones

---

## Lesson 8: Smart Pointers and Collections 🟡

### Overview
Collections are how you store multiple values. **Vec<T>** is Rust's dynamic array, like Python's list. Other important collections include **HashMap** and **String** (which is a collection of characters).

This lesson covers collections and their unique properties in Rust.

### Vec<T> - Dynamic Arrays

Create a vector:

```rust
let mut numbers = vec![1, 2, 3];     // Macro with initial values
let mut numbers: Vec<i32> = Vec::new();  // Empty vector
```

Common operations:

```rust
let mut v = vec![1, 2, 3];

v.push(4);               // Add element
v.pop();                 // Remove last element
println!("Length: {}", v.len());  // Number of elements
println!("First: {}", v[0]);      // Index (panics if out of bounds)

for item in &v {         // Iterate
    println!("{}", item);
}

let first = v.get(0);    // Safe access (Option)
```

### Vectors and Ownership

When you create a vector, you own it. When it goes out of scope, it's freed:

```rust
let v = vec![1, 2, 3];
// When v goes out of scope, the memory is freed automatically
```

Passing vectors:

```rust
fn sum(numbers: Vec<i32>) -> i32 {
    numbers.iter().sum()
}  // numbers freed here

let v = vec![1, 2, 3];
sum(v);  // Ownership moves
// println!("{:?}", v);  // ERROR: v was moved
```

Better: borrow instead:

```rust
fn sum(numbers: &[i32]) -> i32 {
    numbers.iter().sum()
}

let v = vec![1, 2, 3];
sum(&v);  // Borrow
println!("{:?}", v);  // Still valid
```

### Iterating Vectors

Three ways:

```rust
let v = vec![1, 2, 3];

// 1. Immutable reference (read-only)
for item in &v {
    println!("{}", item);
}

// 2. Mutable reference (can modify)
for item in &mut v {
    *item *= 2;  // Dereference and modify
}

// 3. Ownership (can't use v afterward)
for item in v {
    println!("{}", item);
}
// v is no longer valid
```

### Slices

A slice is a contiguous sequence of elements, like a borrowed view:

```rust
let v = vec![1, 2, 3, 4, 5];
let slice = &v[1..3];  // Elements at index 1 and 2

// Works with arrays too
let arr = [1, 2, 3, 4, 5];
let slice = &arr[1..3];
```

Slices are powerful for parsing—you can pass a range of bytes without copying.

### HashMap - Key-Value Storage

```rust
use std::collections::HashMap;

let mut scores = HashMap::new();
scores.insert("Alice", 50);
scores.insert("Bob", 75);

println!("{:?}", scores.get("Alice"));  // Some(50)
```

### String as a Collection

Strings are actually collections of bytes:

```rust
let s = String::from("Hello");
let s_bytes = s.as_bytes();  // &[u8]
let s_str = &s;              // &str (string slice)
```

### Code Examples

#### Example 1: Working with Vectors
```rust
fn main() {
    let mut fruits = vec!["apple", "banana", "cherry"];

    fruits.push("date");
    println!("Count: {}", fruits.len());

    for fruit in &fruits {
        println!("- {}", fruit);
    }

    // Get by index safely
    match fruits.get(1) {
        Some(fruit) => println!("Second: {}", fruit),
        None => println!("No second element"),
    }
}
```

#### Example 2: Vector Transformation
```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];

    // Create a new vector with transformed values
    let doubled: Vec<i32> = numbers.iter().map(|x| x * 2).collect();

    // Filter and collect
    let evens: Vec<i32> = numbers.iter()
        .filter(|x| x % 2 == 0)
        .copied()
        .collect();

    println!("Doubled: {:?}", doubled);
    println!("Evens: {:?}", evens);
}
```

#### Example 3: Using Slices in Functions
```rust
fn sum_slice(numbers: &[i32]) -> i32 {
    numbers.iter().sum()
}

fn main() {
    let v = vec![1, 2, 3, 4, 5];
    let arr = [1, 2, 3];

    println!("Vec sum: {}", sum_slice(&v));
    println!("Array sum: {}", sum_slice(&arr));

    // Slice of a vector
    println!("Slice sum: {}", sum_slice(&v[1..3]));
}
```

#### Example 4: Binary Protocol Parsing with Slices
```rust
fn parse_version(packet: &[u8]) -> u8 {
    packet[0]
}

fn parse_message_type(packet: &[u8]) -> u8 {
    packet[1]
}

fn parse_payload(packet: &[u8]) -> &[u8] {
    &packet[4..packet.len()-1]  // Skip header, exclude checksum
}

fn main() {
    let packet = vec![1, 5, 0, 10, 72, 101, 108, 108, 111];

    println!("Version: {}", parse_version(&packet));
    println!("Type: {}", parse_message_type(&packet));
    println!("Payload: {:?}", parse_payload(&packet));
}
```

### Common Mistakes for Python Developers

1. **Panicking on Out-of-Bounds Access**
   ```rust
   // ❌ Panics if index out of bounds
   let v = vec![1, 2, 3];
   let x = v[10];  // PANIC!

   // ✓ Safe access
   match v.get(10) {
       Some(x) => println!("{}", x),
       None => println!("Out of bounds"),
   }
   ```

2. **Forgetting Borrowing in Functions**
   ```rust
   // ❌ Takes ownership
   fn process(v: Vec<i32>) { /* ... */ }
   let v = vec![1, 2, 3];
   process(v);
   // println!("{:?}", v);  // ERROR: moved

   // ✓ Borrow instead
   fn process(v: &[i32]) { /* ... */ }
   process(&v);
   println!("{:?}", v);  // OK
   ```

3. **Modifying While Iterating**
   ```rust
   // ❌ Can't modify with immutable iterator
   let v = vec![1, 2, 3];
   for item in &v {
       v.push(*item);  // ERROR
   }

   // If you need to modify, collect first
   let v = vec![1, 2, 3];
   let mut new_items = Vec::new();
   for item in &v {
       new_items.push(*item);
   }
   ```

### Why This Matters

Collections enable:
- **Storage:** Keep multiple values together
- **Efficiency:** Vectors are optimized for access and modification
- **Type Safety:** Compile-time checking of operations
- **Zero-Copy Borrowing:** Slices avoid unnecessary copies

For the binary protocol project:
- Packets are `Vec<u8>` (owned bytes)
- Payloads are `&[u8]` (borrowed slices)
- Parsing creates vectors of messages

### Key Takeaways

- ✓ `Vec<T>` is a dynamic array; use `vec![]` macro or `Vec::new()`
- ✓ Access with indexing `v[i]` or safely with `v.get(i)`
- ✓ Borrowing vectors: `&[T]` is more flexible than `Vec<T>`
- ✓ Iterate with `for item in &v`, `for item in &mut v`, or `for item in v`
- ✓ Slices `&[T]` are zero-cost views into sequential data
- ✓ HashMap for key-value storage

---

## Lesson 9: Pattern Matching and Destructuring 🟡

### Overview
**Pattern matching** is one of Rust's most powerful features. It lets you extract data from complex types and handle multiple cases with exhaustiveness checking. Every path must be handled.

### Match Expressions

Basic syntax:

```rust
match value {
    pattern1 => result1,
    pattern2 => result2,
    _ => default_result,  // Catch-all
}
```

**Exhaustiveness:** The compiler ensures every possibility is handled.

```rust
let grade = 'A';
let result = match grade {
    'A' => "Excellent",
    'B' => "Good",
    'C' => "Pass",
    'D' => "Barely",
    'F' => "Fail",
    _ => "Unknown",  // Catch-all required if all cases not covered
};
```

### Destructuring Structs

Extract struct fields:

```rust
struct Point(i32, i32);

let p = Point(3, 4);
match p {
    Point(x, y) => println!("x: {}, y: {}", x, y),
}
```

With named structs:

```rust
struct User {
    name: String,
    age: u8,
}

let user = User { name: String::from("Alice"), age: 30 };
match user {
    User { name, age } => {
        println!("{} is {} years old", name, age);
    }
}
```

### If Let - Single Pattern

When you only care about one pattern:

```rust
let maybe_num = Some(5);

if let Some(num) = maybe_num {
    println!("Got number: {}", num);
} else {
    println!("No number");
}
```

More concise than `match` for single patterns.

### While Let - Loop Pattern

Repeatedly match until pattern fails:

```rust
let mut stack = vec![1, 2, 3];

while let Some(num) = stack.pop() {
    println!("{}", num);  // Prints 3, 2, 1
}
```

### Match Guards

Add conditions to patterns:

```rust
let age = 25;
match age {
    n if n < 18 => println!("Minor"),
    n if n < 65 => println!("Adult"),
    _ => println!("Senior"),
}
```

### Combining Patterns

Pattern matching is incredibly flexible:

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
}

let msg = Message::Move { x: 10, y: 20 };

match msg {
    Message::Quit => println!("Quit"),
    Message::Move { x, y } => println!("Move to ({}, {})", x, y),
    Message::Write(text) => println!("Write: {}", text),
}
```

### Code Examples

#### Example 1: Basic Matching
```rust
fn describe_number(n: i32) {
    match n {
        0 => println!("Zero"),
        1 => println!("One"),
        2..=10 => println!("Two to ten"),
        11..=100 => println!("Eleven to hundred"),
        _ => println!("Large number"),
    }
}

fn main() {
    describe_number(5);
    describe_number(50);
    describe_number(1000);
}
```

#### Example 2: Destructuring and Pattern Matching
```rust
struct Packet {
    version: u8,
    message_type: u8,
    length: u16,
}

fn main() {
    let packet = Packet {
        version: 1,
        message_type: 5,
        length: 10,
    };

    match packet {
        Packet { version: 1, message_type, length } => {
            println!("V1 message type {}, length {}", message_type, length);
        }
        _ => println!("Unknown protocol version"),
    }
}
```

#### Example 3: Matching with Guards
```rust
fn categorize_age(age: u8) {
    match age {
        n if n < 13 => println!("Child"),
        n if n < 18 => println!("Teen"),
        n if n < 65 => println!("Adult"),
        _ => println!("Senior"),
    }
}

fn main() {
    categorize_age(10);
    categorize_age(25);
    categorize_age(70);
}
```

#### Example 4: If Let vs Match
```rust
fn main() {
    let config = Some(String::from("setup.cfg"));

    // If let for single pattern
    if let Some(filename) = config {
        println!("Using config: {}", filename);
    }

    // While let for looping
    let mut stack = vec![1, 2, 3];
    while let Some(num) = stack.pop() {
        println!("Popped: {}", num);
    }
}
```

### Common Mistakes for Python Developers

1. **Not Being Exhaustive**
   ```rust
   // ❌ ERROR: missing patterns
   match value {
       Some(x) => println!("{}", x),
       // ERROR: no pattern for None!
   }

   // ✓ Correct
   match value {
       Some(x) => println!("{}", x),
       None => println!("No value"),
   }
   ```

2. **Using Catch-All Too Much**
   ```rust
   // ❌ Misses explicit patterns
   match value {
       _ => println!("Something"),  // Too broad!
   }

   // ✓ Better
   match value {
       Some(x) => println!("Got: {}", x),
       None => println!("Nothing"),
   }
   ```

3. **Forgetting to Bind Values**
   ```rust
   // ❌ Pattern doesn't extract value
   match maybe_num {
       Some(_) => {
           // println!("{}", maybe_num);  // Can't use it!
       }
       None => {}
   }

   // ✓ Correct
   match maybe_num {
       Some(num) => println!("{}", num),
       None => {}
   }
   ```

### Why This Matters

Pattern matching enables:
- **Exhaustive checking:** Compiler ensures all cases handled
- **Safe extraction:** Extract data without runtime type checks
- **Concise code:** Replace many if-else chains
- **Type safety:** Patterns work with enums and custom types

For the binary protocol project:
- Matching on `Result` to handle parse errors
- Matching on `Option` for optional fields
- Destructuring parsed message fields

### Key Takeaways

- ✓ `match` is exhaustive—all patterns must be handled
- ✓ Use `_` as catch-all for unmatched patterns
- ✓ `if let` for single pattern matching
- ✓ `while let` to loop until pattern fails
- ✓ Pattern guards (`if`) add conditions
- ✓ Destructuring extracts values from structs and enums

---

## Lesson 10: Enums and Associated Data 🟡

### Overview
**Enums** (enumerations) let you define a type that can be one of several variants. Unlike structs (which have all fields), enums let you represent "one of these choices." They're especially powerful when combined with associated data.

### Enum Basics

Simple enum without data:

```rust
enum Direction {
    North,
    South,
    East,
    West,
}

let way = Direction::North;
```

### Enums with Associated Data

The powerful part—variants can carry data:

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(u8, u8, u8),
}

let m1 = Message::Quit;
let m2 = Message::Move { x: 10, y: 20 };
let m3 = Message::Write(String::from("hello"));
let m4 = Message::ChangeColor(255, 100, 50);
```

Each variant can have different data types.

### Pattern Matching with Enums

Match on variants:

```rust
match message {
    Message::Quit => println!("Quit"),
    Message::Move { x, y } => println!("Move to ({}, {})", x, y),
    Message::Write(text) => println!("Write: {}", text),
    Message::ChangeColor(r, g, b) => println!("Color: ({}, {}, {})", r, g, b),
}
```

### Methods on Enums

Like structs, enums can have methods:

```rust
impl Message {
    fn call(&self) {
        match self {
            Message::Quit => println!("Quit"),
            Message::Move { x, y } => println!("Move to ({}, {})", x, y),
            _ => {}
        }
    }
}

let message = Message::Move { x: 10, y: 20 };
message.call();
```

### Option and Result

Two critical built-in enums:

**Option<T>** - represents something that might exist:
```rust
enum Option<T> {
    Some(T),
    None,
}

let maybe_number: Option<i32> = Some(5);
let nothing: Option<i32> = None;
```

**Result<T, E>** - represents success or failure:
```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}

let success: Result<i32, String> = Ok(42);
let failure: Result<i32, String> = Err(String::from("Error!"));
```

### Code Examples

#### Example 1: Simple Enum
```rust
enum Color {
    Red,
    Green,
    Blue,
}

fn main() {
    let color = Color::Red;

    match color {
        Color::Red => println!("Stop!"),
        Color::Green => println!("Go!"),
        Color::Blue => println!("Caution!"),
    }
}
```

#### Example 2: Enum with Associated Data
```rust
enum Packet {
    Handshake(u32),
    Data(Vec<u8>),
    Close(String),
}

fn handle_packet(packet: Packet) {
    match packet {
        Packet::Handshake(id) => println!("Handshake {}", id),
        Packet::Data(bytes) => println!("Data: {:?}", bytes),
        Packet::Close(reason) => println!("Closed: {}", reason),
    }
}

fn main() {
    let p1 = Packet::Handshake(123);
    let p2 = Packet::Data(vec![1, 2, 3]);
    let p3 = Packet::Close(String::from("Goodbye"));

    handle_packet(p1);
    handle_packet(p2);
    handle_packet(p3);
}
```

#### Example 3: Enum with Methods
```rust
enum Status {
    Waiting,
    Running { progress: u8 },
    Done { result: String },
}

impl Status {
    fn describe(&self) -> String {
        match self {
            Status::Waiting => "Waiting to start".to_string(),
            Status::Running { progress } => format!("Running... {}%", progress),
            Status::Done { result } => format!("Done: {}", result),
        }
    }
}

fn main() {
    let s = Status::Running { progress: 50 };
    println!("{}", s.describe());
}
```

#### Example 4: Using Option and Result
```rust
fn find_user(id: u32) -> Option<String> {
    if id == 1 {
        Some(String::from("Alice"))
    } else {
        None
    }
}

fn parse_number(s: &str) -> Result<i32, String> {
    match s.parse::<i32>() {
        Ok(n) => Ok(n),
        Err(_) => Err(format!("Can't parse '{}'", s)),
    }
}

fn main() {
    match find_user(1) {
        Some(name) => println!("Found: {}", name),
        None => println!("User not found"),
    }

    match parse_number("42") {
        Ok(n) => println!("Number: {}", n),
        Err(e) => println!("Error: {}", e),
    }
}
```

### Common Mistakes for Python Developers

1. **Forgetting Variants Need a Value**
   ```rust
   // ❌ Wrong: variant path without construction
   let m = Message::Move;  // ERROR: missing data

   // ✓ Correct
   let m = Message::Move { x: 10, y: 20 };
   ```

2. **Not Matching All Variants**
   ```rust
   // ❌ Missing pattern
   match msg {
       Message::Quit => println!("Quit"),
       // ERROR: Message::Move not handled!
   }

   // ✓ Correct (all patterns)
   match msg {
       Message::Quit => println!("Quit"),
       Message::Move { x, y } => println!("Move"),
       _ => println!("Other"),
   }
   ```

3. **Treating Enums Like Classes**
   ```rust
   // ❌ Trying to construct like a class
   let m = Message::Move(10, 20);  // Wrong!

   // ✓ Correct
   let m = Message::Move { x: 10, y: 20 };
   ```

### Why This Matters

Enums enable:
- **Type-safe choices:** Only valid variants allowed
- **Associated data:** Different variants carry different data
- **Exhaustiveness:** Compiler ensures all cases handled
- **Semantics:** "This is one of these specific things"

For the binary protocol project:
- `MessageType` enum for different packet types
- `ParseError` enum for different error cases
- `PacketField` enum for optional fields

### Key Takeaways

- ✓ Enums represent "one of several choices"
- ✓ Variants can have associated data
- ✓ Use pattern matching to handle variants
- ✓ Option<T> and Result<T, E> are built-in enums
- ✓ Enums can have methods like structs
- ✓ Matching is exhaustive—compiler ensures all cases

---

## Lesson 11: Option and Result - Error Handling 🟡

### Overview
**Option** and **Result** are Rust's way of handling absence of values and errors respectively. They replace try-catch blocks with compile-time guarantees that errors are handled.

### Option<T>

Represents a value that might not exist:

```rust
enum Option<T> {
    Some(T),    // Value exists
    None,       // Value doesn't exist
}
```

```rust
let maybe_number: Option<i32> = Some(5);
let nothing: Option<i32> = None;

match maybe_number {
    Some(num) => println!("Got: {}", num),
    None => println!("No value"),
}
```

### Result<T, E>

Represents success (Ok) or failure (Err):

```rust
enum Result<T, E> {
    Ok(T),      // Success
    Err(E),     // Failure
}
```

```rust
let success: Result<i32, String> = Ok(42);
let failure: Result<i32, String> = Err(String::from("Something went wrong"));

match success {
    Ok(value) => println!("Success: {}", value),
    Err(e) => println!("Error: {}", e),
}
```

### The `?` Operator - Error Propagation

The `?` operator unwraps a Result. If it's Ok, continue. If Err, return immediately:

```rust
fn parse_int(s: &str) -> Result<i32, String> {
    s.parse::<i32>().map_err(|_| String::from("Invalid number"))
}

fn get_sum(a_str: &str, b_str: &str) -> Result<i32, String> {
    let a = parse_int(a_str)?;  // If error, return immediately
    let b = parse_int(b_str)?;  // If error, return immediately
    Ok(a + b)                    // Success
}

fn main() {
    match get_sum("10", "20") {
        Ok(sum) => println!("Sum: {}", sum),
        Err(e) => println!("Error: {}", e),
    }
}
```

**Without `?`, you'd need extensive matching:**
```rust
// Verbose without ?
fn get_sum_verbose(a_str: &str, b_str: &str) -> Result<i32, String> {
    let a = match parse_int(a_str) {
        Ok(n) => n,
        Err(e) => return Err(e),
    };
    let b = match parse_int(b_str) {
        Ok(n) => n,
        Err(e) => return Err(e),
    };
    Ok(a + b)
}
```

The `?` operator saves tons of boilerplate.

### Handling Option and Result

**Unwrap (unsafe):**
```rust
let maybe = Some(5);
let value = maybe.unwrap();  // Panics if None!
```

**Unwrap with default:**
```rust
let maybe: Option<i32> = None;
let value = maybe.unwrap_or(0);  // Returns 0 if None
```

**If let:**
```rust
let maybe = Some(5);
if let Some(value) = maybe {
    println!("Got: {}", value);
}
```

**Map to transform:**
```rust
let maybe = Some(5);
let doubled = maybe.map(|x| x * 2);  // Some(10)
```

### Code Examples

#### Example 1: Option Usage
```rust
fn find_user(id: u32) -> Option<String> {
    if id == 1 {
        Some(String::from("Alice"))
    } else {
        None
    }
}

fn main() {
    let user = find_user(1);

    match user {
        Some(name) => println!("Found: {}", name),
        None => println!("No user"),
    }

    // Using if let
    if let Some(name) = find_user(2) {
        println!("Found: {}", name);
    } else {
        println!("User not found");
    }
}
```

#### Example 2: Result with Error Handling
```rust
fn parse_version(s: &str) -> Result<u8, String> {
    match s.parse::<u8>() {
        Ok(v) if v <= 255 => Ok(v),
        Ok(_) => Err("Version out of range".to_string()),
        Err(_) => Err("Invalid version format".to_string()),
    }
}

fn main() {
    match parse_version("1") {
        Ok(v) => println!("Version: {}", v),
        Err(e) => println!("Error: {}", e),
    }
}
```

#### Example 3: The `?` Operator
```rust
use std::num::ParseIntError;

fn parse_pair(s: &str) -> Result<(i32, i32), ParseIntError> {
    let parts: Vec<&str> = s.split(',').collect();
    let x = parts[0].parse::<i32>()?;  // ? propagates error
    let y = parts[1].parse::<i32>()?;  // ? propagates error
    Ok((x, y))
}

fn main() {
    match parse_pair("10,20") {
        Ok((x, y)) => println!("Pair: ({}, {})", x, y),
        Err(_) => println!("Invalid pair"),
    }
}
```

#### Example 4: Practical Binary Protocol Parsing
```rust
struct Message {
    version: u8,
    message_type: u8,
    payload: Vec<u8>,
}

fn parse_message(data: &[u8]) -> Result<Message, String> {
    if data.len() < 4 {
        return Err("Data too short".to_string());
    }

    let version = data[0];
    let message_type = data[1];
    let length = u16::from_be_bytes([data[2], data[3]]) as usize;

    if data.len() < 4 + length {
        return Err("Incomplete payload".to_string());
    }

    let payload = data[4..4 + length].to_vec();

    Ok(Message { version, message_type, payload })
}

fn main() {
    let packet = vec![1, 5, 0, 5, 72, 101, 108, 108, 111];

    match parse_message(&packet) {
        Ok(msg) => println!("Parsed: v{} type{}", msg.version, msg.message_type),
        Err(e) => println!("Parse error: {}", e),
    }
}
```

### Common Mistakes for Python Developers

1. **Using `.unwrap()` in Production Code**
   ```rust
   // ❌ Panics if None/Err
   let value = result.unwrap();

   // ✓ Handle the error
   let value = result.unwrap_or_default();
   // or use match, or ?
   ```

2. **Forgetting to Handle Both Cases**
   ```rust
   // ❌ Wrong: only handles Ok
   if let Ok(v) = result {
       println!("{}", v);
   }

   // ✓ Handle both (or use match)
   match result {
       Ok(v) => println!("{}", v),
       Err(e) => println!("Error: {}", e),
   }
   ```

3. **Not Understanding `?` Return Type Requirement**
   ```rust
   // ❌ ERROR: ? only works in Result/Option returning functions
   fn main() {
       let value = Some(5);
       let x = value?;  // ERROR: main doesn't return Option/Result
   }

   // ✓ Correct: use match or if let
   fn main() {
       if let Some(x) = value {
           println!("{}", x);
       }
   }
   ```

### Why This Matters

Option and Result enable:
- **Explicit error handling:** Compiler enforces handling
- **No silent failures:** Can't forget to check for errors
- **Concise error propagation:** `?` eliminates boilerplate
- **Type safety:** Errors are values, not hidden exceptions

For the binary protocol project:
- `Result<Message, ParseError>` for parsing
- `Option<u8>` for optional fields
- `?` for error propagation in parsers

### Key Takeaways

- ✓ `Option<T>` for values that might not exist
- ✓ `Result<T, E>` for operations that might fail
- ✓ Use `match` for complete handling
- ✓ Use `if let` for single-case handling
- ✓ The `?` operator propagates errors automatically
- ✓ Never use `.unwrap()` in production unless certain

---

## Lesson 12: Custom Error Types 🟡

### Overview
Creating custom error types enables clear, domain-specific error messages. Instead of generic strings, define error enums that represent real failure modes in your code.

### Defining Custom Errors

Simple enum-based error:

```rust
enum ParseError {
    InvalidVersion,
    MessageTooShort,
    ChecksumMismatch,
    InvalidPayload,
}
```

### Implementing the Error Trait

```rust
use std::fmt;
use std::error::Error;

#[derive(Debug)]
enum ParseError {
    InvalidVersion,
    MessageTooShort,
    ChecksumMismatch,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidVersion => write!(f, "Invalid protocol version"),
            ParseError::MessageTooShort => write!(f, "Message data too short"),
            ParseError::ChecksumMismatch => write!(f, "Checksum verification failed"),
        }
    }
}

impl Error for ParseError {}
```

### Using Custom Errors with Result

```rust
fn parse_message(data: &[u8]) -> Result<Message, ParseError> {
    if data.len() < 4 {
        return Err(ParseError::MessageTooShort);
    }

    let version = data[0];
    if version != 1 {
        return Err(ParseError::InvalidVersion);
    }

    // ... rest of parsing
    Ok(message)
}
```

### Error Context with `From` Trait

```rust
use std::num::ParseIntError;

enum ParseError {
    InvalidVersion,
    NumberParseError(ParseIntError),
}

impl From<ParseIntError> for ParseError {
    fn from(err: ParseIntError) -> Self {
        ParseError::NumberParseError(err)
    }
}

fn parse_length(s: &str) -> Result<u16, ParseError> {
    let n = s.parse::<u16>()?;  // ? automatically converts
    Ok(n)
}
```

### Code Examples

#### Example 1: Simple Custom Error
```rust
use std::error::Error;
use std::fmt;

#[derive(Debug)]
enum FileError {
    NotFound,
    PermissionDenied,
    ReadFailed,
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileError::NotFound => write!(f, "File not found"),
            FileError::PermissionDenied => write!(f, "Permission denied"),
            FileError::ReadFailed => write!(f, "Failed to read file"),
        }
    }
}

impl Error for FileError {}

fn main() {
    let result: Result<String, FileError> = Err(FileError::NotFound);
    match result {
        Ok(content) => println!("Content: {}", content),
        Err(e) => println!("Error: {}", e),
    }
}
```

#### Example 2: Error with Context
```rust
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
enum ValidationError {
    InvalidAge(String),
    ParseError(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidationError::InvalidAge(msg) => write!(f, "Invalid age: {}", msg),
            ValidationError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl Error for ValidationError {}

fn parse_age(s: &str) -> Result<u8, ValidationError> {
    let age = s.parse::<u8>()
        .map_err(|e| ValidationError::ParseError(e.to_string()))?;

    if age > 150 {
        return Err(ValidationError::InvalidAge("Age too high".to_string()));
    }

    Ok(age)
}

fn main() {
    match parse_age("abc") {
        Ok(age) => println!("Age: {}", age),
        Err(e) => println!("Error: {}", e),
    }
}
```

#### Example 3: Binary Protocol Custom Errors
```rust
use std::error::Error;
use std::fmt;

#[derive(Debug)]
enum ProtocolError {
    InvalidVersion(u8),
    PayloadTooLarge(usize),
    ChecksumMismatch { expected: u8, got: u8 },
    MessageTooShort(usize),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProtocolError::InvalidVersion(v) => {
                write!(f, "Unsupported protocol version: {}", v)
            }
            ProtocolError::PayloadTooLarge(size) => {
                write!(f, "Payload too large: {} bytes", size)
            }
            ProtocolError::ChecksumMismatch { expected, got } => {
                write!(f, "Checksum mismatch: expected {}, got {}", expected, got)
            }
            ProtocolError::MessageTooShort(len) => {
                write!(f, "Message too short: {} bytes", len)
            }
        }
    }
}

impl Error for ProtocolError {}

fn parse_protocol_message(data: &[u8]) -> Result<Message, ProtocolError> {
    if data.len() < 5 {
        return Err(ProtocolError::MessageTooShort(data.len()));
    }

    let version = data[0];
    if version != 1 {
        return Err(ProtocolError::InvalidVersion(version));
    }

    // ... parsing logic
    Ok(Message { /* ... */ })
}
```

### Common Mistakes for Python Developers

1. **Forgetting Debug Derive**
   ```rust
   // ❌ Error doesn't implement Debug
   enum MyError {
       Something,
   }

   // ✓ Add #[derive(Debug)]
   #[derive(Debug)]
   enum MyError {
       Something,
   }
   ```

2. **Not Implementing Display**
   ```rust
   // ❌ Missing Display implementation
   impl Error for MyError {}

   // ✓ Implement both Display and Error
   impl Display for MyError { /* ... */ }
   impl Error for MyError {}
   ```

3. **Generic Error Strings**
   ```rust
   // ❌ Vague errors
   Err("error".to_string())

   // ✓ Specific error types
   Err(ProtocolError::ChecksumMismatch { expected, got })
   ```

### Why This Matters

Custom errors enable:
- **Clarity:** Specific error types document failure modes
- **Handling:** Different errors can be handled differently
- **Debugging:** Rich error context helps troubleshooting
- **API design:** Public functions communicate failure modes

For the binary protocol project:
- `ProtocolError` with variants for each failure mode
- Context in errors (expected vs. actual checksums)
- Clear error messages for debugging

### Key Takeaways

- ✓ Define error types as enums with specific variants
- ✓ Implement `Display` for human-readable messages
- ✓ Implement `Error` trait (empty trait, but required)
- ✓ Add `#[derive(Debug)]` automatically
- ✓ Use enums to capture error context (values involved)
- ✓ Implement `From` for automatic error conversion with `?`

---

## Lesson 13: Modules and Visibility 🟡

### Overview
**Modules** organize your code into logical units. **Visibility** controls what other modules can access. Together, they enable encapsulation and prevent accidental misuse.

### Module Basics

Modules create a namespace:

```rust
mod network {
    pub fn send_packet() {
        println!("Sending packet");
    }

    fn internal_function() {
        // Private: only accessible within this module
    }
}

fn main() {
    network::send_packet();  // OK
    // network::internal_function();  // ERROR: private
}
```

### Visibility: pub vs Private

By default, everything is **private** (only visible within the module):

```rust
mod api {
    pub fn public_function() {}  // Accessible from outside
    fn private_function() {}     // Only within module

    pub struct PublicStruct {
        pub field: i32,          // Accessible
        private_field: String,   // Only within module
    }
}
```

### Nested Modules

Modules can contain other modules:

```rust
mod network {
    pub mod tcp {
        pub fn connect() { /* ... */ }
    }

    pub mod udp {
        pub fn send() { /* ... */ }
    }
}

fn main() {
    network::tcp::connect();
}
```

### Using `use` for Convenience

Import items to avoid full paths:

```rust
use network::tcp::connect;

fn main() {
    connect();  // Instead of network::tcp::connect()
}
```

### Re-exporting with `pub use`

Make a module's items accessible through your module:

```rust
mod internal {
    pub fn helper() { /* ... */ }
}

pub use internal::helper;  // Now accessible as module::helper
```

### Module File Organization

For larger projects, modules can live in separate files:

```
src/
  main.rs
  lib.rs
  network.rs          // mod network { ... }
  network/
    tcp.rs           // pub mod tcp { ... }
    udp.rs           // pub mod udp { ... }
```

**lib.rs:**
```rust
pub mod network;  // Loads network.rs or network/mod.rs

pub fn public_api() { /* ... */ }
```

**network.rs:**
```rust
pub mod tcp;
pub mod udp;

pub fn network_helper() { /* ... */ }
```

### Code Examples

#### Example 1: Basic Module Organization
```rust
mod parser {
    pub struct Message {
        pub version: u8,
        pub data: Vec<u8>,
    }

    pub fn parse(bytes: &[u8]) -> Message {
        // Parsing logic
        Message { version: 1, data: vec![] }
    }

    fn internal_helper() {
        // Hidden from outside
    }
}

fn main() {
    let msg = parser::parse(&[1, 2, 3]);
    println!("Version: {}", msg.version);
}
```

#### Example 2: Using pub use for Convenience
```rust
mod internal {
    pub fn important_function() {
        println!("Important!");
    }
}

pub use internal::important_function;

fn main() {
    important_function();  // Can call directly now
}
```

#### Example 3: Nested Modules
```rust
pub mod protocol {
    pub mod message {
        pub struct Header {
            pub version: u8,
            pub msg_type: u8,
        }

        pub fn create(version: u8, msg_type: u8) -> Header {
            Header { version, msg_type }
        }
    }

    pub mod validation {
        pub fn validate_version(v: u8) -> bool {
            v >= 1 && v <= 3
        }
    }
}

fn main() {
    let header = protocol::message::create(1, 5);
    if protocol::validation::validate_version(header.version) {
        println!("Valid version");
    }
}
```

### Common Mistakes for Python Developers

1. **Forgetting `pub` for Public Items**
   ```rust
   // ❌ Function is private by default
   mod api {
       fn public_function() {}  // ERROR: private
   }

   // ✓ Make it public
   mod api {
       pub fn public_function() {}
   }
   ```

2. **Not Organizing Modules Logically**
   ```rust
   // ❌ Everything in one file gets messy
   // ✓ Separate parsing, validation, errors into modules
   ```

3. **Overusing `pub` (Exposing Internals)**
   ```rust
   // ❌ Exposing too much
   pub struct Message {
       pub internal_state: Vec<u8>,
       pub cache: HashMap<String, String>,
   }

   // ✓ Hide internals
   pub struct Message {
       // Private fields
   }
   impl Message {
       pub fn new() { /* ... */ }
       pub fn validate(&self) -> Result<(), Error> { /* ... */ }
   }
   ```

### Why This Matters

Modules enable:
- **Organization:** Group related code
- **Encapsulation:** Hide implementation details
- **Safety:** Prevent accidental misuse
- **Clarity:** Clear public API

For the binary protocol project:
- `mod parser` for parsing logic
- `mod error` for error types
- `mod validation` for validation logic
- Clear public API, hidden internals

### Key Takeaways

- ✓ Modules create namespaces and organize code
- ✓ Everything is private by default
- ✓ Use `pub` to make items publicly accessible
- ✓ Use `use` to import and shorten paths
- ✓ `pub use` re-exports items from submodules
- ✓ Larger projects: organize modules into separate files

---

## Lesson 14: Traits and Polymorphism 🔴

### Overview
**Traits** define shared behavior. They enable **polymorphism**—different types can implement the same trait, allowing code to work with any type that satisfies the trait.

### Defining Traits

A trait is a collection of methods:

```rust
trait Animal {
    fn speak(&self) -> String;
    fn name(&self) -> &str;
}
```

### Implementing Traits

Implement a trait for a type:

```rust
struct Dog;
impl Animal for Dog {
    fn speak(&self) -> String {
        String::from("Woof!")
    }
    fn name(&self) -> &str {
        "Dog"
    }
}

struct Cat;
impl Animal for Cat {
    fn speak(&self) -> String {
        String::from("Meow!")
    }
    fn name(&self) -> &str {
        "Cat"
    }
}
```

### Trait Bounds

Use traits in function signatures:

```rust
fn make_sound(animal: &dyn Animal) {
    println!("{} says: {}", animal.name(), animal.speak());
}

fn main() {
    let dog = Dog;
    let cat = Cat;

    make_sound(&dog);
    make_sound(&cat);
}
```

`dyn Animal` means "any type that implements Animal."

### Generics with Trait Bounds

More type-safe than trait objects:

```rust
fn describe<T: Animal>(animal: T) {
    println!("{} says: {}", animal.name(), animal.speak());
}

fn main() {
    let dog = Dog;
    describe(dog);  // Monomorphization: compiler creates separate code for Dog
}
```

### Common Traits

**Display** - human-readable formatting:
```rust
use std::fmt;

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Message: v{}", self.version)
    }
}

println!("{}", message);  // Uses Display
```

**Debug** - developer-friendly formatting:
```rust
#[derive(Debug)]
struct Message {
    version: u8,
}

println!("{:?}", message);  // Uses Debug
```

**Clone** - copying:
```rust
#[derive(Clone)]
struct Message {
    version: u8,
}

let msg2 = msg1.clone();
```

**PartialEq** - equality comparison:
```rust
#[derive(PartialEq)]
struct Message {
    version: u8,
}

if msg1 == msg2 { /* ... */ }
```

### Code Examples

#### Example 1: Implementing Traits
```rust
use std::fmt;

trait Serializable {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Self;
}

struct Person {
    name: String,
    age: u8,
}

impl Serializable for Person {
    fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.push(self.age);
        result.extend_from_slice(self.name.as_bytes());
        result
    }

    fn deserialize(data: &[u8]) -> Self {
        let age = data[0];
        let name = String::from_utf8_lossy(&data[1..]).to_string();
        Person { name, age }
    }
}
```

#### Example 2: Display and Debug Traits
```rust
use std::fmt;

struct Packet {
    version: u8,
    message_type: u8,
    payload: Vec<u8>,
}

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Packet v{} type {} ({} bytes)",
               self.version, self.message_type, self.payload.len())
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Packet")
            .field("version", &self.version)
            .field("message_type", &self.message_type)
            .field("payload_len", &self.payload.len())
            .finish()
    }
}
```

#### Example 3: Trait Objects for Polymorphism
```rust
trait Validator {
    fn validate(&self, data: &[u8]) -> Result<(), String>;
}

struct ChecksumValidator;
impl Validator for ChecksumValidator {
    fn validate(&self, data: &[u8]) -> Result<(), String> {
        // Checksum validation logic
        Ok(())
    }
}

struct LengthValidator;
impl Validator for LengthValidator {
    fn validate(&self, data: &[u8]) -> Result<(), String> {
        if data.len() > 1024 {
            Err("Data too long".to_string())
        } else {
            Ok(())
        }
    }
}

fn validate_all(data: &[u8], validators: &[Box<dyn Validator>]) -> Result<(), String> {
    for validator in validators {
        validator.validate(data)?;
    }
    Ok(())
}
```

### Common Mistakes for Python Developers

1. **Forgetting `dyn` for Trait Objects**
   ```rust
   // ❌ Wrong: missing dyn
   fn process(item: &Animal) { /* ... */ }

   // ✓ Correct
   fn process(item: &dyn Animal) { /* ... */ }
   ```

2. **Not Understanding Trait Bounds**
   ```rust
   // ❌ Wrong: T doesn't have method
   fn describe<T>(item: T) {
       println!("{}", item.name());  // ERROR: T doesn't have name()
   }

   // ✓ Add bound
   fn describe<T: Animal>(item: T) {
       println!("{}", item.name());  // OK
   }
   ```

3. **Performance Impact of Trait Objects**
   ```rust
   // Slower: dynamic dispatch
   fn process(item: &dyn Trait) { /* ... */ }

   // Faster: monomorphization
   fn process<T: Trait>(item: T) { /* ... */ }
   ```

### Why This Matters

Traits enable:
- **Polymorphism:** Different types, same interface
- **Flexibility:** Code works with any implementing type
- **Abstraction:** Hide implementation details
- **Composability:** Combine traits for complex behavior

For the binary protocol project:
- Implement `Display` for pretty-printing messages
- Implement `Debug` for testing and debugging
- Implement custom traits for validation

### Key Takeaways

- ✓ Traits define shared behavior (like interfaces)
- ✓ Implement traits with `impl Trait for Type`
- ✓ Use `&dyn Trait` for trait objects (dynamic dispatch)
- ✓ Use generics `<T: Trait>` for type safety (static dispatch)
- ✓ Common traits: Display, Debug, Clone, PartialEq
- ✓ Implement traits to define custom behavior

---

## Summary

This curriculum takes you from basics (primitives and functions) through intermediate concepts (ownership and borrowing) to advanced patterns (traits and custom types). Each lesson builds on previous ones, culminating in the Binary Protocol Parser project that integrates everything.

The lessons follow this progression:
1. **Foundations (Lessons 1-4):** Types, functions, structs, methods
2. **Ownership & Sharing (Lessons 5-8):** Ownership, borrowing, lifetimes, collections
3. **Expressiveness (Lessons 9-10):** Pattern matching, enums
4. **Safety (Lessons 11-12):** Option, Result, custom errors
5. **Organization (Lesson 13):** Modules and visibility
6. **Abstraction (Lesson 14):** Traits and polymorphism

Master these concepts and you'll have a deep understanding of Rust!
