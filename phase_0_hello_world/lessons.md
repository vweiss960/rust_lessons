# Phase 0: Hello World - Detailed Lessons

## Lesson 1: Installing Rust & Understanding the Toolchain

### Overview
Rust is installed through **rustup**, a version manager that downloads and manages the Rust compiler, package manager, and other tools. Unlike Python where you might install one version globally, rustup lets you manage multiple Rust versions easily.

### Installation Steps

#### macOS and Linux
Open your terminal and run:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

This downloads a script and runs it. You'll be prompted with installation options—press Enter to accept defaults.

After installation completes, activate your environment:

```bash
source $HOME/.cargo/env
```

#### Windows
Download the installer from [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install) and run it. Choose the default installation.

### Verify Installation

Check that Rust is installed correctly:

```bash
rustc --version
cargo --version
rustup --version
```

You should see version numbers like:
```
rustc 1.75.0 (1d8b05fc5 2023-12-21)
cargo 1.75.0 (ecb9851a5 2023-12-21)
rustup 1.26.0 (5af9b9adb 2023-12-18)
```

If you see version numbers, **installation succeeded!** If not, check the official [Rust installation guide](https://doc.rust-lang.org/book/ch01-01-installation.html).

### Understanding Rust Tools

**rustc** (Rust Compiler)
- The core compiler that translates Rust code to machine code
- You rarely use directly; cargo handles it

**cargo** (Package Manager & Build Tool)
- Similar to Python's pip + setuptools
- Manages dependencies, builds projects, runs tests
- Your primary tool for development

**rustup** (Version Manager)
- Manages which Rust version you're using
- Can update Rust automatically with `rustup update`
- Lets you use different versions for different projects

### Common Installation Issues

**Problem:** `rustc: command not found` after installation
- **Solution:** Close and reopen your terminal, or run `source $HOME/.cargo/env` on macOS/Linux

**Problem:** Windows PowerShell won't run installer
- **Solution:** Use Command Prompt (cmd.exe) instead

**Problem:** Network issues during installation
- **Solution:** Check your internet connection and firewall settings

### Key Takeaways
- rustup installs and manages the Rust toolchain
- rustc is the compiler, cargo is the project manager
- Verify installation with `rustc --version`
- You can update Rust anytime with `rustup update`

---

## Lesson 2: Creating Your First Project

### The `cargo new` Command

When you create a new Rust project, use:

```bash
cargo new hello_rust
cd hello_rust
```

This creates a directory with everything you need:

```
hello_rust/
├── Cargo.toml          # Project manifest
├── src/
│   └── main.rs         # Your program code
└── .gitignore          # Git ignore file (if creating git repo)
```

### Understanding Cargo.toml

The `Cargo.toml` file is your project's configuration:

```toml
[package]
name = "hello_rust"
version = "0.1.0"
edition = "2021"

[dependencies]
```

- **[package]**: Describes your project
  - `name`: Your project name (used in builds)
  - `version`: Current version (follows semantic versioning)
  - `edition`: Rust language edition (2021 is current)

- **[dependencies]**: Lists libraries your project uses
  - Empty for now—we'll add dependencies in later phases

### The src Directory

Rust code lives in `src/`. For binary programs:
- **src/main.rs** - Entry point for executables
- Optional: **src/lib.rs** - Library code (for later)

Example: If you want to share code between programs, you'd create:
```
src/
├── main.rs              # Binary executable
├── lib.rs               # Shared library code
└── utils.rs             # Utility module (lib.rs imports this)
```

### Project Naming Convention

Rust follows these conventions:
- **Project names**: lowercase with underscores (`hello_world`, `my_app`)
- **Type names**: PascalCase (`MyStruct`, `Parser`)
- **Variable/function names**: lowercase with underscores (`my_variable`, `process_data`)

Example:
```bash
cargo new my_networking_app
# Creates directory with valid Rust naming
```

### Common Beginner Mistakes

**Mistake 1: Creating projects in random places**
```bash
# ❌ WRONG - Hard to find later
cargo new /tmp/hello_rust

# ✅ RIGHT - Organized location
mkdir ~/projects && cd ~/projects
cargo new hello_rust
```

**Mistake 2: Modifying Cargo.toml incorrectly**
```toml
# ❌ WRONG - Missing quotes
name = hello_rust

# ✅ RIGHT - Strings need quotes
name = "hello_rust"
```

**Mistake 3: Not understanding project structure**
```bash
# ❌ WRONG - Editing files outside src/
echo 'fn main() {}' > hello_rust.rs

# ✅ RIGHT - Edit src/main.rs
# Edit: hello_rust/src/main.rs
```

### Key Takeaways
- `cargo new project_name` creates a full Rust project structure
- `Cargo.toml` is your project manifest file
- `src/main.rs` is where your code goes
- Follow naming conventions (snake_case for projects)
- Always work within the project directory

---

## Lesson 3: Your First Program - Hello World

### Understanding fn main()

Every Rust program needs a `main` function—it's where execution starts:

```rust
fn main() {
    println!("Hello, world!");
}
```

**Breakdown:**
- `fn`: Declares a function
- `main`: Function name (special—program starts here)
- `{}`: Function body (code inside the braces)
- `println!`: Macro that prints text with a newline

### The println! Macro

`println!` prints a line of text to the console:

```rust
fn main() {
    // Print a simple string
    println!("Hello, world!");

    // Print multiple messages
    println!("Welcome to Rust");
    println!("This is amazing!");
}
```

**Output:**
```
Hello, world!
Welcome to Rust
This is amazing!
```

Notice the `!` after `println`. That indicates it's a **macro**, not a regular function. Macros are powerful tools that generate code. For now, just remember: `println!` = "print this text."

### Running Your Code

You have three commands:

**1. `cargo run`** (Most Common)
```bash
cd hello_rust
cargo run
```
This compiles and runs your program in one step.

**Output:**
```
   Compiling hello_rust v0.1.0
    Finished dev [unoptimized + debuginfo] target/debug/hello_rust
     Running `target/debug/hello_rust`
Hello, world!
```

**2. `cargo build`** (Compile Only)
```bash
cargo build
```
Compiles your code but doesn't run it. Creates executable at `target/debug/hello_rust`.

**3. `cargo release`** (Optimized)
```bash
cargo build --release
```
Compiles with optimizations for speed. Creates executable at `target/release/hello_rust`. This is what you'd use before shipping, but it takes longer to compile.

### Modifying Your Program

Try changing the message:

```rust
fn main() {
    println!("Rust is cool!");
}
```

Then run again:
```bash
cargo run
```

**Output:**
```
   Compiling hello_rust v0.1.0
    Finished dev [unoptimized + debuginfo] target/debug/hello_rust
     Running `target/debug/hello_rust`
Rust is cool!
```

### Understanding Compiler Error Messages

Rust's compiler is very helpful. If you make a mistake, it tells you exactly what's wrong.

**Example: Missing semicolon**
```rust
fn main() {
    println!("Hello")  // Oops, missing semicolon
}
```

**Compiler output:**
```
error: expected `;`, found `}`
 --> src/main.rs:2:20
  |
2 |     println!("Hello")
  |                      ^ expected `;`
  |
help: add `;` here
  |
2 |     println!("Hello");
  |                      ^

error: aborting due to previous error
```

The compiler shows:
- **The line number** where it found the problem
- **What it expected** vs what it found
- **A helpful suggestion** to fix it

This is Rust being your friend. Read error messages carefully—they're usually right!

### Common Beginner Mistakes

**Mistake 1: Forgetting the main function**
```rust
// ❌ WRONG
println!("Hello!");
```
Result: Compiler error (no main function)

**Mistake 2: Missing semicolons**
```rust
// ❌ WRONG (usually)
fn main() {
    println!("Hello")
}
```
Result: Compiler error

**Mistake 3: Wrong brackets or syntax**
```rust
// ❌ WRONG - Square brackets
fn main [
    println!("Hello");
]
```
Result: Compiler error

### Key Takeaways
- `fn main() {}` is your program's entry point
- `println!("text")` prints output
- `cargo run` compiles and executes
- Rust's compiler error messages are helpful—read them!
- Semicolons (`;`) end statements
- Small typos are caught immediately by the compiler

---

## Lesson 4: Command-Line Arguments & Variables

### Command-Line Arguments

Programs often need input from the user. In Rust, read arguments with `std::env::args()`:

```rust
fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("{:?}", args);
}
```

Run it:
```bash
cargo run -- hello world
```

Output:
```
["target/debug/hello_rust", "hello", "world"]
```

**What's happening?**
- `std::env::args()` reads command-line arguments
- `.collect()` gathers them into a `Vec<String>` (a list of text)
- `{:?}` prints the list in debug format
- The `--` after `cargo run` tells cargo to pass the following arguments to your program

### Understanding Variables with `let`

Variables store values. Use `let` to create them:

```rust
fn main() {
    let name = "Alice";
    let age = 30;
    let score = 95.5;

    println!("Name: {}", name);
    println!("Age: {}", age);
    println!("Score: {}", score);
}
```

Output:
```
Name: Alice
Age: 30
Score: 95.5
```

**Breakdown:**
- `let name = "Alice"` creates a variable named `name`
- `{}` in println is a placeholder where the value goes
- Rust **infers the type** automatically:
  - `"Alice"` is a string
  - `30` is an integer
  - `95.5` is a floating-point number

### Working with Command-Line Arguments

Here's a practical example that accepts a name and greets the user:

```rust
fn main() {
    let args: Vec<String> = std::env::args().collect();

    // args[0] is the program name, args[1] is our first argument
    if args.len() > 1 {
        let name = &args[1];
        println!("Hello, {}!", name);
    } else {
        println!("Please provide your name!");
    }
}
```

Run it:
```bash
cargo run -- Alice
```

Output:
```
Hello, Alice!
```

Or without an argument:
```bash
cargo run
```

Output:
```
Please provide your name!
```

**Key point:** `args[1]` gets the second item (because `args[0]` is the program name).

### Looping Over Arguments

To process multiple arguments, loop through them:

```rust
fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Skip the first argument (program name)
    for arg in args.iter().skip(1) {
        println!("You said: {}", arg);
    }
}
```

Run it:
```bash
cargo run -- hello world rust
```

Output:
```
You said: hello
You said: world
You said: rust
```

**Breakdown:**
- `.iter()` creates an iterator (something you can loop over)
- `.skip(1)` skips the first item (program name)
- `for arg in ...` loops through remaining items
- Each iteration, `arg` holds one item

### Printing Multiple Values

Use `{}` for each value you want to print:

```rust
fn main() {
    let name = "Bob";
    let age = 25;

    println!("{} is {} years old", name, age);
    println!("{} says: I am {} years old!", name, age);
}
```

Output:
```
Bob is 25 years old
Bob says: I am 25 years old!
```

**Alternative: Using variable names (clearer)**
```rust
fn main() {
    let name = "Bob";
    let age = 25;

    println!("{name} is {age} years old");
}
```

Same output, but easier to read!

### Common Beginner Mistakes

**Mistake 1: Accessing an argument that doesn't exist**
```rust
fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("{}", args[5]);  // Panics if there are fewer than 6 arguments!
}
```
**Solution:** Check the length first:
```rust
if args.len() > 5 {
    println!("{}", args[5]);
}
```

**Mistake 2: Forgetting the `&` when borrowing**
```rust
let args = std::env::args().collect();
for arg in args {  // Moves ownership, can't use args later
    println!("{}", arg);
}
println!("{}", args);  // ERROR: args was moved
```
**Solution:** Use `.iter()`:
```rust
for arg in args.iter() {  // Borrows, args still available
    println!("{}", arg);
}
println!("{:?}", args);  // OK!
```

**Mistake 3: String vs &str confusion (advanced)**
```rust
// Don't worry about this yet—just know:
// "text" in quotes is &str (string slice)
// String (capitalized) is an owned string
// For now, they work similarly
```

### Key Takeaways
- `std::env::args()` reads command-line arguments
- `let name = value` creates variables
- Variables hold values of different types (numbers, text, etc.)
- Rust infers types automatically
- `{}` in println is a placeholder for values
- Use `.iter()` to loop over collections safely
- Check argument length before accessing by index

---

## Summary & Next Steps

You've learned:
1. ✅ How to install Rust and verify it works
2. ✅ How to create projects with `cargo new`
3. ✅ How to write and run your first program
4. ✅ How to accept command-line arguments
5. ✅ How to use variables and print values

**Congratulations! You're ready for Phase 1: Rust Fundamentals.**

In Phase 1, you'll:
- Learn ownership and borrowing (core Rust concepts)
- Work with structs and enums
- Handle errors gracefully
- Build a real parser project

Ready to dive deeper? Move on to Phase 1!
