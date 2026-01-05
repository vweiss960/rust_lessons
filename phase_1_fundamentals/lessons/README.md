# Phase 1: Rust Fundamentals - Individual Lesson Files

This directory contains **detailed, concept-focused lesson files** for Phase 1 of the Rust curriculum.

## Structure

Each lesson file is **standalone** and includes:
- **Detailed concept explanations** with theory and intuition
- **Why it matters** - practical relevance
- **Multiple code examples** (beginner to intermediate)
- **Common mistakes** specific to Python developers
- **Practice exercises**
- **Key takeaways**

## Lessons by Week

### Week 1: Building Blocks (Foundation)

#### [Lesson 1: Primitive Data Types](lesson_01_primitive_types.md) 🟢
- **Duration:** 30-40 minutes
- **Topics:**
  - Type system and why Rust is statically typed
  - Stack vs Heap memory (conceptual understanding)
  - Integers (signed/unsigned), floats, booleans, chars
  - String types: `&str` vs `String`
  - Collections: `Vec<T>`
  - Type inference
- **Key Concepts:** Fixed types, stack allocation, memory safety
- **For Binary Protocol Project:** Understanding bytes, types for protocol fields

#### [Lesson 2: Creating Functions](lesson_02_creating_functions.md) 🟢
- **Duration:** 30-40 minutes
- **Topics:**
  - Function syntax and requirements
  - Statements vs Expressions (critical!)
  - Parameter and return types
  - Implicit vs explicit returns
  - Function scope
  - Variable shadowing
- **Key Concepts:** Type-safe functions, expression returns, scope
- **For Binary Protocol Project:** Writing parsers and validators

#### Lesson 3: Creating Structs 🟢
- **Duration:** 35-45 minutes
- **Topics:**
  - Struct definition and instantiation
  - Named and tuple structs
  - Field access and mutability
  - Struct update syntax
  - Visibility of fields
- **Key Concepts:** Custom types, grouping data
- **For Binary Protocol Project:** Representing messages, headers, errors

#### Lesson 4: Impl Blocks and Methods 🟡
- **Duration:** 35-45 minutes
- **Topics:**
  - Impl blocks and associated functions
  - Methods: `&self`, `&mut self`, `self`
  - Method chaining and builder pattern
  - Multiple impl blocks
- **Key Concepts:** Encapsulation, methods, ownership in methods
- **For Binary Protocol Project:** Parsing and validation methods

### Week 1-2: Core Language Concepts (Part 1)

#### Lesson 5: Ownership Fundamentals 🟡
- **Duration:** 40-50 minutes
- **Topics:**
  - The three ownership rules
  - Move semantics
  - Copy vs Clone
  - Stack vs Heap implications
  - Why ownership prevents bugs
- **Key Concepts:** Memory safety, move semantics, preventing double-free
- **For Binary Protocol Project:** Managing parsed message ownership

#### Lesson 6: References and Borrowing 🟡
- **Duration:** 40-50 minutes
- **Topics:**
  - Immutable references (`&T`)
  - Mutable references (`&mut T`)
  - The borrow checker rules
  - Dangling reference prevention
  - Borrowing in functions
- **Key Concepts:** Sharing without ownership, borrow rules
- **For Binary Protocol Project:** Functions that borrow data

#### Lesson 7: Lifetimes Explained 🟡
- **Duration:** 40-50 minutes
- **Topics:**
  - Lifetime syntax and annotations
  - Lifetime elision
  - Multiple lifetimes
  - Why lifetimes matter
  - `'static` lifetime
- **Key Concepts:** Reference validity, preventing use-after-free
- **For Binary Protocol Project:** Parsing with references

#### Lesson 8: Smart Pointers and Collections 🟡
- **Duration:** 40-50 minutes
- **Topics:**
  - `Vec<T>` and vectors
  - Vector operations and ownership
  - Iterating over vectors
  - Slices (`&[T]`)
  - HashMap and other collections
  - Collections in functions
- **Key Concepts:** Dynamic arrays, zero-copy borrowing
- **For Binary Protocol Project:** Working with byte vectors

### Week 2-3: Core Language Concepts (Part 2)

#### Lesson 9: Pattern Matching and Destructuring 🟡
- **Duration:** 40-50 minutes
- **Topics:**
  - Match expressions and exhaustiveness
  - Destructuring in matches
  - If let and while let
  - Match guards
  - Combining patterns
- **Key Concepts:** Exhaustive checking, safe extraction
- **For Binary Protocol Project:** Handling different message types

#### Lesson 10: Enums and Associated Data 🟡
- **Duration:** 40-50 minutes
- **Topics:**
  - Enum definition and variants
  - Enums with associated data
  - Pattern matching with enums
  - Methods on enums
  - Option and Result
- **Key Concepts:** Type-safe choices, algebraic data types
- **For Binary Protocol Project:** Message types, error handling

#### Lesson 11: Option and Result - Error Handling 🟡
- **Duration:** 45-55 minutes
- **Topics:**
  - `Option<T>` (Some, None)
  - `Result<T, E>` (Ok, Err)
  - The `?` operator
  - Error propagation
  - Combinators (map, and_then)
  - Panic vs Result
- **Key Concepts:** Explicit error handling, no silent failures
- **For Binary Protocol Project:** Handling parse failures gracefully

#### Lesson 12: Custom Error Types 🟡
- **Duration:** 40-50 minutes
- **Topics:**
  - Defining custom error enums
  - Implementing Error trait
  - Display trait for messages
  - Error context and fields
  - From trait for conversions
- **Key Concepts:** Domain-specific errors, actionable messages
- **For Binary Protocol Project:** ParseError with rich context

### Week 3: Advanced Concepts and Integration

#### Lesson 13: Modules and Visibility 🟡
- **Duration:** 40-50 minutes
- **Topics:**
  - Module system organization
  - Visibility: pub, private, pub(crate)
  - Use statements and imports
  - Re-exports with pub use
  - File organization
- **Key Concepts:** Code organization, encapsulation
- **For Binary Protocol Project:** Organizing parser, validator, error modules

#### Lesson 14: Traits and Polymorphism 🔴
- **Duration:** 45-55 minutes
- **Topics:**
  - Trait definition and implementation
  - Trait bounds
  - Implementing Display and Debug
  - Trait objects with dyn
  - Generics with trait bounds
- **Key Concepts:** Flexible abstractions, polymorphism
- **For Binary Protocol Project:** Implementing Display and Error traits

---

## How to Use These Lessons

### For Students

1. **Read in order** - Lessons build on each other
2. **Follow the examples** - Copy, run, modify
3. **Do the practice exercises** - Learning by doing
4. **Refer back** - Use as reference while coding

### For Instructors

1. **Use the structure** - Detailed explanations, examples, exercises
2. **Adapt timing** - Suggested durations, adjust for your class
3. **Supplement with exercises** - Each lesson has practice problems
4. **Reference the concepts** - Each lesson explains why it matters

---

## Key Features of These Lessons

### 1. Concept-Focused Explanations
Each lesson explains **why** concepts exist, not just **what** they are:
- Stack vs Heap - why it matters for Rust design
- Ownership - why it prevents bugs
- Borrowing - why it's safe and how it works
- Type safety - why static types are valuable

### 2. Python Comparisons
For intermediate Python developers:
- How Rust differs from Python
- What becomes explicit that was implicit
- Why the changes matter

### 3. Binary Protocol Project Integration
Every lesson shows relevance to the project:
- Lesson 1: Types for protocol fields
- Lesson 2: Functions for parsing
- Lesson 11: Result for error handling
- Lesson 12: ParseError enum
- And so on...

### 4. Multiple Examples
Each concept has:
- Simple examples (introductory)
- Practical examples (real-world scenarios)
- Complex examples (putting it together)

### 5. Common Mistakes Section
Specifically addresses issues Python developers encounter:
- Type confusion (char vs str)
- Ownership surprise
- Borrow checker errors
- Other common pitfalls

---

## Learning Path

### Recommended Schedule

**Week 1 (Days 1-5):**
- Lesson 1-4: Building Blocks
- 2.5-3 hours per day
- Focus: Understanding types, functions, structs

**Week 1-2 (Days 6-10):**
- Lesson 5-8: Ownership & Borrowing
- 3-3.5 hours per day
- Focus: Memory safety, references

**Week 2-3 (Days 11-15):**
- Lesson 9-12: Pattern Matching & Error Handling
- 3-3.5 hours per day
- Focus: Type safety, error handling

**Week 3 (Days 16-19):**
- Lesson 13-14: Modules & Traits
- 2.5-3 hours per day
- Focus: Organization, abstraction

**Week 3 (Day 20):**
- Binary Protocol Parser Project
- 5-8 hours
- Integration of all concepts

---

## Quick Reference

### Difficulty Distribution

| Difficulty | Count | Lessons |
|-----------|-------|---------|
| 🟢 Easy | 4 | 1-4 |
| 🟡 Medium | 9 | 5-13 |
| 🔴 Hard | 1 | 14 |

### Topics by Category

**Fundamentals:**
- Lesson 1: Types
- Lesson 2: Functions
- Lesson 3: Structs
- Lesson 4: Methods

**Safety & Correctness:**
- Lesson 5: Ownership
- Lesson 6: Borrowing
- Lesson 7: Lifetimes
- Lesson 11: Error Handling
- Lesson 12: Custom Errors

**Language Features:**
- Lesson 8: Collections
- Lesson 9: Pattern Matching
- Lesson 10: Enums
- Lesson 13: Modules
- Lesson 14: Traits

---

## Companion Resources

These lessons are paired with:
- **lesson_plan.md** - Overview and timeline
- **lessons.md** - Original comprehensive guide (combined version)
- **starter_code/** - Incomplete binary protocol parser (student assignment)
- **solution/** - Complete, tested implementation (reference)

---

## Tips for Success

### For Concept Mastery

1. **Understand the WHY**
   - Don't just memorize syntax
   - Understand why Rust requires each constraint
   - See how it prevents bugs

2. **Practice with Real Code**
   - Type out examples (don't copy-paste)
   - Modify examples to experiment
   - Make mistakes and understand errors

3. **Use the Compiler as a Teacher**
   - Rust's error messages are very helpful
   - Read them carefully
   - They tell you exactly what's wrong

4. **Build the Project**
   - Every lesson concept applies
   - Work on the project during lessons
   - See how theory becomes practice

### For Common Struggles

**"I don't understand ownership"**
- Re-read Lesson 5's concept section
- Draw diagrams of who owns what
- Try the practice exercises

**"Borrow checker won't let me do X"**
- It's preventing a real bug
- Lesson 6 explains the rules
- Refactor to satisfy the constraints

**"Why is this syntax so complex?"**
- It's explicit for safety
- Lesson 1 explains why static types matter
- The boilerplate prevents bugs

---

## Next: The Binary Protocol Project

Once you complete all lessons, use your knowledge for:

**[Binary Protocol Parser Project](../starter_code/README.md)**
- Integrates all Phase 1 concepts
- Real-world parsing problem
- Types, functions, error handling, modules
- Complete with tests

---

## Questions or Feedback?

These lessons are designed to be clear and comprehensive. If you:
- Find a concept confusing
- Want more examples
- Spot an error
- Have suggestions

**Your feedback helps improve the curriculum!**

---

## Lesson Checklist

Track your progress:

- [ ] Lesson 1: Primitive Data Types
- [ ] Lesson 2: Creating Functions
- [ ] Lesson 3: Creating Structs
- [ ] Lesson 4: Impl Blocks and Methods
- [ ] Lesson 5: Ownership Fundamentals
- [ ] Lesson 6: References and Borrowing
- [ ] Lesson 7: Lifetimes Explained
- [ ] Lesson 8: Smart Pointers and Collections
- [ ] Lesson 9: Pattern Matching and Destructuring
- [ ] Lesson 10: Enums and Associated Data
- [ ] Lesson 11: Option and Result
- [ ] Lesson 12: Custom Error Types
- [ ] Lesson 13: Modules and Visibility
- [ ] Lesson 14: Traits and Polymorphism

**Total: 14 lessons, ~40-50 hours of learning**

---

## Key Concepts Summary

| Concept | Why It Matters | Lesson |
|---------|---------------|--------|
| Types | Prevent errors at compile time | 1 |
| Functions | Code organization and reuse | 2 |
| Structs | Custom data types | 3 |
| Methods | Type-safe operations | 4 |
| Ownership | Memory safety without GC | 5 |
| Borrowing | Share without moving | 6 |
| Lifetimes | Prevent use-after-free | 7 |
| Collections | Variable-size storage | 8 |
| Pattern Matching | Exhaustive handling | 9 |
| Enums | Type-safe choices | 10 |
| Result | Explicit error handling | 11 |
| Error Types | Domain-specific errors | 12 |
| Modules | Code organization | 13 |
| Traits | Code abstraction | 14 |

---

## How Lessons Build On Each Other

```
Lesson 1 (Types)
    ↓
Lesson 2 (Functions) + Lesson 3 (Structs)
    ↓
Lesson 4 (Methods)
    ↓
Lessons 5-8 (Memory & Collections)
    ↓
Lessons 9-10 (Pattern Matching & Enums)
    ↓
Lessons 11-12 (Error Handling)
    ↓
Lesson 13 (Organization)
    ↓
Lesson 14 (Abstraction)
    ↓
Binary Protocol Parser PROJECT
```

Each lesson enables the next. Complete them in order!

---

Happy learning! 🦀
