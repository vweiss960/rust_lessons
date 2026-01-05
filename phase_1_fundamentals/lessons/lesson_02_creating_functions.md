# Lesson 2: Creating Functions 🟢

**Duration:** 30-40 minutes
**Difficulty:** 🟢 Easy
**Prerequisites:** Lesson 1 (Primitive Types)

---

## Learning Objectives

By the end of this lesson, you will:
- Understand Rust's function syntax and requirements
- Know the difference between statements and expressions
- Write functions with proper parameter and return types
- Use the function scope correctly
- Understand variable shadowing

---

## Concept: Functions as Building Blocks

### What are Functions?

Functions are reusable blocks of code that perform specific tasks. Every Rust program starts with a `main()` function.

**Why Functions?**
1. **Reusability** - write once, use many times
2. **Organization** - break large problems into smaller pieces
3. **Testing** - easier to test small functions
4. **Clarity** - well-named functions document intent
5. **Maintenance** - easier to fix bugs in one place

### Python vs Rust Functions

**Python - Dynamic Types:**
```python
def add(x, y):
    return x + y

result = add(5, 3)
result = add("hello", " world")  # Also works!
```

The function works with any type that supports `+`. Type flexibility, but errors can happen at runtime.

**Rust - Static Types:**
```rust
fn add(x: i32, y: i32) -> i32 {
    x + y
}

let result = add(5, 3);          // Works: 8
// let result = add("hello", " world");  // ERROR at compile time!
```

Every parameter and return type must be explicit. Type safety prevents errors.

---

## Concept: Statements vs Expressions

This is **crucial** for understanding Rust function returns.

### Statements

**Definition:** Instructions that **don't return a value**

**Characteristics:**
- End with a semicolon (;)
- Don't return anything
- Examples: variable declarations, assignments

```rust
let x = 5;          // Statement: declare and assign
let y = (let z = 6);  // ERROR: let statements don't return values!
```

### Expressions

**Definition:** Evaluate to a **resulting value that can be used**

**Characteristics:**
- **No semicolon** (semicolon turns it into a statement)
- Return a value
- Result can be assigned or used

```rust
5 + 6                // Expression: evaluates to 11
{
    let x = 3;       // Statement inside block
    x + 1            // Expression: evaluates to 4
}

let y = {
    let x = 3;
    x + 1            // No semicolon! Returns 4
};
// y is now 4
```

### The Critical Difference: Semicolon

**Without semicolon = Expression (returns value):**
```rust
{
    let x = 5;
    x + 1  // Expression: returns 6
}
// Block evaluates to 6
```

**With semicolon = Statement (returns nothing, unit type ()):**
```rust
{
    let x = 5;
    x + 1;  // Statement: returns () (nothing)
}
// Block evaluates to ()
```

This seems small, but it's **the most common function bug** for new Rustaceans.

---

## Function Syntax: The Basics

### Function Declaration

```rust
fn function_name(param1: Type1, param2: Type2) -> ReturnType {
    // function body
    return_value  // Implicit return (expression, no semicolon)
}
```

### Breaking Down the Syntax

```rust
fn         add        (x: i32, y: i32)  ->  i32   {
│          │          │                  │    │
keyword   name     parameters       return   body
                                   type
```

**fn** - keyword to declare a function
**add** - function name (snake_case by convention)
**(x: i32, y: i32)** - parameters with explicit types
**-> i32** - return type annotation
**{ ... }** - function body

### Parameter Rules

1. **Every parameter needs a type annotation**
   ```rust
   fn add(x: i32, y: i32) -> i32 { }  // ✓ Correct
   fn add(x, y) -> i32 { }            // ❌ ERROR: no types
   ```

2. **Multiple parameters are comma-separated**
   ```rust
   fn greet(name: &str, age: u8, city: &str) -> String {
       format!("{} is {} and lives in {}", name, age, city)
   }
   ```

3. **Type annotation is needed for inference context**
   ```rust
   fn describe(items: Vec<i32>) -> usize {
       items.len()  // Compiler knows items is Vec<i32>
   }
   ```

### Return Type Annotation

If a function returns a value, you must specify the type:

```rust
fn get_number() -> i32 {
    42
}

fn get_greeting() -> String {
    String::from("Hello!")
}

fn is_positive(n: i32) -> bool {
    n > 0
}
```

### Functions Returning Nothing

If a function doesn't return a value, **omit the return type** (implicitly returns `()`):

```rust
fn say_hello() {
    println!("Hello!");
}
// Implicitly returns ()

// Explicitly returning () (rarely done):
fn say_goodbye() -> () {
    println!("Goodbye!");
}
```

The `-> ()` is valid but usually omitted.

---

## Implicit vs Explicit Returns

### Implicit Return (Recommended)

Return the last expression **without a semicolon**:

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b  // No semicolon = returns value
}

fn double(x: i32) -> i32 {
    x * 2  // Implicit return
}

fn get_max(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }  // Expression returns value
}
```

**Why this style?**
- More idiomatic Rust
- Cleaner, less verbose
- Makes it obvious this is the return value
- Functions are expressions in Rust

### Explicit Return (Also Valid)

Use the `return` keyword with a semicolon:

```rust
fn add(a: i32, b: i32) -> i32 {
    return a + b;  // Explicit return
}

fn double(x: i32) -> i32 {
    return x * 2;  // Explicit return
}
```

**Both work**, but implicit returns are more idiomatic.

### Early Returns

Use `return` to exit early:

```rust
fn process(x: i32) -> &'static str {
    if x < 0 {
        return "negative";  // Return early
    }

    if x == 0 {
        return "zero";      // Return early
    }

    "positive"              // Implicit return at end
}
```

---

## The Semicolon Bug (Most Common Mistake)

### The Problem

```rust
fn get_number() -> i32 {
    42;  // ❌ Semicolon makes this a statement!
}

// Compiler error:
// expected i32, found ()
// the function is supposed to return an i32, but it's returning ()
```

**Why?** The semicolon turns the expression `42` into a statement that returns nothing (`()`).

### The Solution

```rust
fn get_number() -> i32 {
    42   // ✓ No semicolon = expression returns 42
}
```

### Fixing the Error

If you see "expected Type, found ()", always check for a stray semicolon:

```rust
fn broken() -> String {
    String::from("hello");  // ❌ Semicolon!
    // Returns ()
}

fn fixed() -> String {
    String::from("hello")   // ✓ No semicolon
    // Returns String
}
```

---

## Function Scope and Variable Shadowing

### Scope: Where Variables Live

A variable only exists within its **scope** (the block where it's declared):

```rust
fn scope_example() {
    let x = 5;
    println!("{}", x);  // ✓ x exists here

    {
        let y = 10;
        println!("{}", x);  // ✓ x still exists
        println!("{}", y);  // ✓ y exists here
    }

    println!("{}", x);  // ✓ x still exists
    // println!("{}", y);  // ❌ ERROR: y doesn't exist (out of scope)
}
```

### Shadowing: Reusing Variable Names

You can declare a new variable with the same name in an inner scope:

```rust
fn shadowing_example() {
    let x = 5;
    println!("{}", x);  // 5

    {
        let x = 10;     // Shadows outer x
        println!("{}", x);  // 10 (inner x)
    }

    println!("{}", x);  // 5 (outer x again)
}
```

### Shadowing for Type Changes

Shadowing is often used to transform values:

```rust
let input = "42";                    // &str
println!("{:?}", input);             // "42"

let input = input.trim();            // Still &str, whitespace removed
println!("{:?}", input);             // "42"

let input: i32 = input.parse()       // Parse to i32
    .expect("Not a number");
println!("{}", input);               // 42 (integer)

let input = input * 2;               // Multiply
println!("{}", input);               // 84
```

This pattern is idiomatic—reusing the name for successive transformations.

### Shadowing vs Mutation

**Different approaches, different styles:**

**Mutation (modify in place):**
```rust
let mut x = 5;
x = 10;
println!("{}", x);  // 10
```

**Shadowing (create new binding):**
```rust
let x = 5;
let x = 10;  // New binding, old x no longer accessible
println!("{}", x);  // 10
```

Both are valid; use based on intent:
- **Shadowing:** Better for transforms (type conversions, progressive refinement)
- **Mutation:** Better for in-place changes

---

## Parameters and Arguments

### Parameters vs Arguments

```rust
fn add(x: i32, y: i32) -> i32 {
    //  ↑              ↑  parameters
    x + y
}

let result = add(5, 3);
    //           ↑  ↑  arguments
```

**Parameters:** The types/names in the function definition
**Arguments:** The actual values passed when calling the function

### Parameter Patterns

**Single parameter:**
```rust
fn greet(name: &str) {
    println!("Hello, {}!", name);
}

greet("Alice");
```

**Multiple parameters:**
```rust
fn add_and_multiply(x: i32, y: i32, multiplier: i32) -> i32 {
    (x + y) * multiplier
}

let result = add_and_multiply(2, 3, 4);  // (2+3)*4 = 20
```

**No parameters:**
```rust
fn get_pi() -> f64 {
    3.14159
}

let pi = get_pi();
```

---

## Code Examples

### Example 1: Basic Functions

```rust
// Function with parameters and return type
fn multiply(x: i32, y: i32) -> i32 {
    x * y
}

// Function with no return value
fn print_twice(message: &str) {
    println!("{}", message);
    println!("{}", message);
}

// Function that returns based on condition
fn absolute_value(n: i32) -> i32 {
    if n >= 0 { n } else { -n }
}

fn main() {
    let product = multiply(5, 7);
    println!("5 * 7 = {}", product);

    print_twice("Rust is awesome!");

    println!("|-42| = {}", absolute_value(-42));
}
```

Output:
```
5 * 7 = 35
Rust is awesome!
Rust is awesome!
|-42| = 42
```

### Example 2: Using Expressions

```rust
fn calculate_discount(price: f64, discount_percent: f64) -> f64 {
    // Expression: calculates the discounted price
    price * (1.0 - discount_percent / 100.0)
}

fn is_even(n: i32) -> bool {
    // Expression: returns true if even
    n % 2 == 0
}

fn largest(a: i32, b: i32) -> i32 {
    // Expression: conditional expression
    if a > b { a } else { b }
}

fn main() {
    println!("Price after 20% off: ${}", calculate_discount(100.0, 20.0));
    println!("Is 4 even? {}", is_even(4));
    println!("Larger of 10 and 15: {}", largest(10, 15));
}
```

Output:
```
Price after 20% off: $80
Is 4 even? true
Larger of 10 and 15: 15
```

### Example 3: Shadowing and Scope

```rust
fn process_input(input: &str) -> i32 {
    // Parse string to integer
    let input = input.trim();           // Remove whitespace
    let input = input.parse::<i32>()    // Parse to i32
        .expect("Not a valid number");

    // Validate and transform
    let input = if input < 0 {
        0  // Minimum is 0
    } else if input > 100 {
        100  // Maximum is 100
    } else {
        input
    };

    input
}

fn main() {
    let result = process_input("  42  ");
    println!("Result: {}", result);
}
```

Output:
```
Result: 42
```

### Example 4: Complex Example with Multiple Functions

```rust
fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

fn is_freezing(celsius: f64) -> bool {
    celsius < 0.0
}

fn describe_temperature(celsius: f64) {
    let fahrenheit = celsius_to_fahrenheit(celsius);

    println!("Temperature: {}°C = {}°F", celsius, fahrenheit);

    if is_freezing(celsius) {
        println!("It's freezing!");
    } else if celsius < 15.0 {
        println!("It's cold.");
    } else if celsius < 25.0 {
        println!("It's moderate.");
    } else {
        println!("It's hot!");
    }
}

fn main() {
    describe_temperature(-5.0);
    describe_temperature(20.0);
    describe_temperature(35.0);
}
```

Output:
```
Temperature: -5°C = 23°F
It's freezing!
Temperature: 20°C = 68°F
It's moderate.
Temperature: 35°C = 95°F
It's hot!
```

---

## Common Mistakes

### Mistake 1: Forgetting Parameter Types

**❌ Wrong:**
```rust
fn add(x, y) {  // ERROR: types missing
    x + y
}
```

**✓ Correct:**
```rust
fn add(x: i32, y: i32) -> i32 {
    x + y
}
```

### Mistake 2: Semicolon in Return Expression

**❌ Wrong:**
```rust
fn get_value() -> i32 {
    42;  // Semicolon changes expression to statement!
}
// ERROR: expected i32, found ()
```

**✓ Correct:**
```rust
fn get_value() -> i32 {
    42   // No semicolon = returns 42
}
```

### Mistake 3: Forgetting Return Type

**❌ Wrong:**
```rust
fn add(x: i32, y: i32) {  // Missing return type annotation
    x + y
}
```

**✓ Correct:**
```rust
fn add(x: i32, y: i32) -> i32 {
    x + y
}
```

### Mistake 4: Using Variables Out of Scope

**❌ Wrong:**
```rust
fn example() {
    {
        let x = 5;
    }
    println!("{}", x);  // ERROR: x is out of scope
}
```

**✓ Correct:**
```rust
fn example() {
    let x = 5;
    {
        // x is still available here
        println!("{}", x);
    }
    println!("{}", x);  // x is still available
}
```

### Mistake 5: Not Understanding Expression Returns

**❌ Wrong:**
```rust
fn process(n: i32) -> i32 {
    if n > 10 { n * 2 }
    // Missing: what if n <= 10?
}
// ERROR: not all code paths return a value
```

**✓ Correct:**
```rust
fn process(n: i32) -> i32 {
    if n > 10 {
        n * 2
    } else {
        n  // All paths must return a value
    }
}
```

---

## Why This Matters

Understanding functions is critical because:

1. **Organization:** Functions break problems into manageable pieces
2. **Type Safety:** Explicit types prevent errors at compile time
3. **Reusability:** Write once, use many times
4. **Testing:** Functions are easier to test in isolation
5. **Clarity:** Good function names document intent

For the binary protocol project:
- You'll write functions to parse bytes
- Each function has clear input/output types
- Type system ensures correctness automatically

---

## Practice Exercises

### Exercise 1: Basic Functions

```rust
// Write a function that converts miles to kilometers
fn miles_to_kilometers(miles: f64) -> f64 {
    // 1 mile = 1.60934 km
    todo!()
}

// Write a function that checks if a year is a leap year
fn is_leap_year(year: u32) -> bool {
    // Rules: divisible by 4, except centuries unless divisible by 400
    todo!()
}

fn main() {
    println!("10 miles = {} km", miles_to_kilometers(10.0));
    println!("2024 is leap year? {}", is_leap_year(2024));
}
```

### Exercise 2: Expression vs Statement

```rust
fn main() {
    // What do these return?
    let x = {
        let y = 5;
        y + 1  // Expression or statement?
    };
    println!("{}", x);

    let z = {
        let y = 5;
        y + 1;  // What about with semicolon?
    };
    println!("{:?}", z);  // This will show something unexpected!
}
```

### Exercise 3: Scope and Shadowing

```rust
fn main() {
    let x = 10;
    println!("x = {}", x);

    {
        let x = 20;  // Shadows outer x
        println!("x = {}", x);
    }

    println!("x = {}", x);  // Which x?

    let x = x * 2;  // Shadow and transform
    println!("x = {}", x);
}
```

---

## Key Takeaways

✓ **Functions require explicit types** for all parameters and return values
✓ **Statements end with ;** and don't return values
✓ **Expressions have no ;** and return values
✓ **The semicolon matters!** It determines if value is returned
✓ **Implicit return** (no semicolon) is idiomatic Rust
✓ **Scope** determines where variables are visible
✓ **Shadowing** allows reusing names for transformations
✓ **Every code path must return** a value of the declared type

---

## Next Steps

Now that you understand functions, you're ready for:
- **Lesson 3:** Creating Structs - grouping related data
- **Lesson 4:** Methods (impl blocks) - attaching functions to types

**Quiz Yourself:**
- What's the difference between a parameter and an argument?
- Why do you need to omit the semicolon in return expressions?
- What does "scope" mean?
- When would you use shadowing instead of mutation?
