# Phase 1: Rust Fundamentals - Lesson Plan

**Duration:** 3 weeks (40-50 hours)
**Target Audience:** Intermediate Python developers
**Difficulty Progression:** 🟢 Easy → 🟡 Medium → 🔴 Hard

> **📚 Detailed Lessons:** See the [`lessons/`](lessons/) directory for individual lesson files with comprehensive explanations, examples, and exercises.

---

## Week 1: Building Blocks (Foundation)

### [Lesson 1: Primitive Data Types](lessons/lesson_01_primitive_types.md) 🟢
- **Duration:** 30-40 minutes
- **Key Topics:**
  - Integers: signed (i8, i16, i32, i64, isize) and unsigned (u8, u16, u32, u64, usize)
  - Floating-point: f32, f64
  - Booleans (bool)
  - Characters (char) and string slices (&str)
  - String owned type (String)
  - Type inference and explicit type annotations
  - Number literals and underscores
- **Python Connection:** How Rust types differ from Python's dynamic typing
- **Why This Matters:** Understanding fixed types is foundational; enables memory safety and performance
- **Critical Path:** Yes - prerequisite for all following lessons
- **Detailed Lesson:** See [lesson_01_primitive_types.md](lessons/lesson_01_primitive_types.md) for comprehensive explanations, multiple code examples, and practice exercises

### [Lesson 2: Creating Functions](lessons/lesson_02_creating_functions.md) 🟢
- **Duration:** 30-40 minutes
- **Key Topics:**
  - Function syntax and naming conventions
  - Parameters and type annotations
  - Return types with `->` syntax
  - Expressions vs statements (semicolon impact)
  - Implicit returns
  - Function scope and variable shadowing
  - Documentation comments (///)
- **Python Connection:** How Rust functions differ from Python (type requirements, implicit returns)
- **Why This Matters:** Functions are the building blocks of all Rust programs
- **Critical Path:** Yes - essential for all following code
- **Detailed Lesson:** See [lesson_02_creating_functions.md](lessons/lesson_02_creating_functions.md) for detailed explanations of statements vs expressions, scope rules, and practical examples

### Lesson 3: Creating Structs 🟢
- **Duration:** 35-45 minutes
- **Key Topics:**
  - Struct definition and field organization
  - Creating struct instances
  - Accessing struct fields
  - Tuple structs
  - Unit structs
  - Struct field visibility
  - Struct update syntax
- **Python Connection:** Similarity to Python dataclasses; differences in immutability by default
- **Why This Matters:** Structs are how you group related data; essential for the binary protocol project
- **Critical Path:** Yes - used extensively in Projects
- **Practices:**
  - Defining structs for domain models
  - Creating instances with and without field init shorthand
  - Destructuring struct fields

### Lesson 4: Impl Blocks and Methods 🟡
- **Duration:** 35-45 minutes
- **Key Topics:**
  - Impl blocks and associated functions
  - Methods (functions that take &self, &mut self, or self)
  - Method call syntax and auto-dereferencing
  - Associated functions (without self)
  - Multiple impl blocks
  - Self keyword and ownership in methods
- **Python Connection:** Impl blocks as "where you put methods"; self parameter is explicit
- **Why This Matters:** Methods enable encapsulation and clean APIs
- **Critical Path:** Yes - critical for project implementation
- **Practices:**
  - Implementing constructors and builders
  - Writing methods with different receiver types
  - Understanding when to use &self vs &mut self vs self

---

## Week 1-2: Core Language Concepts (Part 1)

### Lesson 5: Ownership Fundamentals 🟡
- **Duration:** 40-50 minutes
- **Key Topics:**
  - The three ownership rules
  - Stack vs heap memory (conceptual)
  - Move semantics
  - Copying types (primitives and Copy trait)
  - Taking ownership vs borrowing
  - The borrow checker (mental model)
- **Python Connection:** Python always references; Rust can move or copy
- **Why This Matters:** Ownership is Rust's killer feature; enables memory safety without GC
- **Critical Path:** Yes - essential for all borrowing concepts
- **Practices:**
  - Understanding when values are moved
  - Identifying which types implement Copy
  - Predicting borrow checker behavior

### Lesson 6: References and Borrowing 🟡
- **Duration:** 40-50 minutes
- **Key Topics:**
  - Immutable references (&T)
  - Mutable references (&mut T)
  - Borrowing rules (many readers OR one writer)
  - Lifetime basics (implicitly, why they exist)
  - Dangling reference prevention
  - Reference counting implications
- **Python Connection:** References in Python are always possible; Rust restricts when
- **Why This Matters:** Borrowing enables sharing without ownership transfer
- **Critical Path:** Yes - required for pattern matching and error handling
- **Practices:**
  - Writing functions that borrow instead of taking ownership
  - Understanding mutable vs immutable borrows
  - Debugging borrow checker errors

### Lesson 7: Lifetimes Explained 🟡
- **Duration:** 40-50 minutes
- **Key Topics:**
  - Lifetime syntax and annotations
  - Implicit lifetime elision
  - Multiple lifetimes in functions and structs
  - 'static lifetime
  - Why lifetimes matter (dangling reference prevention)
  - Common lifetime patterns
- **Python Connection:** Python has garbage collection; Rust uses explicit lifetimes
- **Why This Matters:** Lifetimes prevent common memory bugs at compile time
- **Critical Path:** Important but Lesson 6 is more immediately critical
- **Practices:**
  - Reading lifetime signatures
  - Writing functions with explicit lifetimes
  - Understanding lifetime bounds

### Lesson 8: Smart Pointers and Collections 🟡
- **Duration:** 40-50 minutes
- **Key Topics:**
  - Vec<T> and resizing
  - Vector ownership and borrowing
  - Common vector operations
  - Iterating over vectors (ownership, references)
  - String vs &str revisited
  - Other collections (HashMap, HashSet)
- **Python Connection:** Vec similar to Python lists; ownership rules differ
- **Why This Matters:** Collections are essential for real-world programs
- **Critical Path:** Yes - used in the binary protocol project
- **Practices:**
  - Creating and manipulating vectors
  - Iterating efficiently
  - Working with collection methods

---

## Week 2-3: Core Language Concepts (Part 2)

### Lesson 9: Pattern Matching and Destructuring 🟡
- **Duration:** 40-50 minutes
- **Key Topics:**
  - Match expressions and exhaustiveness
  - Match guards
  - Destructuring in matches
  - If let and while let patterns
  - Refutable vs irrefutable patterns
  - Combining patterns (or, binding, wildcards)
- **Python Connection:** Match is new to Python 3.10+; Rust's is more powerful
- **Why This Matters:** Pattern matching is idiomatic Rust; replaces many if-else chains
- **Critical Path:** Yes - critical for error handling and parsing
- **Practices:**
  - Writing exhaustive match statements
  - Using if let for single-pattern matches
  - Destructuring complex types

### Lesson 10: Enums and Associated Data 🟡
- **Duration:** 40-50 minutes
- **Key Topics:**
  - Enum basics and variants
  - Enum variants with associated data
  - Enum methods
  - Using enums for type-safe choices
  - Difference from union types in other languages
  - Pattern matching with enums (covered in context)
- **Python Connection:** More powerful than Python enums; better for type safety
- **Why This Matters:** Enums are how Rust achieves type-safe optionality
- **Critical Path:** Yes - Option and Result are enums
- **Practices:**
  - Defining enums with associated data
  - Matching on enum variants
  - Building type-safe APIs with enums

### Lesson 11: Option and Result - Error Handling 🟡
- **Duration:** 45-55 minutes
- **Key Topics:**
  - Option<T> (Some, None)
  - Result<T, E> (Ok, Err)
  - The ? operator and early returns
  - Handling Results with match
  - Combinators (map, and_then, or_else)
  - Error propagation patterns
  - panic! vs Result
- **Python Connection:** Replaces try-except in many cases; more explicit
- **Why This Matters:** Rust's approach to errors enables reliable error handling
- **Critical Path:** Yes - essential for production code
- **Practices:**
  - Using ? for error propagation
  - Matching on Result values
  - Writing functions that return Result
  - Using Option for nullable values

### Lesson 12: Custom Error Types 🟡
- **Duration:** 40-50 minutes
- **Key Topics:**
  - Defining custom error types with enums or structs
  - Implementing std::error::Error trait
  - Implementing Display trait
  - Error context and messaging
  - Error conversion with From trait
  - Real-world error handling patterns
- **Python Connection:** Custom exceptions; Rust errors are more structured
- **Why This Matters:** Custom errors enable clear, actionable error messages
- **Critical Path:** Yes - critical for the binary protocol project
- **Practices:**
  - Defining domain-specific error enums
  - Implementing Error trait
  - Converting between error types
  - Adding error context

---

## Week 3: Advanced Concepts and Integration

### Lesson 13: Modules and Visibility 🟡
- **Duration:** 40-50 minutes
- **Key Topics:**
  - Module system and organization
  - Module files and mod.rs
  - pub keyword and visibility levels
  - pub(crate), pub(super)
  - use statements and imports
  - Re-exports
  - Workspace structure
- **Python Connection:** Similar to Python modules but with explicit visibility control
- **Why This Matters:** Modules enable code organization and encapsulation
- **Critical Path:** Important for organizing the binary protocol project
- **Practices:**
  - Organizing code into logical modules
  - Controlling visibility appropriately
  - Understanding module paths

### Lesson 14: Traits and Polymorphism 🔴
- **Duration:** 45-55 minutes
- **Key Topics:**
  - Trait definition and implementation
  - Trait bounds
  - Implementing common traits (Display, Debug, Clone, Copy, PartialEq)
  - Trait objects with dyn
  - Generics with trait bounds
  - Associated types
- **Python Connection:** Similar to protocols/interfaces; enables type-safe polymorphism
- **Why This Matters:** Traits enable flexible, reusable code
- **Critical Path:** Not immediately critical but important for refining the project
- **Practices:**
  - Implementing Display and Debug
  - Using trait bounds in generics
  - Understanding when to use trait objects vs generics

---

## Timeline Summary

| Week | Lessons | Focus | Time |
|------|---------|-------|------|
| Week 1 (Days 1-5) | 1-4 | Building Blocks (Foundation) | 2.5-3 hours/day |
| Week 1-2 (Days 6-10) | 5-8 | Ownership & Borrowing | 3-3.5 hours/day |
| Week 2-3 (Days 11-15) | 9-12 | Pattern Matching & Error Handling | 3-3.5 hours/day |
| Week 3 (Days 16-19) | 13-14 | Modules, Traits & Project Integration | 2.5-3 hours/day |
| Week 3 (Day 20) | Project | Binary Protocol Parser Implementation | 5-8 hours |

---

## Critical Concepts and Dependencies

```
Lesson 1 (Primitives)
    ↓
Lesson 2 (Functions)
    ↓
Lesson 3-4 (Structs & Methods)
    ↓
Lesson 5-8 (Ownership, References, Lifetimes, Collections)
    ↓
Lesson 9-10 (Pattern Matching & Enums)
    ↓
Lesson 11-12 (Option, Result, Error Handling)
    ↓
Lesson 13 (Modules & Organization)
    ↓
Lesson 14 (Traits - Optional for mastery)
    ↓
Project: Binary Protocol Parser
```

---

## Difficulty Progression

- **🟢 Easy (Lessons 1-2):** Focus on syntax and basic concepts
- **🟡 Medium (Lessons 3-13):** Building understanding and applying concepts
- **🔴 Hard (Lesson 14):** Abstraction and advanced patterns

---

## Prerequisites Check

Before starting Phase 1, students should:
- [ ] Complete Phase 0 (create and run Rust programs)
- [ ] Have basic Python experience
- [ ] Understand basic algorithms and data structures
- [ ] Be comfortable with the command line

---

## Practice Project Integration

The **Binary Protocol Parser** project integrates concepts from:
- Lesson 1: Data types (bytes, versions, messages)
- Lesson 2-4: Functions and methods for parsing
- Lesson 5-8: Ownership of parsed data
- Lesson 9-10: Enums for message types
- Lesson 11-12: Error handling for malformed packets
- Lesson 13: Module organization (parser logic separate from main)

Students should begin reference implementation work during Lessons 11-12 and complete during Week 3.
