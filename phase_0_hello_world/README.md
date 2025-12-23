# Phase 0: Hello World - Getting Started with Rust

Welcome! This is your first step into Rust programming. By the end of this phase, you'll have Rust installed and will have written and run your first program.

## What You'll Learn

- ✅ Installing Rust and verifying your setup
- ✅ Creating a new Rust project with `cargo new`
- ✅ Writing your first program: Hello, World!
- ✅ Understanding basic Rust syntax (fn, println!, main)
- ✅ Reading and processing command-line arguments
- ✅ Using variables with `let`

## Phase Overview

- **Duration:** 1 day (4-6 hours)
- **Difficulty:** 🟢 Easy (no prior Rust knowledge needed)
- **Goal:** Get comfortable with Rust basics and the cargo workflow

## Getting Started

### Step 1: Review the Lessons

Start with **lesson_plan.md** to understand the structure, then read **lessons.md** for detailed explanations with examples.

### Step 2: Install Rust

If you haven't already, install Rust:

**macOS/Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**Windows:**
- Download from https://www.rust-lang.org/tools/install
- Run the installer and follow prompts

**Verify installation:**
```bash
rustc --version
cargo --version
```

### Step 3: Work on the Project

This phase includes a starter project and a complete solution.

#### Option A: Challenge Yourself (Recommended)
1. Copy the `starter_code/` directory to a working location
2. Read the comments in `starter_code/src/main.rs` for guidance
3. Implement the code yourself
4. Run with: `cargo run -- arg1 arg2`
5. When stuck, check `solution/src/main.rs`

#### Option B: Learn by Example
1. Review `solution/src/main.rs` to see the complete implementation
2. Understand each part by reading the comments
3. Modify the code and experiment
4. Run with: `cargo run -- your arguments here`

### Step 4: Experiment

Modify the program and see what happens:

**Try these experiments:**
```bash
# Run with no arguments
cargo run

# Run with one argument
cargo run -- Alice

# Run with multiple arguments
cargo run -- hello world rust

# What happens if you use quotes?
cargo run -- "hello world"

# What if you break something? Try these intentional errors:
# 1. Remove a semicolon and see the compiler error
# 2. Remove the closing } and see what happens
# 3. Delete the println! and run again
```

## Understanding the Project Structure

```
phase_0_hello_world/
├── lesson_plan.md          # Overview of all lessons
├── lessons.md              # Detailed lessons with examples
├── README.md               # This file
├── starter_code/           # Incomplete code for you to finish
│   ├── Cargo.toml
│   └── src/main.rs
└── solution/               # Complete working solution
    ├── Cargo.toml
    └── src/main.rs
```

## Key Concepts Covered

### 1. The `fn main()` Function
Every Rust program needs a `main` function. Execution always starts here.

```rust
fn main() {
    // Code here runs when you execute the program
}
```

### 2. The `println!` Macro
Prints text to the console:

```rust
println!("Hello, world!");
println!("My age is {}", age);
```

### 3. Variables with `let`
Bind a name to a value. Rust automatically infers the type.

```rust
let name = "Alice";         // &str (string)
let age = 30;               // i32 (integer)
let score = 95.5;           // f64 (floating point)
```

### 4. Command-Line Arguments
Read what the user typed when running your program:

```rust
let args: Vec<String> = std::env::args().collect();
// args[0] is the program name
// args[1] is the first user-provided argument
// etc.
```

### 5. Looping with `for`
Iterate over a collection:

```rust
for arg in args.iter().skip(1) {
    println!("You said: {}", arg);
}
```

## How to Use Cargo

Cargo is your project manager and build tool:

| Command | What it does |
|---------|-------------|
| `cargo new project_name` | Create a new project |
| `cargo run` | Compile and run your program |
| `cargo build` | Compile without running |
| `cargo build --release` | Compile with optimizations |
| `cargo check` | Check for errors without building |
| `cargo clean` | Delete compiled artifacts |

## Running the Program

Once you're in the project directory:

```bash
# No arguments - program handles this gracefully
cargo run

# One argument
cargo run -- Alice

# Multiple arguments (note the -- separating cargo args from program args)
cargo run -- hello world rust

# Save the compiled binary and run it directly
cargo build
./target/debug/phase_0_hello_world arg1 arg2
```

## Common Beginner Mistakes

### 1. Forgetting the `--` when passing arguments to cargo
```bash
# ❌ WRONG - cargo tries to interpret "Alice" as a cargo argument
cargo run Alice

# ✅ RIGHT - the -- tells cargo these args go to the program
cargo run -- Alice
```

### 2. Not reading compiler error messages
The Rust compiler is your friend! It gives very specific, helpful error messages. Read them carefully—they tell you exactly what's wrong and how to fix it.

### 3. Trying to access arguments that might not exist
```rust
// ❌ RISKY - what if there are fewer than 3 arguments?
println!("{}", args[2]);

// ✅ SAFE - check the length first
if args.len() > 2 {
    println!("{}", args[2]);
}
```

## Troubleshooting

### "rustc: command not found"
You installed Rust but your terminal doesn't see it.
- **Solution:** Close and reopen your terminal, or run `source $HOME/.cargo/env`

### "error: expected `;`"
You forgot a semicolon (semicolons end statements in Rust).
- **Solution:** Add `;` at the end of the line indicated by the error

### "could not compile `phase_0_hello_world`"
Something is wrong with your code syntax.
- **Solution:** Read the compiler error message carefully. It tells you the exact line and problem.

### "No such file or directory"
You're not in the right directory.
- **Solution:** Make sure you're inside the project directory before running `cargo run`

## Next Steps

Once you've completed this phase:

1. ✅ You have Rust installed and working
2. ✅ You understand the basic structure of a Rust program
3. ✅ You can compile and run code with `cargo`
4. ✅ You understand variables and command-line arguments

You're ready for **Phase 1: Rust Fundamentals**, where you'll learn:
- Ownership and borrowing (core Rust concept)
- Structs and enums
- Pattern matching
- Error handling
- Building a binary protocol parser

## Resources

- **The Rust Book:** https://doc.rust-lang.org/book/
- **Rust by Example:** https://doc.rust-lang.org/rust-by-example/
- **Rust Playground:** https://play.rust-lang.org/ (write and run code online)
- **Ask for help:** https://users.rust-lang.org/ (Rust forum)

## Summary

Phase 0 sets up your Rust environment and gets you comfortable with the basics. It's short but important—don't rush it. Make sure you understand:

- How to create projects with `cargo new`
- How to run programs with `cargo run`
- Basic syntax: `fn main() {}`, `println!()`, `let`
- How to read command-line arguments

Once this feels natural, move on to Phase 1 where things get more interesting!

---

**Happy coding! 🦀**

If you get stuck, read the lessons carefully, look at the solution code, and don't hesitate to experiment. The best way to learn Rust is by doing.
