# MACsec Packet Analyzer - Quick Start Guide

## What You Have

A fully modular, production-ready Rust packet analyzer for detecting frame loss in network traffic.

## Key Features

✅ Analyzes PCAP files or live network interfaces
✅ Detects packet loss (gaps in packet numbers)
✅ Supports multiple protocols: **MACsec**, **IPsec ESP**, **Generic L3 (TCP/UDP)**
✅ Handles multiple flows (protocol-specific identifiers)
✅ Handles sequence wraparound (32-bit counter)
✅ Reorders out-of-order packets
✅ REST API for querying results
✅ Bandwidth and timing metrics
✅ Async live packet capture
✅ SQLite database persistence
✅ Modular architecture for extensibility

## Module Architecture

### The Three Core Abstractions

1. **`PacketSource`** / **`AsyncPacketSource`** - Where packets come from
   - `FileCapture` - Reads from PCAP files
   - `PcapLiveCapture` - Live capture from network interface (async)

2. **`SequenceParser`** - Extract sequence numbers from packets
   - `MACsecParser` - Parses MACsec packet number field
   - `IPsecParser` - Parses IPsec ESP sequence numbers
   - `GenericL3Parser` - Parses TCP/UDP 5-tuple flows

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

## Live Network Packet Capture (Async)

The `async_live_analyzer` binary provides a complete production-ready pipeline for live packet capture with async processing.

### Building the Live Analyzer

```bash
# Build the async_live_analyzer binary
cargo build --features rest-api --bin async_live_analyzer
```

### Running Live Capture

The binary accepts 4 arguments: interface, protocol, database path, capture method.

```bash
# Analyze MACsec traffic from eth0 and save to database
cargo run --features rest-api --bin async_live_analyzer -- eth0 macsec live.db pcap

# Analyze IPsec traffic
cargo run --features rest-api --bin async_live_analyzer -- eth0 ipsec live.db pcap

# Analyze TCP/UDP flows
cargo run --features rest-api --bin async_live_analyzer -- eth0 generic live.db pcap
```

**Note**: Capture requires root/administrator privileges.

### Features of Live Analyzer

- **Async Packet Processing**: Non-blocking packet reception with tokio
- **Periodic Persistence**: Flushes statistics every 5 seconds or 10k packets
- **Real-time Progress**: Displays packets/sec, gap count, flow count
- **Graceful Shutdown**: Ctrl+C saves all data and prints final report
- **Bandwidth Calculations**: Shows Mbps per flow
- **Database Integration**: Stores all stats in SQLite for REST API queries

### Example Output

```
Starting async packet analyzer
  Interface: eth0
  Protocol: macsec
  Database: live.db
  Capture: pcap

PCAP capture started on interface 'eth0'
Press Ctrl+C to stop and save results

[12.3s] Packets: 125000, Gaps: 45, Flows: 3, Rate: 10163 pps
[24.5s] Packets: 250000, Gaps: 87, Flows: 5, Rate: 10204 pps

Shutdown signal received. Flushing data...
Saving final statistics...

=== Analysis Complete ===
Total packets analyzed: 287451
Total gaps detected: 125
Elapsed time: 28.23s
Packet rate: 10184 pps

Flows analyzed: 7

Flow ID                                            Packets           Bytes            Gaps      Bandwidth
----------------------------------------------  ---------------  ---------------  ---------------  ---------------
MACsec { sci: 0x001122334455 }                        51234       26234000           25      7.43 Mbps
MACsec { sci: 0xaabbccddeeff01 }                      48912       24500000           48      6.92 Mbps
...

Results saved to database. Query with:
  cargo run --features rest-api --bin api_server
```

### Querying Results via REST API

After running the live analyzer, start the API server:

```bash
cargo run --features rest-api --bin api_server
```

Then query the results:

```bash
# Get summary statistics
curl http://localhost:8080/api/v1/stats/summary

# List all flows with bandwidth
curl "http://localhost:8080/api/v1/flows?limit=10&min_bandwidth_mbps=5"

# Get specific flow details
curl "http://localhost:8080/api/v1/flows/MACsec%20%7B%20sci:%200x001122334455%20%7D"

# Get sequence gaps for a flow
curl "http://localhost:8080/api/v1/flows/MACsec%20%7B%20sci:%200x001122334455%20%7D/gaps?limit=20"
```

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

## Testing the Live Capture

### Prerequisites

You need libpcap installed:

```bash
# Ubuntu/Debian
sudo apt-get install libpcap-dev

# macOS
brew install libpcap

# Fedora/RHEL
sudo dnf install libpcap-devel
```

### Test Methods

#### 1. Generate Test Traffic (Local Loopback)

```bash
# Terminal 1: Start listening on lo (loopback interface)
sudo cargo run --features rest-api --bin async_live_analyzer -- lo generic test_lo.db pcap

# Terminal 2: Generate TCP traffic
for i in {1..1000}; do
  curl http://localhost:9999 2>/dev/null || true
done
```

#### 2. Replay PCAP File to Virtual Interface

```bash
# If you have a PCAP file, replay it to a virtual interface
tcpreplay -i eth0 sample.pcap

# Meanwhile, capture with analyzer
sudo cargo run --features rest-api --bin async_live_analyzer -- eth0 macsec capture.db pcap
```

#### 3. Monitor Real Network Interface

```bash
# Monitor MACsec traffic on a real interface
sudo cargo run --features rest-api --bin async_live_analyzer -- eth0 macsec live.db pcap

# In another terminal, generate traffic on that network
ping 192.168.1.1  # or other target
```

#### 4. Unit Tests

```bash
# Run all unit tests
cargo test --features rest-api --lib

# Run tests for specific module
cargo test --features rest-api --lib analysis::flow
cargo test --features rest-api --lib protocol::macsec
cargo test --features rest-api --lib protocol::ipsec
```

### Verifying Results

After running the analyzer:

```bash
# 1. Check the database was created
ls -lh live.db

# 2. Query the database directly with sqlite3
sqlite3 live.db "SELECT flow_id, packets_received, total_bytes FROM flows LIMIT 5"

# 3. Start the API server
cargo run --features rest-api --bin api_server

# 4. Query via REST (in another terminal)
curl -s http://localhost:8080/api/v1/stats/summary | jq .

# 5. Check flow details with bandwidth
curl -s "http://localhost:8080/api/v1/flows?limit=5" | jq '.flows[] | {flow_id, bandwidth_mbps, packets_received}'
```

### Troubleshooting Live Capture

**Permission Denied**: You need root/admin privileges
```bash
# Linux/macOS
sudo cargo run --features rest-api --bin async_live_analyzer -- eth0 generic test.db pcap

# Windows (run as Administrator in PowerShell)
cargo run --features rest-api --bin async_live_analyzer -- eth0 generic test.db pcap
```

**No packets captured**: Check the interface name
```bash
# List available interfaces
ip link show                  # Linux
ifconfig                      # macOS/Linux
ipconfig                      # Windows
netsh interface show interface  # Windows

# Verify interface is up
ip link show eth0
```

**Permission denied for database**: Run from a writable directory
```bash
cd /tmp
sudo cargo run --features rest-api --bin async_live_analyzer -- eth0 generic live.db pcap
```

**libpcap not found**: Install development libraries
```bash
# Ubuntu
sudo apt-get install libpcap-dev

# macOS
brew install libpcap

# Fedora
sudo dnf install libpcap-devel
```

## Next Steps

1. ✅ Build the library: `cargo build --lib`
2. ✅ Review the architecture: See `ARCHITECTURE.md`
3. ✅ Analyze PCAP files: `cargo run --features cli -- file.pcap`
4. ✅ Capture live traffic: `sudo cargo run --features rest-api --bin async_live_analyzer -- eth0 macsec live.db pcap`
5. ✅ Query results: `cargo run --features rest-api --bin api_server`
6. 🧪 Write custom analysis (extend `FlowTracker`)
7. 📊 Integrate with monitoring systems (use REST API)
8. 📦 Consider making this a published crate

Enjoy your modular packet analyzer! 🚀
