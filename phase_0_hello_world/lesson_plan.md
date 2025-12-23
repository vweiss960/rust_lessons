# Phase 0: Hello World - Lesson Plan

## Overview
- **Total Duration:** 1 day (4-6 hours)
- **Target Audience:** Intermediate Python developers
- **Goal:** Get Rust installed, run your first program, and understand basic syntax

## Lessons

### Lesson 1: Installing Rust & Verifying Setup (20 minutes)
**Topics:**
- Download and install Rust using rustup
- Verify installation (rustc, cargo)
- Understanding Rust toolchain

**Learning Outcomes:**
- Rust is installed and working
- Understanding basic Rust tools (rustc, cargo)
- Able to check Rust version

**Key Concepts:**
- rustup: Rust installer and version manager
- rustc: Rust compiler
- cargo: Rust package manager and build tool

---

### Lesson 2: Creating Your First Project (15 minutes)
**Topics:**
- Using `cargo new` to create projects
- Understanding project structure (Cargo.toml, src/main.rs)
- Project organization

**Learning Outcomes:**
- Can create a new Rust project with `cargo new`
- Understand directory structure
- Know difference between Cargo.toml and src/main.rs

**Key Concepts:**
- Cargo: project management tool
- Cargo.toml: project manifest (name, version, dependencies)
- src/main.rs: entry point for executable

---

### Lesson 3: Your First Program - Hello World (20 minutes)
**Topics:**
- fn main() function
- println! macro
- Running code with `cargo run`
- Understanding output

**Learning Outcomes:**
- Can write and run a simple Rust program
- Understand println! macro
- Know difference between `cargo build` and `cargo run`

**Key Concepts:**
- fn main() is the program entry point
- println! is a macro (note the !) that prints to console
- cargo run compiles and executes in one step
- Rust uses semicolons to end statements

---

### Lesson 4: Accepting Arguments & Basic Variables (25 minutes)
**Topics:**
- Command-line arguments with std::env::args()
- Variables and let bindings
- String types and ownership intro
- Iterating over collections
- Basic control flow

**Learning Outcomes:**
- Can read command-line arguments
- Understand variable binding with `let`
- Know how to iterate over collections
- Understand basic control flow

**Key Concepts:**
- std::env::args() returns command-line arguments
- `let` binds a value to a variable name
- String vs &str (brief intro)
- Iteration patterns
- println! with multiple values

---

## Total Time Estimate
- Lesson 1: 20 minutes
- Lesson 2: 15 minutes
- Lesson 3: 20 minutes
- Lesson 4: 25 minutes
- **Buffer time & experimentation:** 1-2 hours
- **Total:** 4-6 hours for day one

## Success Criteria
By end of Phase 0, you should be able to:
- ✅ Install Rust successfully
- ✅ Create a new Rust project with cargo
- ✅ Write and run a simple program
- ✅ Modify Rust code and see results
- ✅ Pass command-line arguments to your program
- ✅ Print variables and values to console
- ✅ Understand error messages from the Rust compiler

---

**Next:** Phase 1 - Rust Fundamentals (after Phase 0 is complete)
