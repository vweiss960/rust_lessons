# Binary Protocol Parser - Starter Code

## Project Overview

This project implements a parser for a simple binary protocol. It serves as the capstone project for Phase 1 (Rust Fundamentals), integrating knowledge from all lessons.

## Protocol Specification

The protocol is a simple message format with the following structure:

```
Byte Layout:
0           1              2-3              4...N           N+1
[Version]   [MessageType]  [PayloadLength]  [Payload]       [Checksum]
  (u8)        (u8)         (u16 BE)        (variable)       (u8)
```

### Field Descriptions

- **Version** (1 byte): Protocol version. Currently only version 1 is valid.
- **Message Type** (1 byte): Type of message (0-255). Different types may have different payload structures.
- **Payload Length** (2 bytes, big-endian): Length of the payload in bytes. Maximum payload size is 65535 bytes.
- **Payload** (variable): The actual message data.
- **Checksum** (1 byte): XOR of all payload bytes for integrity verification.

### Example

Valid packet for "Hello World":

```
Byte Index:  0     1     2     3     4     5     6     7     8     9    10    11    12    13    14
Value (hex): 0x01  0x05  0x00  0x0B  0x48  0x65  0x6C  0x6C  0x6F  0x20  0x57  0x6F  0x72  0x6C  0x64
             v1    type5 len=11(BE)  H     e     l     l     o     sp    W     o     r     l     d

Checksum calculation: 0x48 ^ 0x65 ^ 0x6C ^ 0x6C ^ 0x6F ^ 0x20 ^ 0x57 ^ 0x6F ^ 0x72 ^ 0x6C ^ 0x64 = 0x??
```

## Your Tasks

Implement the following components in `src/lib.rs`:

### 1. Error Handling
- [ ] Define `ParseError` enum with variants for:
  - Invalid version
  - Message too short
  - Checksum mismatch (include expected and actual values)
  - Payload too large
- [ ] Implement `Display` for error messages
- [ ] Implement `Error` trait

### 2. Protocol Types
- [ ] Complete the `Message` struct
- [ ] Implement `Message::new()`
- [ ] Implement `Message::to_bytes()` - serialize to protocol format
- [ ] Implement `Message::validate()` - verify message integrity
- [ ] Implement `Display` for pretty printing

### 3. Parsing Functions
- [ ] Implement `parse(data: &[u8]) -> Result<Message, ParseError>`
  - Validate minimum length (5 bytes header + payload + checksum)
  - Extract and validate version
  - Extract message type
  - Extract and validate payload length (big-endian)
  - Extract payload
  - Verify checksum
- [ ] Implement `parse_multiple()` for sequential parsing
- [ ] Implement helper functions:
  - `calculate_checksum()` - XOR all bytes
  - `bytes_to_u16()` - parse big-endian u16
  - `u16_to_bytes()` - encode big-endian u16

### 4. Tests
- [ ] Add at least 8 unit tests in `lib.rs`:
  - Valid simple message
  - Invalid version
  - Message too short
  - Checksum mismatch
  - Payload extraction
  - Empty payload
  - Maximum payload
  - Boundary conditions
- [ ] Add at least 2 integration tests in `tests/integration_tests.rs`:
  - Parse "Hello World" message
  - Round-trip test (create → serialize → parse)

## Getting Started

1. **Read** `src/lib.rs` - it contains detailed TODOs and comments
2. **Implement** error types and parsing logic
3. **Test** with `cargo test`
4. **Debug** using `cargo run`

## Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_parse_hello_world
```

## Key Concepts Applied

This project integrates:
- **Primitive Types** (u8, u16, Vec<u8>)
- **Functions** with proper signatures and return types
- **Structs** for protocol data
- **Methods** (impl blocks)
- **Ownership** (transferring byte vectors)
- **Borrowing** (&[u8] for read-only access)
- **Pattern Matching** (match expressions in parsing)
- **Enums** (for error types and message variants)
- **Error Handling** (Result<T, E>, ? operator)
- **Custom Errors** (ParseError trait implementation)
- **Traits** (Display, Error)
- **Modules** (lib.rs organization)

## Hints

### Checksum Calculation
XOR (^) each payload byte together:
```rust
let checksum = payload.iter().fold(0u8, |acc, &byte| acc ^ byte);
```

### Big-Endian Conversion
Use bit shifting to handle multi-byte values:
```rust
let value: u16 = ((data[0] as u16) << 8) | (data[1] as u16);
```

### Result and Error Propagation
Use the `?` operator to propagate errors:
```rust
let bytes = &data[2..4];
let length = bytes_to_u16(bytes)?;  // Returns early if error
```

## Validation Rules

The parser should validate:
1. Minimum length (5 bytes for header + at least 1 for checksum)
2. Version must be 1 (reject other versions)
3. Payload length must be valid (0-65535)
4. Total packet length must match: 5 + payload_length + 1
5. Checksum must match XOR of payload bytes

## Expected Compilation

The code should compile without warnings:
```bash
cargo build --release
cargo test
```

## Solution

See `../solution/` for a complete working implementation with tests.
