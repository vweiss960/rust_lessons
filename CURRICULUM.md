# Rust Networking Curriculum: Beginner to Advanced

**Target Audience:** Intermediate Python developers interested in building high-performance network applications

**Duration:** 30-36 weeks (adjustable based on pace) - includes 4 capstone projects with comprehensive debugging guidance

**Learning Path:** Project-based progression toward three capstone projects

---

## Table of Contents

0. [Hello World: Your First Rust Program](#hello-world-your-first-rust-program)
1. [Debugging Rust Code](#debugging-rust-code)
2. [Phase 1: Rust Fundamentals](#phase-1-rust-fundamentals-weeks-1-3)
3. [Phase 2: Systems Programming & Bytes](#phase-2-systems-programming--bytes-weeks-4-6)
4. [Phase 3: Async Rust & Networking Basics](#phase-3-async-rust--networking-basics-weeks-7-9)
5. [Debugging Network Code](#debugging-network-code)
6. [Phase 4: Packet Capture & Network I/O](#phase-4-packet-capture--network-io-weeks-10-12)
7. [Phase 5: Protocol Deep Dive](#phase-5-protocol-deep-dive-weeks-13-15)
8. [Phase 6: Performance Optimization](#phase-6-performance-optimization-weeks-16-18)
9. [Capstone Project 1: Network Tracker](#capstone-project-1-macsecipsec-network-tracker-weeks-19-22)
10. [Capstone Project 2: Packet Generator](#capstone-project-2-macsecipsec-packet-generator-weeks-23-26)
11. [Capstone Project 3: REST API Backend](#capstone-project-3-rest-api-backend-weeks-27-30)
12. [Capstone Project 4: gRPC Alternative](#capstone-project-4-grpc-api-backend-weeks-31-34)
13. [Optional: Advanced Projects](#optional-advanced-capstone-projects-weeks-35-40)

---

## Hello World: Your First Rust Program

Before diving into the curriculum, let's verify your Rust installation and get your bearings.

### Setup

1. **Install Rust** (if you haven't already):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

2. **Verify installation**:
```bash
rustc --version
cargo --version
```

### Your First Program

Create a new project:
```bash
cargo new hello_rust
cd hello_rust
```

This creates a directory structure:
```
hello_rust/
├── Cargo.toml
└── src/
    └── main.rs
```

Open `src/main.rs` and you'll see:
```rust
fn main() {
    println!("Hello, world!");
}
```

Run it:
```bash
cargo run
```

You should see: `Hello, world!`

### Understanding the Structure

- **Cargo.toml**: Package manifest (name, version, dependencies)
- **src/main.rs**: Entry point for your program
- **fn main()**: The main function that runs when your program starts
- **println!**: A macro (note the `!`) that prints text to console

### Next Steps

- Try modifying the text in `println!`
- Add a second `println!` statement
- Explore `cargo build` vs `cargo run` vs `cargo release`

This is the foundation. Once you're comfortable building and running projects, move to the debugging section and Phase 1.

---

## Debugging Rust Code

Before writing network code, master debugging tools. This applies to all phases of the curriculum.

### 1. Println Debugging

The simplest approach (often sufficient for this curriculum):

```rust
let x = 42;
println!("x = {}", x);
println!("x = {:?}", x);  // Debug formatting
println!("x = {:#?}", x); // Pretty-print Debug
```

**When to use:** Quick iterations, understanding data flow

**Limitations:** No breakpoints, no inspection of state at runtime

### 2. The Rust Debugger (GDB/LLDB)

Set up breakpoints and step through code:

**On macOS/Linux (using LLDB):**
```bash
cargo build  # Create debug binary
lldb ./target/debug/your_binary
(lldb) breakpoint set --file src/main.rs --line 10
(lldb) run
(lldb) p variable_name  # Print variable
(lldb) n  # Next line
(lldb) c  # Continue
```

**On Linux (using GDB):**
```bash
cargo build
gdb ./target/debug/your_binary
(gdb) break src/main.rs:10
(gdb) run
(gdb) print variable_name
(gdb) next
```

**IDE Integration:** Most IDEs (VS Code with CodeLLDB extension, RustRover, etc.) provide graphical debugging interfaces.

### 3. Assertions and Panics

Use assertions during development:

```rust
let value = 10;
assert_eq!(value, 10, "Expected value to be 10, got {}", value);
assert!(value > 5, "Value too small");
debug_assert!(value > 0);  // Removed in release builds
```

### 4. The dbg! Macro

Powerful debugging tool that prints and returns values:

```rust
let x = 5;
let y = dbg!(x + 1);  // Prints "[src/main.rs:2] x + 1 = 6"
```

### 5. Structured Logging

For larger projects, use `tracing` or `log`:

```rust
use log::{debug, info, warn, error};

fn process_data(value: i32) {
    debug!("Processing value: {}", value);
    if value < 0 {
        warn!("Negative value received: {}", value);
    }
}
```

Initialize logging:
```rust
env_logger::init();
```

Run with debug output:
```bash
RUST_LOG=debug cargo run
```

### 6. Rust-Analyzer & IDE Insights

Modern IDEs provide:
- **Type hints**: Hover over variables to see inferred types
- **Error messages**: Detailed explanations of compilation errors
- **Refactoring**: Automated code improvements
- **Quick fixes**: Suggested solutions to common errors

### 7. Common Debugging Scenarios

**Ownership/Borrow Checker Errors:**
- Read the error message carefully (Rust's messages are detailed)
- Visualize the lifetime of values
- Use `clone()` if desperate, but understand the performance cost

**Async Code Issues:**
- Print at strategic points in async code
- Remember that `.await` can be cancelled
- Use `tokio::spawn` with explicit task names for tracking

**Type Confusion:**
- Use `dbg!` to see actual types
- Explicitly annotate types when confused
- Check trait bounds in error messages

### Best Practices

1. **Start with println!** - It's fast and works everywhere
2. **Graduate to structured logging** - Better for understanding program flow
3. **Use assertions liberally** - Catch bugs early
4. **Debug in dev, not production** - Remove println!s before shipping
5. **Learn your IDE's debugger** - Essential for complex bugs
6. **Read error messages fully** - Rust's compiler is helpful

---

## PHASE 1: Rust Fundamentals (Weeks 1-3)

### 1.1 Core Language Concepts

Master the foundational concepts that make Rust unique:

- **Ownership & Borrowing**
  - Stack vs heap allocation
  - Move semantics and copying
  - Mutable vs immutable references
  - The borrow checker and lifetime basics

- **Pattern Matching & Enums**
  - `match` expressions
  - `if let` and `while let`
  - Enum variants and associated data
  - Custom data types

- **Error Handling**
  - `Result<T, E>` type
  - `Option<T>` type
  - `?` operator and error propagation
  - Custom error types

- **Modules & Visibility**
  - Module organization
  - `pub` visibility
  - `use` statements
  - Workspace structure

### 1.2 Project 1: Binary Protocol Parser

**Objective:** Build confidence with structs, traits, and error handling

**Description:**
Create a parser for a simple custom binary protocol (e.g., a toy protocol with version, message type, length, and payload):

- Define struct types for protocol messages
- Implement manual parsing from `Vec<u8>`
- Implement trait `Display` for pretty printing
- Handle malformed data gracefully
- Write unit tests for valid and invalid inputs

**Deliverables:**
- Working parser with full test coverage
- Error types that clearly communicate failures
- Documentation with examples

**Learning Outcomes:**
- Comfortable with ownership and borrowing
- Understanding trait implementation
- Pattern matching in real code
- Error handling patterns

---

## PHASE 2: Systems Programming & Bytes (Weeks 4-6)

### 2.1 Working with Binary Data

Develop deep understanding of how data is represented at the byte level:

- **Byte Manipulation**
  - Reading/writing bytes
  - Bitwise operations (AND, OR, XOR, shifts)
  - Endianness (big-endian vs little-endian)
  - Working with bit flags

- **Memory Layout & Alignment**
  - `#[repr(C)]` for C-compatible structs
  - Struct field ordering and padding
  - Size and alignment calculations
  - Stack vs heap for different types

- **Parser Combinator Libraries**
  - `nom` parser combinator basics
  - Byte-level parsers
  - Combinator composition
  - Error recovery and alternative parsing

- **Byte Slices & Efficient Data Handling**
  - Working with `&[u8]` vs `Vec<u8>`
  - Zero-copy patterns
  - Borrowing slices

### 2.2 Project 2: Network Header Parser (Educational)

**Objective:** Parse real network protocol headers without using external parsing libraries initially

**Description:**
Manually parse Ethernet, IPv4, IPv6, TCP, and UDP headers from raw bytes:

- Parse Ethernet frames (MAC addresses, EtherType)
- Parse IPv4 headers (version, IHL, flags, source/dest IP, checksum)
- Parse IPv6 headers (version, traffic class, flow label)
- Parse TCP headers (ports, sequence/ack numbers, flags, options)
- Parse UDP headers (ports, length)
- Implement checksum validation for IPv4 and TCP/UDP
- Create a unified packet structure

**Constraints:**
- Implement parsers manually WITHOUT external libraries (except maybe `bytes` for convenience)
- Handle edge cases (fragmentation flags, TCP options, IPv6 extension headers)
- Write comprehensive tests with real captured packet data

**Deliverables:**
- Complete header parser for common protocols
- Unit tests with real packet examples
- Comprehensive documentation of each header format
- Performance benchmarks showing parse speed

**Learning Outcomes:**
- Comfortable reading specifications and converting to code
- Understanding network protocol structures
- Bit manipulation confidence
- Zero-copy parsing patterns

---

## PHASE 3: Async Rust & Networking Basics (Weeks 7-9)

### 3.1 Async/Await & Tokio

Understand asynchronous programming model essential for network applications:

- **Futures & Async/Await**
  - Future trait and what it means
  - Async functions and blocks
  - Spawning tasks
  - Awaiting operations

- **Tokio Runtime**
  - Running async code
  - Multi-threaded vs single-threaded runtime
  - Task scheduling and fairness
  - Cancellation and timeouts

- **Concurrency Primitives**
  - Channels (`mpsc`, `broadcast`, `watch`)
  - Mutexes and RwLocks (`tokio::sync`)
  - Atomic operations
  - Synchronization patterns

- **Error Handling in Async**
  - Propagating errors from async functions
  - Cancellation-safe code
  - Timeouts and deadline handling

### 3.2 Socket Programming

Work with OS-level networking primitives:

- **UDP Sockets**
  - Creating UDP sockets with `tokio::net::UdpSocket`
  - Sending and receiving datagrams
  - Connectionless communication patterns
  - Performance characteristics (low latency, no guarantee)

- **TCP Sockets**
  - TCP listeners and connections
  - TcpStream and TcpListener
  - Connection pooling and reuse
  - Flow control and backpressure

- **Socket Options**
  - Setting send/receive buffer sizes
  - SO_REUSEADDR and SO_REUSEPORT
  - TCP_NODELAY (Nagle's algorithm)
  - IP_MULTICAST settings

- **Backpressure & Flow Control**
  - What backpressure is and why it matters
  - TCP flow control (window size)
  - Handling slow consumers (buffering strategies)
  - Bounded vs unbounded channels
  - When to apply backpressure vs when to drop data
  - Impact on million-packet systems (critical for capstones!)

- **Performance Tuning**
  - Measuring socket performance
  - Buffer sizing strategies
  - Thread affinity and locality

### 3.3 Project 3: Simple UDP Echo Server with Stats

**Objective:** Apply async patterns to build a concurrent, multi-client server

**Description:**
Build a multi-client UDP server that echoes received packets and tracks statistics:

- Accept incoming UDP datagrams from multiple clients
- Echo datagrams back to sender
- Track per-client statistics (packets received/sent, bytes transmitted, timestamps)
- Every second, output statistics (total packets/sec, active clients, bytes/sec)
- Handle client timeout (remove inactive clients after 60 seconds)
- Graceful shutdown on signal

**Requirements:**
- Use `tokio` for async runtime
- Use `tokio::sync::DashMap` or `Arc<Mutex>` for concurrent client tracking
- Separate stats task that outputs every second
- Configurable via CLI arguments (port, stats interval, timeout duration)

**Deliverables:**
- Working async UDP server
- Integration tests with test clients
- Statistics output to stdout and optional file
- Load test showing throughput (packets/sec)

**Learning Outcomes:**
- Comfortable writing async/await code
- Understanding task spawning and concurrency
- Practical experience with channels and synchronization
- Measuring and analyzing performance

### 3.4 Deep Dive: Backpressure in High-Performance Systems

Backpressure is critical for your capstone projects targeting millions of packets/second. Master this concept now.

#### What is Backpressure?

**Definition:** A mechanism where a slower component signals to faster upstream components to slow down or buffer data, preventing memory exhaustion and data loss.

**Example Scenario:**
```
Network (fast)  →  [Ring Buffer]  →  Parser (slow)  →  [Queue]  →  Tracker
   1M pps                              500K pps                       500K pps
```

If the parser can only handle 500K packets/second but the network sends 1M/second, the ring buffer fills up. Backpressure tells the network "I can't keep up, slow down or I'll drop data."

#### Types of Backpressure

**1. Implicit Backpressure (TCP Flow Control)**

TCP automatically applies backpressure via the window size:

```rust
// TCP receiver advertises how much data it can accept
// If you don't read from the socket, TCP's recv buffer fills
// TCP sends reduced window size to sender
// Sender automatically slows down

use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();
    let (mut socket, _) = listener.accept().await.unwrap();

    // If we don't read fast enough, TCP backpressure kicks in
    // Sender sees reduced window, slows down automatically

    let mut buf = [0u8; 1024];
    let n = socket.read(&mut buf).await.unwrap();
    println!("Read {} bytes", n);
}
```

**When to use:** TCP connections where you're happy with OS-level flow control

**2. Explicit Backpressure (Bounded Channels)**

Your code explicitly signals when to slow down:

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    // Bounded channel: max 1000 items in flight
    let (tx, mut rx) = mpsc::channel(1000);

    // Producer task
    tokio::spawn(async move {
        for i in 0..10_000_000 {
            // This will block if channel is full (backpressure!)
            tx.send(i).await.unwrap();
        }
    });

    // Consumer task (slow)
    tokio::spawn(async move {
        while let Some(item) = rx.recv().await {
            println!("Processing {}", item);
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }
    });
}
```

When the channel fills (1000 items), `tx.send()` blocks the producer. This slows down the fast producer to match the slow consumer.

**When to use:** When you want to prevent unbounded memory growth and coordinate between different pipeline stages

**3. Reactive Backpressure (Drop Policy)**

Instead of slowing down, drop data when overwhelmed:

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(1000);

    tokio::spawn(async move {
        for i in 0..10_000_000 {
            // Non-blocking send: returns error if channel full
            match tx.try_send(i) {
                Ok(_) => {},
                Err(_) => {
                    // Channel full, drop this packet
                    println!("Dropped packet {}", i);
                }
            }
        }
    });
}
```

**When to use:** Network applications where dropping data is acceptable (UDP, real-time streams) but blocking isn't

#### Ring Buffers: Backpressure for Ultra-High-Performance Systems

Ring buffers are crucial for your capstones. They implement a form of implicit backpressure:

```rust
pub struct RingBuffer<T> {
    buffer: Vec<T>,
    write_pos: AtomicUsize,
    read_pos: AtomicUsize,
    capacity: usize,
}

impl<T: Clone + Default> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![T::default(); capacity],
            write_pos: AtomicUsize::new(0),
            read_pos: AtomicUsize::new(0),
            capacity,
        }
    }

    pub fn try_push(&self, item: T) -> Result<(), T> {
        let write = self.write_pos.load(Ordering::Relaxed);
        let read = self.read_pos.load(Ordering::Relaxed);
        let next_write = (write + 1) % self.capacity;

        if next_write == read {
            // Buffer full - backpressure!
            return Err(item);
        }

        self.buffer[write] = item;
        self.write_pos.store(next_write, Ordering::Release);
        Ok(())
    }

    pub fn try_pop(&self) -> Option<T> {
        let read = self.read_pos.load(Ordering::Relaxed);
        let write = self.write_pos.load(Ordering::Acquire);

        if read == write {
            return None;  // Empty
        }

        let item = self.buffer[read].clone();
        let next_read = (read + 1) % self.capacity;
        self.read_pos.store(next_read, Ordering::Release);
        Ok(Some(item))
    }
}
```

**How ring buffer backpressure works:**
- When full, `try_push()` returns `Err`, signaling to the producer "buffer is full"
- Producer must decide: wait for space, drop data, or apply external backpressure
- No memory allocation, no blocking, deterministic latency

#### Choosing a Backpressure Strategy

**For Capstone 1 (Network Tracker):**
```
Network → [Ring Buffer] → [Parser Threads] → [Flow Tracker]

Strategy: Ring buffer with drop policy
- Ring buffer size: 100K-1M packets (tune based on parsing latency)
- If ring buffer full: drop or overwrite oldest packets
- Log dropped packets for monitoring
- Trade: lose some packets, but no memory growth, low latency
```

**For Capstone 2 (Packet Generator):**
```
[Packet Builder] → [Ring Buffer] → [Socket/File Output]

Strategy: Bounded channel with pacing
- Channel size: 10K-100K packets
- Builder blocks when channel full (natural pacing)
- Output thread drains at configured rate
- Trade: blocking producer, but exact rate control, no drops
```

**For Capstone 3 (REST API):**
```
[Tracker] → [Broadcast Channel] → [Multiple WebSocket Clients]

Strategy: Broadcast with bounded subscribers
- Broadcast channel (one sender, many receivers)
- Each subscriber: bounded channel (1000 items)
- If client can't keep up: drop its updates
- Trade: clients might miss updates, but no blocking tracker
```

#### Implementing Backpressure Handling

**Pattern 1: Producer-Consumer with Backpressure**

```rust
use tokio::sync::mpsc;
use std::time::Instant;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(10_000);  // 10K item buffer
    let mut dropped = 0;
    let mut sent = 0;

    // Producer
    tokio::spawn(async move {
        for i in 0..1_000_000 {
            match tx.send(i).await {
                Ok(_) => sent += 1,
                Err(_) => dropped += 1,
            }
        }
        println!("Producer done: sent={}, dropped={}", sent, dropped);
    });

    // Consumer (slow)
    let start = Instant::now();
    let mut processed = 0;
    while let Some(_) = rx.recv().await {
        processed += 1;
        if processed % 100_000 == 0 {
            println!("Processed {}, rate: {:.0} pps",
                     processed,
                     processed as f64 / start.elapsed().as_secs_f64());
        }
        tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
    }
}
```

**Pattern 2: Monitoring Backpressure**

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(1000);

    // Monitor channel depth
    tokio::spawn(async move {
        loop {
            let depth = tx.capacity() - tx.reserved_capacity();
            println!("Channel depth: {}/1000", depth);
            if depth > 800 {
                eprintln!("WARNING: High backpressure ({}% full)", depth);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });
}
```

#### Backpressure Anti-Patterns

**❌ Unbounded Channels**
```rust
// WRONG: Infinite memory growth
let (tx, rx) = mpsc::unbounded_channel();
```
When producer is faster than consumer, channel grows indefinitely until OOM.

**❌ Ignoring Backpressure**
```rust
// WRONG: No handling of channel full
tx.try_send(item).ok();  // Silently drops on error
```
Data loss with no metrics, can't debug.

**❌ Over-Large Ring Buffers**
```rust
// WRONG: Ring buffer of 10M packets
let ring = RingBuffer::new(10_000_000);
```
Uses too much memory, doesn't solve the underlying speed mismatch.

#### Best Practices

1. **Size buffers to match latency, not throughput**
   - If parsing takes 1ms per batch of 1000 packets, buffer ~1-10ms worth
   - Don't buffer 1 second of data hoping it reduces drops

2. **Monitor backpressure metrics**
   - Track channel depth, drop rate, queue latency
   - Alert when consistently full

3. **Design for the bottleneck**
   - Identify slowest stage (usually I/O or parsing)
   - Buffer before it, apply backpressure after it

4. **Test backpressure scenarios**
   - What happens if consumer dies?
   - What if producer floods suddenly?
   - Does buffer size grow unexpectedly?

5. **Document your strategy**
   - Why bounded vs unbounded?
   - What happens on overflow?
   - How is it monitored?

---

## Debugging Network Code

Network programming introduces unique debugging challenges. Master these techniques before the capstones.

### 1. Packet Inspection with tcpdump

Capture raw packets from the network:

```bash
# Capture all traffic on eth0
sudo tcpdump -i eth0

# Capture only UDP traffic
sudo tcpdump -i eth0 'udp'

# Capture with packet payloads (-A for ASCII, -X for hex)
sudo tcpdump -i eth0 'tcp port 8000' -A

# Save to file for analysis
sudo tcpdump -i eth0 -w captured.pcap

# Filter by IP address
sudo tcpdump -i eth0 'host 192.168.1.100'
```

**Why this matters:** See exactly what's on the wire. Compare your code's behavior to what tcpdump shows.

### 2. Wireshark GUI Analysis

For visual packet inspection:

```bash
# Install Wireshark
# macOS: brew install wireshark
# Linux: sudo apt install wireshark
# Windows: Download from wireshark.org

# Open existing pcap
wireshark captured.pcap

# Or capture live (with GUI filtering)
```

**Key features:**
- Follow TCP streams (decode full conversations)
- Expand packet headers to see field-by-field breakdown
- Color-code packet types
- Export specific packets for testing

### 3. Hexdump for Binary Analysis

Examine raw bytes of your packets:

```bash
# Hex dump of a pcap file
hexdump -C captured.pcap | head -50

# Compare two packet captures byte-by-byte
diff <(hexdump -C file1.pcap) <(hexdump -C file2.pcap)

# View specific byte ranges
xxd -l 64 -g 1 captured.pcap
```

**When to use:** Verify header correctness in generated packets, understand protocol structure

### 4. Logging at Network Boundaries

Strategic logging in network code:

```rust
use log::debug;

// Log on packet capture
debug!("Captured packet: {} bytes from {}", packet.len(), source_ip);
debug!("Raw bytes: {:?}", &packet[0..32.min(packet.len())]);

// Log on parsing
debug!("Parsed MACsec header: SCI={:?}, PN={}", sci, pn);

// Log anomalies
debug!("Sequence number gap: expected {}, got {}", expected_sn, actual_sn);

// Log flow state
debug!("Active flows: {}, anomalies this second: {}", flow_count, anomaly_count);
```

Run with logging:
```bash
RUST_LOG=debug cargo run -- <args>
```

### 5. Socket Tracing

See socket-level operations:

**Linux (strace):**
```bash
# Trace all system calls related to sockets
strace -e trace=socket,sendto,recvfrom cargo run

# Trace specific file descriptors
strace -e write,sendto cargo run 2>&1 | grep "fd="
```

**macOS (dtrace):**
```bash
# Monitor network syscalls
sudo dtrace -n 'syscall:::entry /execname == "your_binary"/ { @[execname] = count(); }'
```

### 6. Checksum & Header Validation

Implement validation functions early:

```rust
fn validate_ipv4_header(header: &[u8]) -> Result<(), String> {
    if header.len() < 20 {
        return Err("IPv4 header too short".to_string());
    }

    let version_ihl = header[0];
    let version = version_ihl >> 4;
    let ihl = (version_ihl & 0x0F) * 4;

    if version != 4 {
        return Err(format!("Invalid version: {}", version));
    }

    if ihl < 20 {
        return Err(format!("Invalid IHL: {}", ihl));
    }

    // Checksum validation
    let checksum_received = u16::from_be_bytes([header[10], header[11]]);
    let checksum_calculated = calculate_ipv4_checksum(&header[0..ihl]);

    if checksum_received != checksum_calculated {
        return Err(format!("Checksum mismatch: {} != {}",
                          checksum_received, checksum_calculated));
    }

    Ok(())
}
```

Use in tests:
```rust
#[test]
fn test_parse_real_packet() {
    let pcap_bytes = include_bytes!("test_packets/ipv4_packet.bin");
    validate_ipv4_header(pcap_bytes).unwrap();
}
```

### 7. Property-Based Testing for Protocols

Use `proptest` to verify parser correctness:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn parse_roundtrip(pn in 0u32..u32::MAX) {
        let original = MACsecPacket::new(pn);
        let bytes = original.to_bytes();
        let parsed = MACsecPacket::from_bytes(&bytes).unwrap();
        prop_assert_eq!(parsed.pn(), pn);
    }
}
```

Tests corner cases automatically (like sequence number wraparound).

### 8. Real Packet Test Data

Strategies for testing:

**Option A: Pre-recorded pcaps**
- Capture real MACsec/IPsec traffic (if available)
- Use `include_bytes!()` to embed in tests
- Compare parser output to Wireshark's dissection

**Option B: Synthetically generated**
- Use Project 2 (Packet Generator) to create test packets
- Verify generator output with parser
- Test anomaly cases (gaps, duplicates, rollovers)

**Option C: Public repositories**
- Download example pcaps from malware-traffic-analysis.net
- NETRESEC has public pcap collections
- Use for realistic testing scenarios

### 9. Common Network Debugging Mistakes

**Mistake 1: Endianness Confusion**
```rust
// WRONG: Assumes host byte order
let port = buffer[2] as u16 | ((buffer[3] as u16) << 8);

// RIGHT: Explicitly handle network byte order (big-endian)
let port = u16::from_be_bytes([buffer[2], buffer[3]]);
```

**Mistake 2: Off-by-One in Parsing**
```rust
// WRONG: Might read beyond packet boundary
let field = &buffer[10..14];

// RIGHT: Validate before reading
if buffer.len() < 14 {
    return Err("Packet too short");
}
let field = &buffer[10..14];
```

**Mistake 3: Ignoring TCP Flags**
```rust
// WRONG: Assumes all packets have payload
let payload = &buffer[iph_len + tcph_len..];

// RIGHT: Check flags (FIN, RST, SYN) which may have no payload
if tcp_flags & 0x01 != 0 { // FIN flag
    if buffer.len() == iph_len + tcph_len {
        // This is normal for FIN packets
    }
}
```

**Mistake 4: Sequence Number Wraparound**
```rust
// WRONG: Simple comparison fails near wraparound boundary
if received_sn > expected_sn + 1000 {
    return Err("Gap detected");
}

// RIGHT: Handle wraparound explicitly
fn has_gap(expected: u32, received: u32) -> bool {
    let diff = received.wrapping_sub(expected);
    diff > 1 && diff < u32::MAX / 2  // Account for wraparound
}
```

### 10. Debugging Checklist for Network Projects

Before debugging, verify the basics:

- [ ] Network interface is promiscuous mode (if needed)
- [ ] Running with appropriate permissions (root for raw sockets)
- [ ] Firewall isn't filtering test packets
- [ ] Packet size expectations are correct
- [ ] Byte order (endianness) is correct
- [ ] Sequence number assumptions are validated
- [ ] Off-by-one errors in header parsing
- [ ] Ring buffer/queue doesn't overflow silently
- [ ] Async tasks aren't being cancelled prematurely
- [ ] Lock contention isn't blocking packet processing

### Best Practices

1. **Log early, log often** - Network bugs are timing-sensitive
2. **Capture packets first, analyze second** - Save raw data for later inspection
3. **Test with known-good packets** - Use Wireshark as reference
4. **Validate assumptions** - Check header fields against specs
5. **Use assertions liberally** - Fail fast when assumptions are violated
6. **Compare to tcpdump** - Ground truth for network behavior

---

## PHASE 4: Packet Capture & Network I/O (Weeks 10-12)

### 4.1 Raw Packet Handling

Transition from socket programming to raw packet capture:

- **Packet Capture Libraries**
  - `libpnet` for cross-platform packet handling
  - `pcap-rs` for working with libpcap
  - Capabilities and limitations of each
  - Privilege requirements for packet capture

- **Raw Socket Programming**
  - Creating raw sockets
  - Platform differences (Linux, macOS, Windows)
  - Packet socket semantics
  - BPF (Berkeley Packet Filter) basics

- **Packet Buffer Management**
  - Ring buffers for efficient capture
  - Memory-mapped packet buffers
  - Zero-copy architectures
  - Overflow handling and packet loss

- **OS-Level Packet Handling**
  - Network interface enumeration
  - Monitoring multiple interfaces
  - Promiscuous mode and VLAN tagging
  - NIC offload capabilities

### 4.2 Project 4: Basic Packet Sniffer

**Objective:** Build infrastructure for live packet analysis

**Description:**
Create a packet sniffer that captures live network traffic and provides filtering:

- Capture packets from a specified network interface (or all interfaces)
- Display packet count and summary information
- Implement packet filtering by:
  - Protocol (IPv4, IPv6, TCP, UDP, ICMP)
  - Source/destination MAC address
  - Source/destination IP address
  - Port numbers
- Track statistics per protocol type
- Output to console or optional pcap file
- Clean shutdown without losing captured packets

**Requirements:**
- Use `libpnet` for packet capture
- Support multiple filter types (composable)
- Statistics update every second
- BPF filter integration (optional but recommended)
- Handle privilege requirements gracefully

**Deliverables:**
- Working packet sniffer with filtering
- Statistics dashboard (packets/sec by protocol)
- Integration with libpnet packet parser
- Documentation on platform requirements

**Learning Outcomes:**
- Understanding packet capture from OS perspective
- Real network interface handling
- Building on top of existing parser code
- Practical debugging of network issues

---

## PHASE 5: Protocol Deep Dive (Weeks 13-15)

### 5.1 MACsec Protocol

Deep study of IEEE 802.1AE (MACsec) protocol:

- **Frame Structure**
  - Ethernet header (destination MAC, source MAC, EtherType)
  - MACsec SecTAG (version, ES/SC bits, SCB, E, C, SL, PN)
  - Payload (encrypted or unencrypted in our use case)
  - ICV (Integrity Check Value)

- **Sequence Numbering**
  - Packet Number (PN) field format (32-bit or 64-bit)
  - PN rollover and wrapping behavior
  - Per-SA (Security Association) sequence numbering
  - Replay window protection concepts

- **Key Concepts**
  - Security Associations (SAs) and their identifiers
  - SCI (Secure Channel Identifier) - source MAC + port
  - Cipher suites (AES-GCM, ChaCha20-Poly1305)
  - Key derivation and session key concepts

- **Anomalies & Edge Cases**
  - PN wraparound detection
  - Duplicate packet detection
  - Out-of-order packets
  - Rollover implications

### 5.2 IPsec Protocol

Deep study of IP Security protocols:

- **Authentication Header (AH)**
  - AH header structure
  - Sequence number field (32-bit or 64-bit)
  - Integrity Check Value (ICV)
  - Transport vs tunnel mode differences

- **Encapsulating Security Payload (ESP)**
  - ESP header structure
  - SPI (Security Parameter Index)
  - Sequence number field
  - Encrypted payload and ICV
  - Padding and Next Header field

- **Sequence Numbering**
  - Per-SA sequence numbers
  - Replay window (default 64, configurable)
  - Anti-replay protection
  - Sequence number management

- **Packet Identification**
  - SPI-based SA lookup
  - Destination IP + protocol combo
  - Handling multiple SAs with same endpoint

- **Edge Cases**
  - Fragmentation and reassembly
  - Sequence number overflow
  - Encrypted vs plaintext payloads
  - Extension header handling

### 5.3 Project 5: MACsec/IPsec Header Parser

**Objective:** Implement production-quality parsers for both security protocols

**Description:**
Create comprehensive parsers for MACsec and IPsec protocols:

**MACsec Parsing:**
- Detect and parse SecTAG from Ethernet frames
- Extract Secure Channel Identifier (SCI)
- Extract and interpret Packet Number (PN)
- Validate ICV presence
- Track per-SCI sequence numbers

**IPsec Parsing:**
- Detect and parse AH headers
- Detect and parse ESP headers
- Extract SPI and use for flow identification
- Extract sequence numbers
- Distinguish transport vs tunnel mode

**Additional Requirements:**
- Define data structures for MACsec and IPsec packets
- Implement checksum/ICV validation (can be dummy for unencrypted payloads)
- Handle edge cases (jumbo frames, fragmentation)
- Create unified packet identification (flow = protocol + identifiers)
- Write tests with synthetically generated packets

**Deliverables:**
- Robust parsers for both protocols
- Data structures representing parsed packets
- Comprehensive test suite
- Documentation of protocol specifics
- Performance benchmarks for parser speed

**Learning Outcomes:**
- Deep protocol knowledge
- Complex parsing patterns
- Handling variable-length structures
- Practical protocol implementation experience

---

## PHASE 6: Performance Optimization (Weeks 16-18)

### 6.1 High-Performance Patterns

Master optimization techniques for low-latency, high-throughput systems:

- **Memory Management**
  - Custom allocators for hot paths
  - Object pooling and reuse
  - Pre-allocation strategies
  - NUMA awareness (multi-socket systems)
  - Cache line alignment and padding

- **Lock-Free & Low-Lock Programming**
  - Atomic operations and memory ordering
  - Lock-free queues and stacks
  - `parking_lot` for faster mutexes
  - `crossbeam` for efficient channels
  - When to lock vs when to use atomics

- **SIMD & Vectorization**
  - SIMD basics and Rust SIMD libraries
  - Auto-vectorization by compiler
  - Hand-optimized SIMD for header parsing
  - Platform-specific optimizations

- **Profiling & Benchmarking**
  - Using `criterion` for micro-benchmarks
  - `flamegraph` for identifying bottlenecks
  - `perf` integration for CPU profiling
  - Cache miss analysis
  - Memory profiling tools

### 6.2 Packet Processing Optimization

Apply general principles to packet processing:

- **Batch Processing**
  - Processing packets in groups
  - Improving cache locality
  - Reducing context switches
  - Throughput vs latency tradeoffs

- **Zero-Copy Techniques**
  - Working with borrowed data
  - Avoiding unnecessary allocations
  - Slice-based processing
  - DMA and kernel bypass concepts

- **Thread Organization**
  - CPU affinity and pinning
  - Separating capture from processing
  - NUMA-aware thread placement
  - Load balancing between threads

- **Hardware Capabilities**
  - NIC offload features
  - RSS (Receive Side Scaling)
  - RPS (Receive Packet Steering)
  - TSO/GSO (Segmentation Offload)

### 6.3 Project 6: Optimized Packet Processor

**Objective:** Achieve million-packet-per-second throughput baseline

**Description:**
Build an optimized packet processing pipeline:

- Create a synthetic packet source (memory buffer of pre-generated packets)
- Implement multi-threaded packet processing
- Apply parsing logic from Phase 5
- Measure and report throughput (packets/sec)
- Identify and eliminate bottlenecks

**Requirements:**
- Target: 1+ million packets/second
- Use `criterion` for benchmarking
- Profile with `flamegraph` to find hot spots
- Implement batch processing (process 1000s at a time)
- Use lock-free structures where appropriate
- CPU-pinned threads
- Pre-allocated buffers

**Deliverables:**
- Optimized packet processor achieving >1M packets/sec
- Benchmark suite with criterion
- Flamegraph profiles showing optimization work
- Report documenting optimization techniques applied
- Comparison of optimized vs baseline versions

**Learning Outcomes:**
- Profiling and optimization skills
- Identifying real bottlenecks vs perceived ones
- Lock-free programming confidence
- Performance measurement rigor

---

## CAPSTONE PROJECT 1: MACsec/IPsec Network Tracker (Weeks 19-22)

### Project Overview

Build a **high-performance packet analysis tool** that captures live network traffic and tracks sequence number progression in MACsec and IPsec packets.

### Business/Technical Goals

- **Real-time packet analysis** of security protocols
- **Sequence number tracking** across multiple flows
- **Anomaly detection** (gaps, duplicates, rollovers)
- **Scalability** to process 1-10 million packets per second
- **Low latency** monitoring for production networks

### Functional Requirements

**Packet Capture & Filtering:**
- Capture packets from one or more network interfaces
- Filter by protocol (MACsec, IPsec-AH, IPsec-ESP, or all)
- Filter by flow identifiers (MAC pairs for MACsec, SPI for IPsec)
- Support include/exclude rules

**Sequence Number Tracking:**
- Per-flow sequence number state
- Track current PN/SN for each flow
- Detect and record anomalies:
  - Out-of-order packets
  - Duplicate packets
  - Gaps in sequence numbers (>1 missing)
  - Sequence number rollovers
- Handle wraparound correctly (32-bit and 64-bit)

**Statistics & Reporting:**
- Per-second output:
  - Total packets captured
  - Packets per protocol (MACsec, AH, ESP)
  - Active flows being tracked
  - Anomalies detected this second
  - Packets/second rate
- Per-flow tracking:
  - Current sequence number
  - Packet count
  - Anomaly count
  - Last seen timestamp
- Configurable output (console, CSV, JSON)

**Flow Management:**
- Automatic flow discovery from packet headers
- Configurable flow timeout (remove flows inactive >N seconds)
- Flow statistics export
- Optional flow persistence (save/load state)

### Non-Functional Requirements

**Performance:**
- Target: Process 1+ million packets/second
- Latency: <1ms for packet capture → analysis → output
- Memory: Configurable max flows (prioritize older/inactive flows when limit reached)
- CPU: Efficient use (target single core for capture, additional cores for processing)

**Reliability:**
- No dropped packets in memory buffers (graceful overflow handling)
- Correct handling of all protocol variants
- Robust error handling and logging
- Graceful shutdown (flush final stats)

**Operational:**
- CLI interface with clear options
- Configurable via YAML/TOML config file
- Health metrics output
- Packet loss indicators

### Implementation Approach

**Architecture:**
```
[Network Interface]
        ↓
   [Packet Capture Thread]
        ↓
   [Ring Buffer / Queue]
        ↓
[Parsing Threads (N threads)]
        ↓
[Flow Tracking (DashMap/RwLock)]
        ↓
   [Stats Aggregation]
        ↓
   [Output Handler]
```

**Key Components:**
1. **PacketCapture**: Uses `libpnet` to capture packets from interface
2. **PacketParser**: Parses Ethernet → MACsec/IPsec headers (Phase 5 code)
3. **FlowTracker**: Maps flow identifiers to sequence number state
4. **AnomalyDetector**: Identifies sequence number violations
5. **StatisticsCollector**: Aggregates metrics
6. **OutputFormatter**: Writes stats to console/file

**Technologies:**
- `tokio` - async runtime for multi-threading
- `libpnet` - packet capture
- `dashmap` or `parking_lot::RwLock` - concurrent flow tracking
- `serde` - config file parsing
- `tracing` - structured logging

### Deliverables

1. **Source Code**
   - Modular Rust project with clear separation of concerns
   - Full test coverage (unit tests for parsing, flow tracking, anomaly detection)
   - Integration tests with synthetic packet streams
   - Benchmarks showing throughput

2. **Configuration & Deployment**
   - CLI argument parsing (clap)
   - YAML config file support
   - Clear logging/verbosity levels
   - Docker/container support (optional)

3. **Documentation**
   - Setup and installation guide
   - Usage examples with common scenarios
   - Configuration reference
   - Performance tuning guide
   - Architecture diagram and design decisions

4. **Testing & Validation**
   - Unit tests for sequence number tracking logic
   - Integration tests with synthetic MACsec/IPsec packets
   - Load tests showing 1M+ pps throughput
   - Validation against real captured packets (if available)

5. **Performance Artifacts**
   - Flamegraph profiles
   - Throughput benchmarks
   - Memory usage analysis
   - Comparison to baseline implementations

---

## CAPSTONE PROJECT 2: MACsec/IPsec Packet Generator (Weeks 23-26)

### Project Overview

Build a **high-performance packet generation tool** that creates valid MACsec and IPsec packets with correct headers and sequence numbers, capable of generating 1-10 million packets per second.

### Business/Technical Goals

- **Test data generation** for security protocol testing
- **Integration testing** with Project 1 (generate packets → feed to tracker)
- **Performance validation** of packet processing pipelines
- **Protocol compliance** verification
- **Stress testing** of network infrastructure

### Functional Requirements

**Packet Generation:**
- Generate valid Ethernet frames with MACsec SecTAG headers
- Generate valid IPsec packets (both AH and ESP variants)
- Configurable packet payloads:
  - Dummy/synthetic data for testing
  - Repeating patterns for easy identification
  - Random data option
- Headers are authentic and correctly formatted
- Payloads do NOT need to be encrypted (can be plaintext)

**Flow Configuration:**
- Define packet flows via configuration:
  - Source/destination MAC addresses
  - Source/destination IP addresses
  - SPI values (for IPsec)
  - Port numbers (if applicable)
- Multiple simultaneous flows
- Per-flow sequence number progression:
  - Normal sequential (1, 2, 3, ...)
  - With intentional gaps (1, 2, 5, 6, ...)
  - With duplicates (1, 2, 2, 3, ...)
  - With rollovers (approaching PN wraparound)

**Output Options:**
- Write to pcap file for later analysis
- Transmit via raw socket to live network interface
- Write to memory buffer (for synthetic testing)
- Rate limiting (packets/second, configurable)

**Packet Pacing & Control:**
- Constant rate generation
- Burst generation (as fast as possible)
- Realistic inter-packet delays
- Configurable packet count or infinite generation

### Non-Functional Requirements

**Performance:**
- Target: Generate 1+ million packets per second
- Latency: <1ms per generated packet
- Memory: Efficient pre-allocated buffers
- CPU: Optimized packet building

**Scalability:**
- Support 100s of concurrent flows
- Large payload sizes (up to jumbo frame sizes)
- Configurable batch sizes

**Reliability:**
- Reproducible packet generation (same seed = same packets)
- Correct header construction for all variants
- Proper sequence number handling and wraparound
- Error recovery for network transmission failures

### Implementation Approach

**Architecture:**
```
[Flow Configuration]
        ↓
[Flow State Manager]
        ↓
[Packet Builder (N threads)]
        ↓
[Packet Queue / Ring Buffer]
        ↓
[Rate Limiter]
        ↓
[Output Handler (pcap/socket)]
```

**Key Components:**
1. **FlowConfig**: Defines flows and their characteristics
2. **FlowState**: Tracks current sequence number, SCI, etc. per flow
3. **PacketBuilder**: Constructs valid MACsec/IPsec headers
4. **SequenceNumberManager**: Handles normal/gap/duplicate/rollover generation
5. **RateLimiter**: Paces packet output
6. **OutputHandler**: Writes to pcap or network socket

**Technologies:**
- `pnet_packet` - packet header builders
- `tokio` - async multi-threaded generation
- `pcap-rs` or `pcap` crate - writing pcap files
- `rand` - randomization options
- `serde` - config parsing

### Deliverables

1. **Source Code**
   - Modular Rust project
   - Packet builder abstractions for MACsec and IPsec
   - Configuration parser
   - Output handlers (pcap, socket, memory)
   - Comprehensive tests

2. **Configuration Format**
   - YAML/TOML config for flows
   - CLI override options
   - Example configurations for common scenarios

3. **Documentation**
   - Quick start guide
   - Configuration reference
   - Output format specifications
   - Integration guide with Project 1
   - Examples (normal flow, gap injection, duplicate injection, rollover testing)

4. **Testing & Validation**
   - Unit tests for packet builders
   - Integration tests comparing generated packets to specification
   - Round-trip tests (generate → parse with Project 1)
   - Performance benchmarks (packets generated/sec)

5. **Artifacts**
   - Pre-built packet profiles for common test scenarios
   - Example pcap files
   - Performance benchmarks
   - Comparison to other packet generators (optional)

---

## CAPSTONE PROJECT 3: REST API Backend (Weeks 27-30)

### Project Overview

Build a **RESTful HTTP API backend** that provides programmatic access to the Network Tracker (Project 1) functionality. This service exposes packet analysis and flow statistics through HTTP endpoints with JSON responses, enabling remote monitoring, integration with dashboards, and browser-based clients.

### Business/Technical Goals

- **REST API** - standard HTTP interface with JSON responses
- **Real-time statistics** accessible to multiple HTTP clients
- **WebSocket support** - push live updates to connected clients
- **Browser-friendly** - JSON responses work with any HTTP client, web dashboards, Postman, curl
- **Integration platform** - enable third-party tools and dashboards to consume packet analysis
- **Scalability** - support 10+ concurrent HTTP clients, 100+ WebSocket connections without impacting packet capture

### Functional Requirements

**Core API Endpoints:**

**Statistics & Monitoring:**
- `GET /api/v1/stats/current` - Real-time statistics (packets/sec, flows, anomalies)
- `GET /api/v1/stats/history?duration=1h&interval=10s` - Historical stats aggregated by time interval
- `GET /api/v1/flows` - List all active flows
- `GET /api/v1/flows/{flow_id}` - Detailed flow statistics
- `GET /api/v1/flows/{flow_id}/sequence` - Sequence number history for flow
- `GET /api/v1/flows/{flow_id}/anomalies` - Anomalies detected on flow

**Configuration & Control:**
- `GET /api/v1/config` - Current tracker configuration
- `POST /api/v1/config/filters` - Add/update packet filters
- `DELETE /api/v1/config/filters/{filter_id}` - Remove filters
- `POST /api/v1/flows/{flow_id}/reset` - Reset flow tracking state
- `POST /api/v1/capture/pause` - Pause packet capture
- `POST /api/v1/capture/resume` - Resume packet capture

**Packet/Anomaly Details:**
- `GET /api/v1/anomalies?limit=100&flow_id=...` - Query anomalies with filters
- `GET /api/v1/anomalies/{anomaly_id}` - Details of specific anomaly
- `GET /api/v1/packets/recent?flow_id=...&count=10` - Most recent packets

**Health & Diagnostics:**
- `GET /health` - Service health check
- `GET /metrics` - Prometheus-style metrics (Grafana integration)
- `GET /api/v1/diagnostics` - Internal state and performance metrics

**WebSocket Endpoints (Optional):**
- `WS /api/v1/stream/stats` - Live stream of statistics updates
- `WS /api/v1/stream/flows` - Live stream of new flows and updates
- `WS /api/v1/stream/anomalies` - Live stream of detected anomalies

### Data Persistence

- **Time-Series Database** (optional):
  - Store statistics history (1 minute, 1 hour, 1 day aggregations)
  - Query historical trends
  - Configurable retention period (default: 30 days)

- **Flow State Serialization**:
  - Option to persist flow states to disk
  - Resume tracking after restart
  - Export flow data in JSON format

### Non-Functional Requirements

**Performance:**
- API response time: <100ms for stats queries
- WebSocket update latency: <500ms per update
- Support 10+ concurrent HTTP clients
- Support 100+ WebSocket subscribers
- Zero impact on packet capture performance

**Scalability:**
- Horizontal scaling (multiple instances behind load balancer)
- Share state across instances (optional Redis backend)
- Configurable buffer sizes for historical data

**Security:**
- Optional API key authentication
- Rate limiting per client
- CORS configuration
- HTTPS support
- Input validation and sanitization

**Reliability:**
- Graceful error handling
- Request timeout handling
- Connection pooling
- Database transaction consistency
- Backup/restore capabilities

### Implementation Approach

**Architecture:**
```
[Network Tracker (Project 1)]
        ↓
[Shared State / Data Channel]
        ↓
[API Server]
   ├─→ [HTTP Handler] → HTTP Clients
   ├─→ [WebSocket Handler] → WebSocket Clients
   └─→ [Time-Series DB] (optional)
```

**Key Components:**
1. **APIServer**: HTTP server with routing (Axum or Actix-web)
2. **StateManager**: Shared state from tracker (Arc<Mutex<...>>)
3. **StatisticsCollector**: Aggregates metrics (real-time and historical)
4. **TimeSeriesDB**: Stores and queries historical data
5. **WebSocketHandler**: Maintains connections and broadcasts updates
6. **AuthMiddleware**: API key validation (optional)
7. **RateLimiter**: Per-client rate limiting

**Technologies:**
- **HTTP Framework**: `axum` or `actix-web` (pick based on preference)
- **WebSockets**: `tokio-tungstenite` or built-in framework support
- **Database**:
  - In-memory: `Arc<DashMap>` for simple deployments
  - Time-series: `InfluxDB`, `TimescaleDB`, or `Prometheus`
- **Serialization**: `serde_json` for JSON
- **Metrics**: `prometheus` crate + Prometheus format output
- **Config**: `serde` with TOML/YAML

### API Schema & Data Models

**Core Models:**
```rust
pub struct Statistics {
    pub timestamp: DateTime<Utc>,
    pub total_packets: u64,
    pub packets_per_second: f64,
    pub active_flows: usize,
    pub anomalies_detected: u64,
    pub bytes_processed: u64,
    pub capture_loss: f64,
}

pub struct Flow {
    pub id: String,              // Unique flow identifier
    pub protocol: Protocol,      // MACsec, AH, ESP
    pub source_identifier: String,
    pub dest_identifier: String,
    pub current_sequence_number: u64,
    pub packet_count: u64,
    pub anomaly_count: u64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

pub struct Anomaly {
    pub id: String,
    pub flow_id: String,
    pub anomaly_type: AnomalyType,  // Gap, Duplicate, Rollover
    pub sequence_number: u64,
    pub expected_sequence_number: u64,
    pub timestamp: DateTime<Utc>,
    pub packet_count: u32,
}

pub enum Protocol {
    MACsec,
    IPsecAH,
    IPsecESP,
}

pub enum AnomalyType {
    OutOfOrder,
    Duplicate,
    Gap { missing_count: u32 },
    Rollover,
}
```

### Deliverables

1. **Source Code**
   - RESTful API implementation
   - WebSocket handler
   - Database integration (or in-memory alternative)
   - Middleware for auth/rate-limiting/CORS
   - Comprehensive API tests
   - Integration tests with Project 1

2. **API Documentation**
   - OpenAPI/Swagger spec (auto-generated from code)
   - Endpoint reference with examples
   - Authentication guide
   - Error code documentation
   - Rate limiting policy
   - WebSocket message formats

3. **Deployment & Ops**
   - Docker configuration
   - Docker Compose for local development
   - Environment configuration reference
   - Kubernetes manifests (optional)
   - Monitoring/alerting setup guide

4. **Client Libraries (Optional)**
   - Rust client library for the API
   - Python client wrapper
   - JavaScript/TypeScript client
   - curl examples for all endpoints

5. **Testing & Validation**
   - Unit tests for API handlers
   - Integration tests with full tracker
   - Load tests (concurrent clients)
   - API contract tests
   - Security/validation tests

6. **Artifacts & Examples**
   - Postman/Insomnia collection
   - Example dashboard configuration (Grafana, etc.)
   - Sample use cases and tutorials
   - Performance benchmarks

### Integration with Projects 1 & 2

**With Project 1 (Tracker):**
- Embed API in same binary or run as separate service
- Share state via channels or inter-process communication
- Mirror tracker config options through API

**With Project 2 (Generator):**
- API client can query live stats while generator produces packets
- Validate generated packets are correctly tracked
- Load testing: generate packets, monitor via API

---

## CAPSTONE PROJECT 4: gRPC API Backend (Weeks 31-34)

### Project Overview

Build a **high-performance gRPC API backend** that provides programmatic access to the Network Tracker (Project 1) functionality. This service exposes packet analysis and flow statistics through gRPC endpoints with Protocol Buffer serialization, enabling high-throughput remote monitoring, binary protocol integration, and performance-critical clients.

### Business/Technical Goals

- **gRPC protocol** - binary protocol with lower latency and higher throughput than REST
- **Protocol Buffers** - efficient serialization for network packet data
- **Real-time streaming** - server-side streaming for live statistics and anomalies
- **Bidirectional streaming** - duplex communication for filter updates and live monitoring
- **High-performance clients** - native language support (Rust, Python, Go, Java, etc.)
- **Integration platform** - enable high-speed network monitoring with minimal overhead
- **Scalability** - support 100+ concurrent gRPC clients with persistent streams

### Functional Requirements

**gRPC Service Definition (Protobuf):**

```protobuf
syntax = "proto3";

package network_tracker;

service NetworkTracker {
  // Query current statistics
  rpc GetCurrentStats(Empty) returns (Statistics);

  // Get historical statistics with time range
  rpc GetStatsHistory(StatsHistoryRequest) returns (stream Statistics);

  // List all active flows
  rpc GetFlows(Empty) returns (stream Flow);

  // Get details of specific flow
  rpc GetFlow(FlowIdRequest) returns (Flow);

  // Stream real-time anomalies
  rpc StreamAnomalies(Empty) returns (stream Anomaly);

  // Bidirectional: receive filter updates, stream matching packets
  rpc StreamPackets(stream PacketFilter) returns (stream PacketSummary);

  // Configure tracker (pause/resume/reset)
  rpc UpdateConfig(TrackerConfig) returns (ConfigResponse);

  // Health check
  rpc HealthCheck(Empty) returns (HealthStatus);
}

message Statistics {
  int64 timestamp_ms = 1;
  uint64 total_packets = 2;
  double packets_per_second = 3;
  int32 active_flows = 4;
  uint64 anomalies_detected = 5;
  uint64 bytes_processed = 6;
  double capture_loss_percent = 7;
}

message Flow {
  string flow_id = 1;
  enum Protocol {
    MACSEC = 0;
    IPSEC_AH = 1;
    IPSEC_ESP = 2;
  }
  Protocol protocol = 2;
  string source_identifier = 3;
  string dest_identifier = 4;
  uint64 current_sequence_number = 5;
  uint64 packet_count = 6;
  uint64 anomaly_count = 7;
  int64 first_seen_ms = 8;
  int64 last_seen_ms = 9;
}

message Anomaly {
  string anomaly_id = 1;
  string flow_id = 2;
  enum AnomalyType {
    OUT_OF_ORDER = 0;
    DUPLICATE = 1;
    GAP = 2;
    ROLLOVER = 3;
  }
  AnomalyType anomaly_type = 3;
  uint64 sequence_number = 4;
  uint64 expected_sequence_number = 5;
  int64 timestamp_ms = 6;
  uint32 packet_count = 7;
}

message PacketFilter {
  string flow_id_filter = 1;  // Optional
  enum Protocol {
    ALL = 0;
    MACSEC = 1;
    IPSEC_AH = 2;
    IPSEC_ESP = 3;
  }
  Protocol protocol_filter = 2;
}

message PacketSummary {
  int64 timestamp_ms = 1;
  string flow_id = 2;
  uint64 sequence_number = 3;
  uint32 packet_size = 4;
  bool is_anomaly = 5;
}
```

**Key Service Features:**

**Unary RPCs (Request-Response):**
- `GetCurrentStats()` - Query latest stats (sub-millisecond response)
- `GetFlow()` - Detailed flow information

**Server Streaming:**
- `GetStatsHistory()` - Stream historical stats with pagination
- `GetFlows()` - Stream all active flows
- `StreamAnomalies()` - Stream detected anomalies in real-time

**Bidirectional Streaming:**
- `StreamPackets()` - Client sends filter updates, server streams matching packets
  - Client can dynamically update filters without disconnecting
  - Server streams only matching packets
  - Backpressure handled by gRPC flow control

**Configuration Management:**
- `UpdateConfig()` - Pause/resume capture, reset flows, update filters
- `HealthCheck()` - Service health status

### Non-Functional Requirements

**Performance:**
- Latency: <10ms for unary RPCs, <1ms for streaming message latency
- Throughput: 100K+ messages/second on single connection
- Connection overhead: Support 100+ concurrent connections
- Zero impact on packet capture performance
- Binary protocol overhead <10% vs REST

**Protocol Efficiency:**
- Message size: ~50-100 bytes per update (vs JSON's 300-500 bytes)
- Compression: optional gzip for bandwidth reduction
- Streaming: server-side backpressure via gRPC flow control

**Reliability:**
- Graceful handling of slow clients
- Automatic reconnection support
- Deadlines and cancellation propagation
- Error messages with detailed context

**Security:**
- Optional TLS/mTLS support
- Token-based authentication
- Rate limiting per client/service
- Input validation on config changes

### Implementation Approach

**Architecture:**
```
[Network Tracker (Project 1)]
        ↓
[Shared State / Data Channel]
        ↓
[gRPC Server]
   ├─→ [Unary Handler] → Single response
   ├─→ [Server Stream] → Unidirectional stream
   └─→ [Bidirectional] → Two-way stream
        ↓
   [gRPC Clients]
   - Rust native clients
   - Python/Go CLI tools
   - WebAssembly dashboards
```

**Key Components:**
1. **ProtobufDefinitions**: Message types and service definition
2. **GrpcServer**: Tokio-based gRPC server
3. **ServiceImpl**: Implement all RPC methods
4. **StateManager**: Access tracker state safely
5. **BackpressureHandler**: Manage gRPC flow control
6. **AuthMiddleware**: Token validation (optional)
7. **MetricsCollector**: Track RPC performance

**Technologies:**
- **gRPC**: `tonic` - Rust's high-performance gRPC framework
- **Protocol Buffers**: `prost` - Rust protobuf compiler
- **Async**: `tokio` - built-in with tonic
- **Streaming**: Built-in bidirectional streaming
- **TLS**: `tokio-rustls` or `native-tls`
- **Health Checks**: `tonic-health` crate

### Deliverables

1. **Source Code**
   - Proto file definitions
   - gRPC service implementation
   - Middleware (auth, rate limiting, logging)
   - Unit and integration tests
   - Load tests with high client concurrency

2. **Client Libraries**
   - Rust client library (auto-generated from proto)
   - Python client wrapper (using grpcio)
   - Go/Java clients (optional)
   - CLI tool for testing (rust-based)

3. **Protocol Documentation**
   - Proto file with detailed comments
   - Service API reference
   - Message field documentation
   - Stream behavior documentation
   - Error codes and meanings

4. **Deployment & Operations**
   - Docker support with TLS configuration
   - gRPC gateway (HTTP/JSON bridge, optional)
   - Kubernetes manifests
   - Health check endpoint
   - Metrics export (Prometheus format)

5. **Testing & Validation**
   - Unit tests for each RPC
   - Integration tests with tracker
   - Load tests (100+ concurrent connections)
   - Bidirectional streaming tests
   - Backpressure handling tests
   - Protocol buffer validation

6. **Performance Artifacts**
   - Latency benchmarks (unary vs streaming)
   - Throughput tests (messages/sec)
   - Comparison to REST API (Capstone 3)
   - Connection overhead analysis
   - Memory usage profiling

### Comparison: REST API vs gRPC

| Aspect | REST (Capstone 3) | gRPC (Capstone 4) |
|--------|-------------------|-------------------|
| **Protocol** | HTTP/1.1 or HTTP/2 | HTTP/2 binary |
| **Serialization** | JSON text | Protocol Buffers binary |
| **Message Size** | 300-500 bytes | 50-100 bytes |
| **Latency** | 10-100ms | 1-10ms |
| **Streaming** | Server-side only | Bidirectional |
| **Backpressure** | Manual handling | Automatic gRPC flow control |
| **Browser Friendly** | Yes (native support) | No (requires gateway) |
| **Client Libraries** | Any HTTP client | Generated from proto |
| **Learning Curve** | Lower (HTTP familiar) | Higher (proto/gRPC concepts) |
| **Performance** | Good for moderate loads | Excellent for high throughput |

**Choose Capstone 3 (REST)** if you need:
- Browser/JavaScript clients
- Simple third-party integration
- Lower learning curve
- HTTP debugging tools

**Choose Capstone 4 (gRPC)** if you need:
- High-throughput monitoring
- Real-time bidirectional streams
- Minimal bandwidth
- Native language clients
- Sub-millisecond latency

### Integration with Other Capstones

**With Project 1 (Tracker):**
- Share same process state
- gRPC server as subsystem of tracker
- No inter-process overhead

**With Project 2 (Generator):**
- gRPC client queries live stats while generator produces packets
- Bidirectional stream: send filters, receive matching packet summaries
- Demonstrate low-latency stream performance

**With Capstone 3 (REST API):**
- Both implemented alongside each other
- Share same state manager
- Compare performance in benchmarks
- REST for dashboards, gRPC for high-performance clients

---

## Optional: Advanced Capstone Projects (Weeks 35-40)

After completing the three main capstones, choose 1-2 additional projects based on interest:

### Option A: Protocol Anomaly Detection Engine

**Goal:** Statistical analysis and ML-based anomaly detection

**Features:**
- Statistical baseline of normal sequence number patterns
- Detect unusual patterns (high gap rates, repeated rollovers)
- Machine learning pipeline (simple decision trees/random forests)
- Alert thresholds and severity scoring
- Integration with Project 1 tracker

**Technologies:**
- `ndarray` and `ndarray-stats` for numerical computing
- `smartcore` or `sklearn-rs` for ML models
- Custom feature engineering from packet data

### Option B: Packet Reconstruction & Forensics

**Goal:** Deep packet analysis and forensics

**Features:**
- Reassemble fragmented IPv4 packets
- Extract and display payloads
- Timeline reconstruction of packet flows
- Correlation across multiple flows
- Export for forensic analysis

**Technologies:**
- Integration with Project 1 parser
- Custom reassembly logic
- Time-series analysis

### Option C: Multi-Interface Correlation

**Goal:** Distributed packet analysis

**Features:**
- Capture from multiple network interfaces simultaneously
- Track packet flow across interface boundaries
- Latency and jitter measurement
- Load balancing and failover testing
- Visualization of traffic patterns

**Technologies:**
- Multiple async tasks per interface
- Cross-interface flow correlation
- Timing and latency measurement

### Option D: Custom Protocol Implementation

**Goal:** Design and implement your own protocol

**Features:**
- Design a custom encapsulation/security protocol
- Implement serialization/deserialization
- Compare performance vs MACsec/IPsec
- Document specification
- Benchmarks

**Technologies:**
- `pnet_packet` builders
- Custom header definitions
- Performance profiling

---

## Curriculum Timeline Summary

| Phase | Duration | Focus | Projects | Key Deliverable |
|-------|----------|-------|----------|-----------------|
| 0 | 1 day | Setup & Hello World | N/A | Working Rust environment |
| Debugging Intro | 1 week | Rust debugging tools | N/A | Proficiency with println, debugger, logging |
| 1 | 3 weeks | Language basics | Project 1 | Binary protocol parser |
| 2 | 3 weeks | Binary data | Project 2 | Network header parser |
| 3 | 3 weeks | Async/networking | Project 3 | UDP echo server |
| Debugging Network | 1 week | tcpdump, Wireshark, validation | N/A | Network debugging skills |
| 4 | 3 weeks | Packet capture | Project 4 | Packet sniffer |
| 5 | 3 weeks | Protocol deep dive | Project 5 | MACsec/IPsec parser |
| 6 | 3 weeks | Performance tuning | Project 6 | Optimized processor (1M pps) |
| **Capstone 1** | **4 weeks** | **Network tracker** | **Capstone 1** | **Live packet analyzer** |
| **Capstone 2** | **4 weeks** | **Packet generator** | **Capstone 2** | **Packet generator (1M pps)** |
| **Capstone 3** | **4 weeks** | **REST API backend** | **Capstone 3** | **RESTful API service** |
| **Capstone 4** | **4 weeks** | **gRPC API backend** | **Capstone 4** | **gRPC service** |
| Advanced | 6 weeks | Optional deep dives | Optional | Specialized project |

**Total Time:** 30-36 weeks (adjustable based on pace and depth)

---

## Key Libraries & Tools Reference

### Core Networking
- **tokio** - Async runtime and networking primitives
- **libpnet** - Cross-platform packet handling
- **pcap-rs** - Wrapper around libpcap
- **pnet_packet** - Packet header builders and parsers

### Parsing & Serialization
- **nom** - Parser combinators for binary formats
- **bytes** - Efficient byte manipulation
- **bitvec** - Bit-level operations
- **serde** - Generic serialization framework

### Performance & Profiling
- **criterion** - Benchmarking framework
- **flamegraph** - Visualization of profiling data
- **parking_lot** - Faster mutex/rwlock implementations
- **crossbeam** - High-performance concurrency utilities
- **proptest** - Property-based testing

### API & Web
- **axum** or **actix-web** - HTTP web frameworks
- **tokio-tungstenite** - WebSocket support
- **prometheus** - Metrics/monitoring
- **sqlx** or **tokio-postgres** - Database access
- **serde_json** - JSON serialization

### Development Tools
- **clippy** - Linting and suggestions
- **cargo-flamegraph** - Flamegraph integration
- **cargo-watch** - Auto-run on file changes
- **cargo-criterion** - Criterion integration
- **cargo-tarpaulin** - Code coverage

---

## Learning Principles

1. **Project-First Approach**
   - Every concept is immediately applied to a project
   - Projects build progressively on each other
   - Real-world context from day one

2. **Incremental Complexity**
   - Start with simpler protocols (UDP, basic sockets)
   - Progress to complex concurrent systems
   - Capstones integrate all learned concepts

3. **Performance Awareness**
   - Optimization isn't an afterthought (Phase 6)
   - Profile before optimizing
   - Benchmarks drive decisions

4. **Network-Centric**
   - All projects relate to networking
   - Deep protocol knowledge earned through implementation
   - Real production concerns (anomalies, reliability, scaling)

5. **Python-Friendly Transition**
   - Start with familiar concepts (parsing, data structures)
   - Async/await is similar to Python's asyncio
   - Gradually introduce Rust-specific concepts
   - Performance improvements visible throughout

---

## Additional Resources

### Documentation
- [The Rust Book](https://doc.rust-lang.org/book/) - Official Rust guide
- [Async Rust](https://tokio.rs/tokio/tutorial) - Tokio tutorial
- [OWASP Top 10](https://owasp.org/) - Security best practices
- Protocol RFCs:
  - [MACsec (IEEE 802.1AE)](https://standards.ieee.org/ieee/802.1AE/)
  - [IPsec (RFC 4301, 4302, 4303)](https://tools.ietf.org/html/rfc4301)
  - [IP (RFC 791, 8200)](https://tools.ietf.org/html/rfc791)
  - [TCP (RFC 793)](https://tools.ietf.org/html/rfc793)
  - [UDP (RFC 768)](https://tools.ietf.org/html/rfc768)

### Communities
- [Rust Users Forum](https://users.rust-lang.org/)
- [Rust Discord](https://discord.gg/rust-lang)
- Tokio documentation and examples
- r/rust subreddit for questions

---

## Success Criteria

By completing this curriculum, you should be able to:

1. ✅ Write idiomatic Rust code with confidence
2. ✅ Design and implement async systems at scale
3. ✅ Parse and manipulate binary data efficiently
4. ✅ Build network applications with millions of packets/second throughput
5. ✅ Profile and optimize Rust code for performance
6. ✅ Understand MACsec and IPsec protocols deeply
7. ✅ Build three production-quality capstone projects
8. ✅ Design RESTful APIs and real-time systems
9. ✅ Deploy and operate networked systems

---

**Created:** 2025-12-21
**Version:** 1.0
**Target Audience:** Intermediate Python developers pursuing advanced Rust networking skills
