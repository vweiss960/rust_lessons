# MACsec Packet Analyzer - Quick Start Guide

## What You Have

A fully modular, production-ready Rust packet analyzer for detecting frame loss in MACsec traffic.

## Key Features

✅ Reads PCAP files
✅ Detects packet loss (gaps in packet numbers)
✅ Handles multiple flows (by Secure Channel Identifier)
✅ Handles sequence wraparound (32-bit counter)
✅ Reorders out-of-order packets
✅ Modular architecture for extensibility

## Module Architecture

### The Three Core Abstractions

1. **`PacketSource`** - Where packets come from
   - `FileCapture` - Reads from PCAP files
   - Future: `LiveCapture` - Read from network interface

2. **`SequenceParser`** - Extract sequence numbers from packets
   - `MACsecParser` - Parses MACsec packet number field
   - Future: `IPsecParser` - Parse IPsec sequence numbers

3. **`PacketAnalyzer`** - Orchestrates analysis
   - Generic over `PacketSource` and `SequenceParser`
   - Detects gaps in sequences
   - Generates reports

### Usage Pattern

```rust
// 1. Create a source
let source = FileCapture::open("traffic.pcap")?;

// 2. Create a parser
let parser = MACsecParser;

// 3. Create analyzer (generic over source + parser)
let mut analyzer = PacketAnalyzer::new(source, parser);

// 4. Analyze
let report = analyzer.analyze()?;

// 5. Use results
for gap in report.gaps {
    println!("Gap: expected {}, got {}", gap.expected, gap.received);
}
```

## File Structure

```
src/
├── types.rs              # Common data types
├── error.rs              # Error types
├── capture/
│   ├── source.rs         # PacketSource trait
│   └── file.rs           # FileCapture implementation
├── protocol/
│   ├── parser.rs         # SequenceParser trait
│   └── macsec.rs         # MACsecParser implementation
├── analysis/
│   ├── mod.rs            # PacketAnalyzer orchestrator
│   └── flow.rs           # Flow tracking + gap detection
├── lib.rs                # Library exports
└── main.rs               # CLI tool
```

## Building

```bash
# Build library (always works)
cargo build --lib

# Build with CLI tool (requires wpcap.lib on Windows)
cargo build --features cli
```

## Running the CLI

```bash
# With default pcap file (macsec_traffic.pcap)
cargo run --features cli

# With custom pcap file
cargo run --features cli -- /path/to/file.pcap
```

## Output Format

```
Analyzing MACsec packets from: macsec_traffic.pcap

Analysis Report:
================
Total packets processed: 200
Protocol: MACsec
Flows detected: 2

Flow: MACsec { sci: 0x001122334455 }
  Packets received: 100
  Gaps detected: 0
  Lost packets (due to gaps): 0
  Sequence range: 1 - 100

Flow: MACsec { sci: 0xaabbccddeeff01 }
  Packets received: 95
  Gaps detected: 5
  Lost packets (due to gaps): 5
  Sequence range: 1 - 100
  Min gap size: 1
  Max gap size: 1

Gaps Detected:
==============
  Gap 1: Flow MACsec { sci: 0xaabbccddeeff01 } - Expected seq 16, received 17 (gap size: 1)
  Gap 2: Flow MACsec { sci: 0xaabbccddeeff01 } - Expected seq 32, received 33 (gap size: 1)
  ...
```

## Adding IPsec Support

### Step 1: Create parser (`src/protocol/ipsec.rs`)

```rust
use byteorder::{BigEndian, ByteOrder};
use crate::types::{FlowId, SequenceInfo};
use crate::error::ParseError;
use super::parser::SequenceParser;

pub struct IPsecParser;

impl SequenceParser for IPsecParser {
    fn parse_sequence(&self, data: &[u8]) -> Result<Option<SequenceInfo>, ParseError> {
        if !self.matches(data) {
            return Ok(None);
        }

        // Extract sequence number from ESP header
        // (at offset after SPI in ESP packet)
        let spi = BigEndian::read_u32(&data[offset..offset+4]);
        let seq_num = BigEndian::read_u32(&data[offset+4..offset+8]);
        let dst_ip = [data[30], data[31], data[32], data[33]];

        Ok(Some(SequenceInfo {
            sequence_number: seq_num,
            flow_id: FlowId::IPsec { spi, dst_ip },
            payload_length: data.len() - offset - 8,
        }))
    }

    fn matches(&self, data: &[u8]) -> bool {
        // Check for IPv4 + ESP protocol (50)
        data.len() > 34 && data[12] == 0x08 && data[13] == 0x00 && data[23] == 50
    }

    fn protocol_name(&self) -> &str {
        "IPsec-ESP"
    }
}
```

### Step 2: Export it (`src/protocol/mod.rs`)

```rust
pub mod ipsec;
pub use ipsec::IPsecParser;
```

### Step 3: Use it

```rust
let source = FileCapture::open("traffic.pcap")?;
let parser = IPsecParser;  // ← Just change this line!
let mut analyzer = PacketAnalyzer::new(source, parser);
let report = analyzer.analyze()?;
```

**That's it!** No changes to capture, analysis, or flow tracking needed.

## Adding Live Capture Support

### Step 1: Create capture source (`src/capture/live.rs`)

```rust
use pcap::Capture;
use crate::types::{CaptureStats, RawPacket};
use crate::error::CaptureError;
use super::source::PacketSource;

pub struct LiveCapture {
    capture: Capture<pcap::Active>,
    packets_read: u64,
}

impl LiveCapture {
    pub fn open(interface: &str) -> Result<Self, CaptureError> {
        let capture = Capture::from_device(interface)
            .map_err(|e| CaptureError::OpenFailed(e.to_string()))?
            .promisc(true)
            .snaplen(65535)
            .open()
            .map_err(|e| CaptureError::OpenFailed(e.to_string()))?;

        Ok(Self {
            capture,
            packets_read: 0,
        })
    }
}

impl PacketSource for LiveCapture {
    fn next_packet(&mut self) -> Result<Option<RawPacket>, CaptureError> {
        // Similar to FileCapture implementation
        // Returns packets from network interface
    }

    fn stats(&self) -> CaptureStats {
        let stats = self.capture.stats().unwrap_or_default();
        CaptureStats {
            packets_received: stats.received as u64,
            packets_dropped: stats.dropped as u64,
        }
    }
}
```

### Step 2: Export it (`src/capture/mod.rs`)

```rust
pub mod live;
pub use live::LiveCapture;
```

### Step 3: Use it

```rust
let source = LiveCapture::open("eth0")?;
let parser = MACsecParser;
let mut analyzer = PacketAnalyzer::new(source, parser);
let report = analyzer.analyze()?;
```

**Works with any protocol parser!**

## Key Design Principles

### 1. Trait-Based Abstraction
Everything important is a trait:
- Packets can come from anywhere (trait `PacketSource`)
- Sequences are extracted protocol-independently (trait `SequenceParser`)
- Analyzer is generic over both

### 2. Separation of Concerns
- `capture/` - Just get bytes
- `protocol/` - Just extract fields
- `analysis/` - Just detect gaps
- Each module independent and testable

### 3. Data-Driven, Not Logic-Heavy
- Types encode what's important (`FlowId`, `SequenceGap`)
- Analyzer just orchestrates the types
- Flow tracker manages per-flow state

### 4. No Generics in Core
The core analysis (flow tracking) is not generic:
- Works with any protocol through `FlowId` enum
- Works with any source through abstraction
- Easy to understand and maintain

## Testing the Existing Implementation

Unit tests exist in each module:

```rust
// In macsec.rs
#[test]
fn test_macsec_parser_valid_packet() { ... }

// In flow.rs
#[test]
fn test_gap_detection() { ... }
#[test]
fn test_wraparound_detection() { ... }

// In analysis/mod.rs
#[test]
fn test_analyzer_with_gaps() { ... }
```

Note: Test binaries require linking, which needs wpcap.lib on Windows.

## Common Patterns

### Process Entire File
```rust
let source = FileCapture::open("file.pcap")?;
let parser = MACsecParser;
let mut analyzer = PacketAnalyzer::new(source, parser);
let report = analyzer.analyze()?;
```

### Multiple Formats
```rust
let parser = match format {
    "macsec" => MACsecParser as Box<dyn SequenceParser>,
    "ipsec" => IPsecParser as Box<dyn SequenceParser>,
    _ => return Err("Unknown format"),
};

let source = FileCapture::open(file)?;
let mut analyzer = PacketAnalyzer::new(source, *parser);
```

### Custom Processing
```rust
let mut source = FileCapture::open("file.pcap")?;
let parser = MACsecParser;
let mut tracker = FlowTracker::new();

while let Some(raw) = source.next_packet()? {
    if let Some(seq_info) = parser.parse_sequence(&raw.data)? {
        let analyzed = AnalyzedPacket {
            sequence_number: seq_info.sequence_number,
            flow_id: seq_info.flow_id,
            timestamp: raw.timestamp,
            payload_length: seq_info.payload_length,
        };

        if let Some(gap) = tracker.process_packet(analyzed) {
            println!("Gap detected: {:?}", gap);
        }
    }
}
```

## Error Handling

All operations return `Result`:

```rust
match FileCapture::open("file.pcap") {
    Ok(source) => { /* ... */ },
    Err(CaptureError::OpenFailed(msg)) => eprintln!("Open failed: {}", msg),
    Err(e) => eprintln!("Capture error: {}", e),
}

match analyzer.analyze() {
    Ok(report) => { /* Process report */ },
    Err(AnalysisError::Capture(e)) => eprintln!("Capture error: {}", e),
    Err(AnalysisError::Parse(e)) => eprintln!("Parse error: {}", e),
}
```

## Performance Notes

- **Single-pass analysis** - O(n) time for n packets
- **Per-flow tracking** - O(f) space for f active flows
- **No allocations in hot path** - Each packet processed in constant time
- **Reorder window** - Bounded by configurable window size (default 32)

## When to Use This

✅ Analyzing recorded pcap files
✅ Implementing custom network monitoring
✅ Testing network security appliances
✅ Detecting frame loss in MACsec deployments
✅ Learning Rust patterns (traits, generics, error handling)

## When to Extend This

🔧 Support new protocols (implement `SequenceParser`)
🔧 Read from different sources (implement `PacketSource`)
🔧 Add new analysis types (extend `analysis/` module)
🔧 Customize reporting (extend `AnalysisReport`)

## Architecture Summary

```
Main Entry Point
       ↓
   Analyzer (generic)
       ↓
  ┌────┴────┐
  ↓         ↓
Source    Parser
  ↓         ↓
FileCapture  MACsecParser
  ↓           ↓
   PCAP File  Packet Bytes
  ↓           ↓
   Raw Data ──→ SequenceInfo
        ↓
     Flow Tracker
        ↓
    Gap Detection
        ↓
     Analysis Report
```

## Next Steps

1. ✅ Build the library: `cargo build --lib`
2. ✅ Review the architecture: See `ARCHITECTURE.md`
3. 📝 Add IPsec support using the pattern above
4. 🌐 Add live capture support using the pattern above
5. 🧪 Write integration tests
6. 📦 Consider making this a published crate

Enjoy your modular packet analyzer! 🚀
