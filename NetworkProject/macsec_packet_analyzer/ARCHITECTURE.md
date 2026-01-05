# MACsec Packet Analyzer - Architecture Documentation

## Overview

This is a modular Rust packet analyzer designed for MACsec traffic analysis with packet number gap detection. The architecture emphasizes:

- **Modularity**: Packet parsing is independent of data source
- **Extensibility**: Easy to add new protocols (IPsec, etc.) and capture sources (live interfaces)
- **Reusability**: Can be used as a library in other applications
- **Type Safety**: Leverages Rust's strong type system for correctness

## Module Structure

```
src/
├── main.rs                    # CLI entry point (requires pcap-dep feature)
├── lib.rs                     # Library root and public API exports
├── types.rs                   # Common data types used across modules
├── error.rs                   # Error types for all modules
├── capture/                   # Packet capture abstraction layer
│   ├── mod.rs                 # Module exports
│   ├── source.rs              # PacketSource trait definition
│   └── file.rs                # PCAP file reader implementation
├── protocol/                  # Protocol parsing abstraction layer
│   ├── mod.rs                 # Module exports
│   ├── parser.rs              # SequenceParser trait definition
│   └── macsec.rs              # MACsec packet parser implementation
└── analysis/                  # Gap detection and flow tracking
    ├── mod.rs                 # Analyzer engine (combines source + parser + tracker)
    └── flow.rs                # Flow tracking with sequence reordering
```

## Key Abstractions

### 1. PacketSource Trait (`src/capture/source.rs`)

Defines the interface for packet capture sources:

```rust
pub trait PacketSource {
    fn next_packet(&mut self) -> Result<Option<RawPacket>, CaptureError>;
    fn stats(&self) -> CaptureStats;
}
```

**Implementations:**
- `FileCapture` - Reads from PCAP files (file.rs)
- `LiveCapture` - Live network interface capture (future)

**Why this abstraction:** The analyzer doesn't care where packets come from - it just needs an iterator.

### 2. SequenceParser Trait (`src/protocol/parser.rs`)

Defines the interface for sequence number extraction from packets:

```rust
pub trait SequenceParser {
    fn parse_sequence(&self, data: &[u8]) -> Result<Option<SequenceInfo>, ParseError>;
    fn matches(&self, data: &[u8]) -> bool;
    fn protocol_name(&self) -> &str;
}
```

**Implementations:**
- `MACsecParser` - Extracts packet number from MACsec SecTag (macsec.rs)
- `IPsecParser` - Future implementation for IPsec sequence numbers

**Why this abstraction:** Different protocols have different headers and field locations. This trait makes adding new protocols trivial.

### 3. FlowTracker (`src/analysis/flow.rs`)

Manages per-flow state including:
- Sequence number tracking
- Reordering buffer for out-of-order packets
- Gap detection with wraparound handling
- Statistics aggregation (min/max gap size, total losses)

**Key features:**
- Maintains per-flow state in a HashMap keyed by FlowId
- Handles 32-bit sequence number wraparound
- Implements smart reordering to buffer out-of-order packets
- Tracks detailed statistics for each flow

### 4. PacketAnalyzer (`src/analysis/mod.rs`)

Generic analyzer that orchestrates:
1. Reading packets from a `PacketSource`
2. Parsing packets with a `SequenceParser`
3. Tracking sequences in the `FlowTracker`
4. Generating an `AnalysisReport`

**Generic signature:**
```rust
pub struct PacketAnalyzer<S: PacketSource, P: SequenceParser> {
    source: S,
    parser: P,
    flow_tracker: FlowTracker,
}
```

This is the key to modularity - the analyzer is completely independent of concrete implementations.

## Data Types

### Core Types (`src/types.rs`)

- **`RawPacket`** - Raw bytes with timestamp from capture source
- **`SequenceInfo`** - Extracted sequence number and flow identifier
- **`AnalyzedPacket`** - Packet with all analysis metadata
- **`FlowId`** - Unique flow identifier (protocol-specific)
  - `MACsec { sci: u64 }` - 8-byte Secure Channel Identifier
  - `IPsec { spi: u32, dst_ip: [u8; 4] }` - Future support
- **`SequenceGap`** - Details about a detected gap
- **`FlowStats`** - Aggregated statistics per flow
- **`AnalysisReport`** - Complete analysis results

## Packet Analysis Process

### MACsec Packet Format

```
Bytes 0-5:     Destination MAC
Bytes 6-11:    Source MAC
Bytes 12-13:   EtherType (0x88E5 for MACsec)
Bytes 14:      TCI/AN (flags)
Bytes 15:      Short Length
Bytes 16-19:   Packet Number (4 bytes, big-endian) ← EXTRACTED
Bytes 20-27:   SCI (8 bytes, big-endian) ← EXTRACTED AS FLOW ID
Bytes 28+:     Encrypted Payload
Last 16:       ICV (Integrity Check Value)
```

### Gap Detection Algorithm

For each packet in a flow:

1. **First packet:** Record as first sequence, set expected = seq + 1
2. **Sequential packet (seq == expected):** Advance expected, no gap
3. **Out-of-order ahead (seq > highest):**
   - Calculate gap = expected - seq
   - Buffer packet for potential reordering
   - Mark highest = seq
4. **Out-of-order behind (seq < highest):**
   - Buffer for gap filling
   - No gap yet (may be retransmit)
5. **Wraparound handling:** Use `wrapping_add()` and `wrapping_sub()` for u32 wraparound

### Reordering Buffer

The `FlowState` maintains a `BTreeMap<u32, AnalyzedPacket>` to handle out-of-order packets:
- Prevents false gap detection for temporarily delayed packets
- Automatically processes sequential packets when they arrive
- Reports only true frame loss after reordering window expires

## Feature Flags

The project uses Cargo features to handle platform dependencies:

```toml
[features]
default = ["pcap-dep"]
pcap-dep = ["pcap"]
```

**Why:** The `pcap` crate requires Windows SDK libraries (wpcap.lib) that may not be available on all systems. The library can be built and tested independently.

**Usage:**
- Build library only: `cargo build --lib` (no dependencies)
- Build with binary: `cargo build --features pcap-dep` (requires wpcap.lib)
- Build without pcap: `cargo build --lib --no-default-features`

## Extending the System

### Adding IPsec Support

Create `src/protocol/ipsec.rs`:

```rust
pub struct IPsecParser;

impl SequenceParser for IPsecParser {
    fn parse_sequence(&self, data: &[u8]) -> Result<Option<SequenceInfo>, ParseError> {
        // Extract SPI and sequence number from ESP header
        // Return FlowId::IPsec { spi, dst_ip }
    }
    // ... implement matches() and protocol_name()
}
```

Then use it:
```rust
let parser = IPsecParser;
let mut analyzer = PacketAnalyzer::new(source, parser);
let report = analyzer.analyze()?;
```

**No changes needed to the capture, analysis, or flow tracking logic!**

### Adding Live Capture Support

Create `src/capture/live.rs`:

```rust
pub struct LiveCapture {
    capture: Capture<pcap::Active>,
    // ...
}

impl PacketSource for LiveCapture {
    fn next_packet(&mut self) -> Result<Option<RawPacket>, CaptureError> {
        // Use pcap::Capture to read from network interface
    }
    // ...
}
```

Then use it:
```rust
let source = LiveCapture::open("eth0")?;
let parser = MACsecParser;
let mut analyzer = PacketAnalyzer::new(source, parser);
let report = analyzer.analyze()?;
```

**Works seamlessly with existing protocol parsers!**

## Design Benefits

### 1. **Separation of Concerns**
Each module has a single responsibility:
- `capture` - Getting bytes off the wire
- `protocol` - Understanding protocol-specific fields
- `analysis` - Analyzing sequences and detecting gaps

### 2. **Testability**
Each module can be tested independently:
- `MACsecParser` - Unit test with hardcoded packet bytes
- `FlowTracker` - Unit test with mock packets
- `PacketAnalyzer` - Integration test with mock source and parser

### 3. **Reusability**
The library can be used in different contexts:
- CLI tool for pcap file analysis
- Library imported by network monitoring systems
- Real-time analysis engine
- Integration with other packet processing pipelines

### 4. **Maintainability**
Clear separation makes changes safe:
- New protocol? Implement `SequenceParser`
- New capture source? Implement `PacketSource`
- New analysis? Create module in `analysis/`
- No impacts on existing code

### 5. **Type Safety**
Rust's type system prevents whole classes of bugs:
- Sequences are `u32`, not generic integers
- FlowIds are enumerated, not strings
- Errors are explicitly handled with `Result<>`
- No null pointers or panics in normal operation

## Performance Characteristics

- **Time Complexity:** O(n) for n packets, O(1) per packet processing
- **Space Complexity:** O(f) for f active flows, O(w) per flow for reorder window
- **Memory:** ~1KB per active flow, minimal overhead

## Future Improvements

1. **Async Support:** Use tokio for real-time capture
2. **Filtering:** Add BPF filter support to live capture
3. **Streaming Analysis:** Stream results instead of collecting all gaps
4. **Compression:** Store only gap summaries instead of full details
5. **Metrics Export:** Prometheus/metrics integration
6. **Configuration:** Load reorder window size and other parameters from config

## Testing

**Module-level tests** are included in each module:
- `src/protocol/macsec.rs::tests` - MACsec parser tests
- `src/analysis/flow.rs::tests` - Flow tracking and gap detection tests
- `src/analysis/mod.rs::tests` - Analyzer integration tests

Run tests with:
```bash
# Build the library (no test binaries)
cargo build --lib

# For running actual tests, you'll need wpcap.lib installed
# or use a Linux/macOS environment where pcap is available
```

## API Examples

### Basic Usage

```rust
use macsec_packet_analyzer::{FileCapture, MACsecParser, PacketAnalyzer};

// Analyze a pcap file
let source = FileCapture::open("traffic.pcap")?;
let parser = MACsecParser;
let mut analyzer = PacketAnalyzer::new(source, parser);
let report = analyzer.analyze()?;

// Print results
for gap in report.gaps {
    println!("Flow {}: Gap at seq {} (expected {})",
        gap.flow_id, gap.received, gap.expected);
}
```

### Library Usage

```rust
use macsec_packet_analyzer::{
    analysis::PacketAnalyzer,
    capture::PacketSource,
    protocol::SequenceParser,
};

// Create custom implementations
struct MyCapture { /* ... */ }
impl PacketSource for MyCapture { /* ... */ }

struct MyParser { /* ... */ }
impl SequenceParser for MyParser { /* ... */ }

// Use with the generic analyzer
let analyzer = PacketAnalyzer::new(MyCapture::new(), MyParser);
let report = analyzer.analyze()?;
```

## Compilation

The project compiles to:
- **Library**: `target/debug/libmacsec_packet_analyzer.rlib`
- **Binary** (with `pcap-dep` feature): `target/debug/macsec_packet_analyzer.exe`

Build commands:
```bash
# Library only (always works)
cargo build --lib

# With binary (requires wpcap.lib)
cargo build --features pcap-dep

# Without optional dependencies
cargo build --lib --no-default-features
```
