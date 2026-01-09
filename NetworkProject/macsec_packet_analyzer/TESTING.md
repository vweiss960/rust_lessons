# Testing Guide for MACsec Packet Analyzer

This guide explains how to use the test suite to run and test the `live_analyzer` and `rest_api_server` executables together.

## Quick Start (30 seconds)

The easiest way to test is using the provided test script:

```bash
./test_analyzer.sh
```

This will:
1. Build both binaries in debug mode
2. Clean old test databases
3. Start `live_analyzer` capturing from the loopback interface (`lo`)
4. Start `rest_api_server` on port 3000
5. Both will use a single test database: `test_analysis.db`

Both servers are now running and ready to use.

## Test Script Options

### Basic Usage

```bash
./test_analyzer.sh [OPTIONS]
```

### Available Options

| Option | Description | Example |
|--------|-------------|---------|
| `-i, --interface IFACE` | Network interface to capture from | `./test_analyzer.sh -i eth0` |
| `-d, --database PATH` | Path to test database | `./test_analyzer.sh -d ./my_test.db` |
| `-p, --port PORT` | REST API server port | `./test_analyzer.sh -p 8080` |
| `--debug` | Enable debug output for live_analyzer | `./test_analyzer.sh --debug` |
| `--release` | Build in release mode | `./test_analyzer.sh --release` |
| `-h, --help` | Show help message | `./test_analyzer.sh -h` |

## Common Test Scenarios

### Scenario 1: Debug Mode on Loopback (Default)

Perfect for quick testing without needing special network interfaces:

```bash
./test_analyzer.sh --debug
```

**Output:**
- Shows startup information from `live_analyzer`
- Displays periodic statistics (packet count, gaps, cache hit rate)
- REST API available at `http://localhost:3000`

### Scenario 2: Release Build with Custom Interface

For testing with actual network traffic:

```bash
./test_analyzer.sh -i eth0 --release
```

**Features:**
- Optimized release binary
- Captures from `eth0` interface
- Quiet mode (no debug output)
- Better performance

### Scenario 3: Custom Port with Debug

When port 3000 is already in use:

```bash
./test_analyzer.sh -p 8080 --debug
```

**Configuration:**
- API endpoints on port 8080 instead of 3000
- Debug output enabled
- Test database at `./test_analysis.db`

### Scenario 4: Specific Database Location

For organizing test runs:

```bash
./test_analyzer.sh -d ./test_run_1.db --release
```

**Result:**
- Database stored at `./test_run_1.db`
- Automatically cleaned and initialized
- All results preserved after script exits

## What Happens When You Run the Script

### Build Phase
```
[1/4] Cleaning old test databases...
✓ Old databases cleaned

[2/4] Building live_analyzer (debug mode)...
✓ live_analyzer built successfully

[3/4] Building rest_api_server (debug mode)...
✓ rest_api_server built successfully
```

### Startup Phase
```
[4/4] Starting servers...
► Starting live_analyzer...
✓ live_analyzer started (PID: 12345)

► Starting rest_api_server...
✓ rest_api_server started (PID: 12346)
```

### Running Phase
Both servers run in the background, writing to the same database:
- **live_analyzer**: Captures packets, detects protocols, tracks flows, writes every 5 seconds
- **rest_api_server**: Serves the captured data via REST API

### Shutdown Phase
Press `Ctrl+C` to stop both servers gracefully:
```
════════════════════════════════════════════════════════════
Shutting down servers...
✓ Servers stopped

Database preserved at: ./test_analysis.db
Query with: ./target/debug/rest_api_server --db ./test_analysis.db --port 3000
```

## Testing the REST API

While the servers are running, you can test the API endpoints:

### Health Check
```bash
curl http://localhost:3000/health
```

### Get Summary Statistics
```bash
curl http://localhost:3000/api/v1/stats/summary | jq
```

### List All Flows
```bash
curl http://localhost:3000/api/v1/flows | jq
```

### Get Details for Specific Flow
```bash
curl "http://localhost:3000/api/v1/flows/<flow_id>" | jq
```

### Get Gaps for a Specific Flow
```bash
curl "http://localhost:3000/api/v1/flows/<flow_id>/gaps" | jq
```

## Multiple Test Runs

You can run multiple tests with different configurations in separate terminals:

**Terminal 1: Test with PCAP backend**
```bash
./test_analyzer.sh -d ./test_pcap.db --debug
```

**Terminal 2: Test with different interface (in another window)**
```bash
./test_analyzer.sh -i eth0 -d ./test_eth0.db --release
```

Each will use its own database and won't interfere with the other.

## Troubleshooting

### Script fails to build
- Make sure you're in the project root directory
- Check that Rust is installed: `rustc --version`
- Try cleaning: `cargo clean && ./test_analyzer.sh`

### Can't access API endpoints
- Verify the server started (check for "rest_api_server started" message)
- Check the port is correct: `lsof -i :3000` (or whatever port you specified)
- Try the health endpoint first: `curl http://localhost:3000/health`

### Permission denied error
- Make sure script is executable: `chmod +x test_analyzer.sh`
- Check you have permissions to write databases in the target location

### Network interface doesn't exist
- List available interfaces: `ip link show` or `ifconfig`
- Use `lo` (loopback) for testing without special privileges
- Use `eth0`, `en0`, `wlan0` etc. for testing with actual network traffic

## Advanced Usage

### Building Only (Without Running)
```bash
cargo build --bin live_analyzer --bin rest_api_server --release
```

### Running Binaries Manually
```bash
# Terminal 1: Start live_analyzer
./target/release/live_analyzer lo ./my_test.db --debug

# Terminal 2: Start rest_api_server
./target/release/rest_api_server --db ./my_test.db --port 3000
```

### Cleaning Up
```bash
# Remove all test databases
rm -f *.db *.db-* ./target/debug/*.db ./target/release/*.db

# Clean build artifacts
cargo clean
```

## Understanding the Database

The test database (`test_analysis.db`) is a SQLite database containing:

- **Flow statistics**: Per-flow packet counts, gap counts, bandwidth
- **Gap records**: Detailed information about detected sequence gaps
- **Packet metadata**: Timestamps, sequence numbers, flow IDs

After the script exits, the database is preserved, allowing you to:
- Re-run the REST API server to query results
- Export data for analysis
- Compare results between test runs

### Query Database Directly

```bash
sqlite3 ./test_analysis.db

# Inside sqlite3:
.tables                          # Show all tables
.schema flows                    # Show flows table structure
SELECT * FROM flows LIMIT 5;     # View first 5 flows
SELECT COUNT(*) FROM gaps;       # Count total gaps detected
```

## Performance Testing

For performance testing with release builds:

```bash
# Test with high packet rates on eth0
./test_analyzer.sh -i eth0 --release

# In another terminal, monitor system stats
watch -n 1 'ps aux | grep live_analyzer | grep -v grep'
```

The REST API can show real-time performance metrics:

```bash
while true; do
  curl -s http://localhost:3000/api/v1/stats/summary | \
    jq '.total_packets_received, .total_gaps_detected'
  sleep 1
done
```

## Detailed Testing Guide

### System Prerequisites

#### System Requirements
- **Linux**, **macOS**, or **Windows** (tested on Linux primarily)
- **Rust** 1.70+ (for async/await support)
- **libpcap** development library
- **root** or **Administrator** privileges for packet capture

#### Install libpcap

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install libpcap-dev

# CentOS/RHEL/Fedora
sudo dnf install libpcap-devel

# macOS
brew install libpcap

# Arch
sudo pacman -S libpcap
```

#### Install Optional Tools (for testing)

```bash
# For traffic generation and analysis
sudo apt-get install curl wget tcpdump iperf3 ncat

# For JSON query (optional but helpful)
sudo apt-get install jq

# For PCAP replay (if you have PCAP files)
sudo apt-get install tcpreplay
```

### Build Instructions

#### Compile the Binary

```bash
cd /path/to/macsec_packet_analyzer

# Build with all features
cargo build --bin live_analyzer --release

# For testing, use debug build (faster to compile)
cargo build --bin live_analyzer
```

#### Verify Build

```bash
# Check binary exists
ls -lh target/debug/live_analyzer

# Test help message
cargo run --bin live_analyzer -- --help 2>&1 || true
```

---

## Test Scenarios

### Scenario 1: Local Loopback Interface (Default)

**Best for**: Quick testing without needing network setup.

```bash
# Terminal 1: Start the analyzer on lo
./test_analyzer.sh --debug
```

**Output:**
- Shows startup information from `live_analyzer`
- Displays periodic statistics (packet count, gaps, cache hit rate)
- REST API available at `http://localhost:3000`

### Scenario 2: Real Network Interface with Custom Options

**Best for**: Testing with actual network traffic.

```bash
# Terminal 1: Test on eth0 with release build
./test_analyzer.sh -i eth0 --release
```

**Features:**
- Optimized release binary
- Captures from `eth0` interface
- Quiet mode (no debug output)
- Better performance

### Scenario 3: Custom Port and Debug

When port 3000 is already in use:

```bash
./test_analyzer.sh -p 8080 --debug
```

**Configuration:**
- API endpoints on port 8080 instead of 3000
- Debug output enabled
- Test database at `./test_analysis.db`

### Scenario 4: Specific Database Location

For organizing test runs:

```bash
./test_analyzer.sh -d ./test_run_1.db --release
```

**Result:**
- Database stored at `./test_run_1.db`
- Automatically cleaned and initialized
- All results preserved after script exits

### Scenario 5: Generate Test Traffic (Local Loopback)

**Setup:**

```bash
# Terminal 1: Start the analyzer on lo
sudo cargo run --bin live_analyzer -- lo test_lo.db pcap
```

**Generate Traffic (Terminal 2):**

```bash
for i in {1..1000}; do
  timeout 1 curl http://localhost:8765 2>/dev/null || true
  sleep 0.01
done

# Or use a simpler loop
for i in {1..500}; do
  echo "test" | nc -q 1 127.0.0.1 8765 2>/dev/null || true
done
```

**Expected Results:**

In Terminal 1:
```
[5.2s] Packets: 1024, Gaps: 0, Flows: 2, Rate: 197 pps
[10.4s] Packets: 2048, Gaps: 0, Flows: 4, Rate: 197 pps
```

Press Ctrl+C to stop and see final report.

### Scenario 6: Real Network Interface with Traffic Generation

**Best for**: Testing with actual network traffic, verifying bandwidth calculations.

**Identify Your Interface:**

```bash
ip addr show
# Pick a common one (e.g., eth0, wlan0, ens33)
```

**Start Analyzer (Terminal 1):**

```bash
sudo cargo run --bin live_analyzer -- eth0 generic capture_eth0.db pcap
```

**Generate Traffic (Terminal 2):**

```bash
# Generate HTTP traffic
for i in {1..50}; do
  curl http://example.com > /dev/null 2>&1 &
done

# Or use iperf3 for sustained traffic
iperf3 -s -p 5201 &

# In another terminal:
iperf3 -c 192.168.1.x -p 5201 -t 30 -P 4
```

**Monitor Progress:**

```
[30.5s] Packets: 15234, Gaps: 0, Flows: 8, Rate: 500 pps
[60.8s] Packets: 30512, Gaps: 2, Flows: 12, Rate: 501 pps
```

**Stop and Check Results:**

Press Ctrl+C and check the final report.

### Scenario 7: PCAP File Replay

**Best for**: Reproducing specific packet sequences, testing with known datasets.

**Create or Find a PCAP File:**

```bash
# Option A: Create a simple PCAP with tcpdump
sudo tcpdump -i eth0 -c 100 -w test_traffic.pcap

# Option B: Use a public PCAP sample
# Download from https://www.wireshark.org/download/sample-captures/
```

**Replay the PCAP:**

```bash
# Terminal 1: Start analyzer before replay
sudo cargo run --bin live_analyzer -- eth0 generic pcap_test.db pcap

# Terminal 2: Replay the PCAP after analyzer starts
sudo tcpreplay -i eth0 test_traffic.pcap
```

**Verify:**

```bash
sqlite3 pcap_test.db "SELECT COUNT(*) FROM flows"
sqlite3 pcap_test.db "SELECT flow_id, packets_received FROM flows"
```

### Scenario 8: Protocol-Specific Testing

**Testing with MACsec Traffic:**

```bash
# Requires MACsec-enabled interface
sudo cargo run --bin live_analyzer -- macsec_eth0 test.db pcap
# Monitor output for MACsec-specific flows
```

**Testing with IPsec Traffic:**

```bash
# First, establish an IPsec connection (VPN)
# Then monitor:
sudo cargo run --bin live_analyzer -- eth0 ipsec ipsec_test.db pcap
# Will detect ESP (Encapsulating Security Payload) packets
```

**Testing with Generic L3 (TCP/UDP) Traffic:**

```bash
sudo cargo run --bin live_analyzer -- eth0 generic generic_test.db pcap

# Generate traffic in another terminal:
curl http://example.com &
ping google.com &
```

**Note**: Gap detection is DISABLED for Generic L3 flows (TCP/UDP) because TCP sequence numbers track cumulative bytes, not packets, and permit retransmissions and out-of-order delivery. For TCP/UDP flows, use packet count and bandwidth metrics instead.

### Scenario 9: Unit Tests

**Run All Tests:**

```bash
cargo test --lib

# Expected output:
running 42 tests
test analysis::flow::tests::test_gap_detection ... ok
test analysis::flow::tests::test_wraparound ... ok
test protocol::macsec::tests::test_parse_valid ... ok
...

test result: ok. 42 passed; 0 failed; 0 ignored
```

**Test Specific Modules:**

```bash
# Test flow tracking (gap detection)
cargo test --lib analysis::flow --

# Test MACsec parser
cargo test --lib protocol::macsec --

# Test IPsec parser
cargo test --lib protocol::ipsec --

# Test Generic L3 parser
cargo test --lib protocol::generic_l3 --

# Test database operations
cargo test --lib db --
```

**Run Specific Test:**

```bash
# Run one test by name
cargo test --lib test_gap_detection -- --exact

# Run tests matching a pattern
cargo test --lib gap --
```

### Scenario 10: REST API Integration

**Setup:**

```bash
# Terminal 1: Run analyzer for a bit
sudo cargo run --bin live_analyzer -- eth0 generic api_test.db pcap

# Generate traffic while running (see earlier scenarios)
# Press Ctrl+C after 30 seconds

# Terminal 2: Start API server
cargo run --bin rest_api_server -- --db api_test.db
```

**Test API Endpoints:**

```bash
# Health check
curl http://localhost:3000/health

# Summary statistics
curl -s http://localhost:3000/api/v1/stats/summary | jq .

# List flows
curl -s http://localhost:3000/api/v1/flows | jq '.flows | length'

# Get specific flow (URL encode the flow_id)
FLOW_ID=$(curl -s http://localhost:3000/api/v1/flows | jq -r '.flows[0].flow_id')
curl -s "http://localhost:3000/api/v1/flows/$(echo -n "$FLOW_ID" | jq -sRr @uri)" | jq .

# Get gaps for a flow
curl -s "http://localhost:3000/api/v1/flows/$(echo -n "$FLOW_ID" | jq -sRr @uri)/gaps" | jq .

# Filter by bandwidth (Mbps)
curl -s "http://localhost:3000/api/v1/flows?min_bandwidth_mbps=1" | jq '.flows[] | {flow_id, bandwidth_mbps}'

# Filter by packet size
curl -s "http://localhost:3000/api/v1/flows?min_bytes=10000" | jq '.flows[] | {flow_id, total_bytes}'
```

**Verify Response Structure:**

```bash
# Check summary response has bandwidth
curl -s http://localhost:3000/api/v1/stats/summary | jq 'keys'
# Should output keys like: ["avg_bandwidth_mbps", "max_gap_size", "total_bytes", ...]

# Check flow response has all metrics
curl -s http://localhost:3000/api/v1/flows | jq '.flows[0] | keys'
# Should include: ["avg_inter_arrival_ms", "bandwidth_mbps", "duration_seconds", ...]
```

### Scenario 11: Performance Testing

**Measure Packet Rate:**

```bash
sudo cargo run --bin live_analyzer -- eth0 generic perf_test.db pcap

# Expected output:
[10.0s] Packets: 50000, Gaps: 0, Flows: 20, Rate: 5000 pps
[20.0s] Packets: 100000, Gaps: 5, Flows: 35, Rate: 5000 pps
```

**Generate High-Rate Traffic:**

```bash
# Using iperf3 (generates sustained traffic)
iperf3 -s &                                    # Server
iperf3 -c localhost -t 60 -P 10 -b 1G         # 10 parallel streams, 1Gbps

# Using netperf (another option)
netserver &
netperf -H localhost -l 60 -P 0 &
netperf -H localhost -l 60 -P 0 &
```

**Benchmark Results:**

Typical performance on commodity hardware:
- **Packet Rate**: 5,000 - 50,000 pps depending on CPU
- **Memory Usage**: ~100-200 MB for 10,000 concurrent flows
- **Database Write**: 5-10 MB per minute of capture
- **Overhead**: < 10% CPU per 10k pps

### Scenario 12: Multiple Test Runs

You can run multiple tests with different configurations in separate terminals:

**Terminal 1: Test with PCAP backend**
```bash
./test_analyzer.sh -d ./test_pcap.db --debug
```

**Terminal 2: Test with different interface (in another window)**
```bash
./test_analyzer.sh -i eth0 -d ./test_eth0.db --release
```

Each will use its own database and won't interfere with the other.

### Scenario 13: PCAP Replay Stress Testing

**Best for**: High-rate stress testing without live traffic, reproducing specific packet sequences, validating gap detection with known patterns.

The PCAP replay feature allows you to load a PCAP file and replay packets at controlled rates with multiple timing modes.

#### Quick Start - Fast Replay

Replay a PCAP file at maximum throughput for CPU-intensive stress testing:

```bash
./test_analyzer.sh --replay fast ./macsec_traffic.pcap -d ./replay_fast.db --debug
```

**Expected Output:**
- Loads PCAP file into memory at startup
- Replays packets as fast as possible (no delays)
- Typical throughput: 100K - 1M packets/sec depending on CPU
- Progress displayed every 5 seconds

#### Replay Mode: Original Timing

Replay packets respecting original inter-packet delays from the PCAP file:

```bash
./test_analyzer.sh --replay original ./macsec_traffic.pcap -d ./replay_original.db --debug
```

**Use Case**: Realistic traffic pattern replay

**Behavior:**
- Maintains original packet spacing from PCAP capture
- Useful for testing packet processing under realistic conditions
- If original PCAP had 1ms inter-packet delays, replay respects that

#### Replay Mode: Fixed Rate (Packets Per Second)

Replay at a fixed, controllable packet rate:

```bash
./test_analyzer.sh --replay fixed ./macsec_traffic.pcap --replay-pps 1000 -d ./replay_fixed.db --debug
```

**Use Case**: Bandwidth and load testing with specific rates

**Configuration Options:**
- `--replay-pps 100` - 100 packets/sec (minimal load)
- `--replay-pps 10000` - 10K packets/sec (moderate stress)
- `--replay-pps 100000` - 100K packets/sec (high stress)

**Example - Moderate Stress Test:**
```bash
./test_analyzer.sh --replay fixed ./macsec_traffic.pcap --replay-pps 50000 -d ./replay_50k.db --release
```

Expected: ~50,000 packets/sec, sustained indefinitely

#### Replay Mode: Speed Multiplier

Accelerate or decelerate the original PCAP timing:

```bash
# 10x faster than original
./test_analyzer.sh --replay speed ./macsec_traffic.pcap --replay-speed 10.0 -d ./replay_10x.db --debug

# Half speed (slower)
./test_analyzer.sh --replay speed ./macsec_traffic.pcap --replay-speed 0.5 -d ./replay_half.db --debug
```

**Use Cases:**
- `10.0` - Accelerated testing for quick validation
- `0.5` - Slowed-down replay for debugging timing issues
- `100.0` - Ultra-fast stress testing (if original PCAP has long delays)

#### Looping for Sustained Stress Testing

Enable infinite looping to continuously replay a PCAP file:

```bash
# Fast mode with looping for 1 hour
timeout 3600 ./test_analyzer.sh --replay fast ./macsec_traffic.pcap --replay-loop -d ./replay_stress.db --release

# Or let it run indefinitely (Ctrl+C to stop)
./test_analyzer.sh --replay fast ./macsec_traffic.pcap --replay-loop -d ./replay_sustained.db
```

**Important**: Looping causes sequence/packet numbers to reset, which temporarily creates artificial gaps. These gaps are expected and indicate loop wraparound, not genuine packet loss.

**Database Query to Verify Looping:**
```bash
sqlite3 ./replay_stress.db "SELECT COUNT(DISTINCT flow_id) FROM flows;"
```

#### Complete Examples

**Example 1: MACsec Stress Test (Fast Mode)**
```bash
./test_analyzer.sh --replay fast ./macsec_traffic.pcap -d ./macsec_stress.db --debug
```

**Expected Results:**
- High packet rate (100K+ pps)
- MACsec flows detected (flow_id contains "MACsec")
- If PCAP has intentional gaps, they appear in `gaps` table

**Example 2: Sustained Load Testing (Fixed Rate, Looping)**
```bash
# 50 minute sustained test at 10K packets/sec
timeout 3000 ./test_analyzer.sh --replay fixed ./macsec_traffic.pcap --replay-pps 10000 --replay-loop \
  -d ./sustained_10k.db --release
```

**Verify Results:**
```bash
sqlite3 ./sustained_10k.db \
  "SELECT total_packets, total_gaps FROM (
    SELECT COUNT(*) as total_packets FROM flows
  ) t1, (
    SELECT COUNT(*) as total_gaps FROM gaps
  ) t2;"
```

**Example 3: Realistic Traffic Replay**
```bash
./test_analyzer.sh --replay original ./macsec_traffic.pcap -d ./realistic.db --debug
```

**Behavior**: Respects original timing, good for testing with realistic inter-packet timing

**Example 4: 10x Speed Acceleration**
```bash
./test_analyzer.sh --replay speed ./macsec_traffic.pcap --replay-speed 10.0 \
  -d ./accelerated.db --debug
```

**Use Case**: Quickly validate behavior with accelerated timing

#### Gap Detection During Looping

When looping is enabled with `--replay-loop`, the analyzer automatically detects and handles loop boundaries:

**How it Works:**
1. PCAP replays until packet count reaches file size
2. At loop boundary, ReplayCapture signals analyzer with `Ok(None)`
3. Analyzer persists current flow data to database
4. Flow tracker resets to clear sequence numbers
5. Replay restarts from beginning of PCAP
6. Sequence numbers restart from PCAP values (expected behavior)

**Database Behavior:**
- Multiple loops create new entries in `gaps` table at loop boundaries
- These gaps are marked with large gap_size (sequence number reset)
- Query across all loops: `SELECT COUNT(*) FROM gaps;`

#### Performance Benchmarks

Typical performance on commodity hardware (Intel i7, 16GB RAM):

| Replay Mode | Throughput | Notes |
|------------|-----------|-------|
| Fast | 100K - 1M pps | CPU-limited, varies by packet size |
| OriginalTiming | Varies | Depends on original PCAP timing |
| FixedRate(1000) | ~1,000 pps | Consistent, limited by sleep precision |
| FixedRate(100000) | ~100K pps | Tokio scheduling overhead |
| SpeedMultiplier(10x) | 10× original | Scales with original rate |

**Memory Usage:**
- Small PCAP (1MB, ~1K packets): ~3-5 MB
- Medium PCAP (100MB, ~100K packets): ~150-200 MB
- Large PCAP (1GB, ~1M packets): ~1.2-1.5 GB

#### Tips for Effective Replay Testing

1. **Start with Debug Output**
   ```bash
   ./test_analyzer.sh --replay fast ./macsec_traffic.pcap -d ./debug_test.db --debug
   ```
   Shows protocol detection, flow creation, and statistics

2. **Use Release Mode for Performance Testing**
   ```bash
   ./test_analyzer.sh --replay fast ./macsec_traffic.pcap -d ./perf_test.db --release
   ```
   ~2-3x faster than debug builds

3. **Monitor System Resources**
   ```bash
   # In another terminal during replay:
   watch -n 1 'ps aux | grep live_analyzer | grep -v grep'
   ```

4. **Query Results in Real-time**
   ```bash
   # While replay is running:
   sqlite3 ./replay.db "SELECT COUNT(*) FROM flows; SELECT COUNT(*) FROM gaps;"
   ```

5. **Test with Known Gap Patterns**
   If you have a PCAP with known sequence number gaps, replay it to verify gap detection

#### Troubleshooting Replay Tests

**"Failed to open" error**
```
[ReplayCapture] Failed to open macsec_traffic.pcap: No such file or directory
```

**Solution**: Ensure PCAP file path is correct and file exists:
```bash
ls -lh macsec_traffic.pcap
```

**Very Low Packet Rate**
```
[5.0s] Packets: 50, Gaps: 0, Flows: 2, Rate: 10 pps
```

**Solution**:
- For OriginalTiming mode, check PCAP has tight inter-packet spacing
- For FixedRate mode, increase `--replay-pps`
- For SpeedMultiplier mode, use a larger multiplier like `--replay-speed 100.0`

**Process Exiting Immediately**
```
live_analyzer started (PID: 12345)
✓ Servers stopped
```

**Solution**: Check debug output or use `--debug` flag:
```bash
./test_analyzer.sh --replay fast ./macsec_traffic.pcap -d ./debug.db --debug
```

---

## Understanding the Database

The test database (`test_analysis.db`) is a SQLite database containing:

- **Flow statistics**: Per-flow packet counts, gap counts, bandwidth
- **Gap records**: Detailed information about detected sequence gaps
- **Packet metadata**: Timestamps, sequence numbers, flow IDs

After the script exits, the database is preserved, allowing you to:
- Re-run the REST API server to query results
- Export data for analysis
- Compare results between test runs

### Query Database Directly

```bash
sqlite3 ./test_analysis.db

# Inside sqlite3:
.tables                          # Show all tables
.schema flows                    # Show flows table structure
SELECT * FROM flows LIMIT 5;     # View first 5 flows
SELECT COUNT(*) FROM gaps;       # Count total gaps detected
```

---

## Performance Testing

For performance testing with release builds:

```bash
# Test with high packet rates on eth0
./test_analyzer.sh -i eth0 --release

# In another terminal, monitor system stats
watch -n 1 'ps aux | grep live_analyzer | grep -v grep'
```

The REST API can show real-time performance metrics:

```bash
while true; do
  curl -s http://localhost:3000/api/v1/stats/summary | \
    jq '.total_packets_received, .total_gaps_detected'
  sleep 1
done
```

### Bandwidth Calculation

The API reports bandwidth metrics for each flow and overall:

- **Per-flow bandwidth**: Calculated from flow-specific first and last timestamps
  - Formula: `(total_bytes * 8 bits/byte) / duration_seconds / 1,000,000 bits/Mbps`

- **Summary bandwidth**: Calculated from the earliest first_timestamp to the latest last_timestamp across all flows
  - This represents the overall utilization span during the capture period
  - If flows start/end at different times, the summary correctly captures the full time window

**Note**: In PCAP replay mode with looping enabled, the timestamp reset signal allows the analyzer to persist intermediate data at loop boundaries. The cumulative timestamps across all loops provide accurate bandwidth measurements for the entire replay session.

---

## Automated Test Script

Create `run_tests.sh`:

```bash
#!/bin/bash

set -e

echo "=== MACsec Packet Analyzer - Test Suite ==="
echo ""

# Build
echo "1. Building binary..."
cargo build --bin live_analyzer --release
echo "   ✓ Build successful"
echo ""

# Unit tests
echo "2. Running unit tests..."
cargo test --lib --release
echo "   ✓ Tests passed"
echo ""

# Loopback test
echo "3. Testing local loopback capture..."
timeout 15 sudo cargo run --bin live_analyzer --release -- lo generic test_lo.db pcap &
sleep 2
for i in {1..100}; do echo "test" | nc -q 1 127.0.0.1 8765 2>/dev/null || true; done
wait
if [ -f test_lo.db ]; then
    FLOWS=$(sqlite3 test_lo.db "SELECT COUNT(*) FROM flows")
    echo "   ✓ Captured $FLOWS flows"
    rm test_lo.db
else
    echo "   ✗ Database not created"
fi
echo ""

echo "=== All tests complete ==="
```

Run with:
```bash
chmod +x run_tests.sh
./run_tests.sh
```

---

## Troubleshooting

### Script fails to build
- Make sure you're in the project root directory
- Check that Rust is installed: `rustc --version`
- Try cleaning: `cargo clean && ./test_analyzer.sh`

### Can't access API endpoints
- Verify the server started (check for "rest_api_server started" message)
- Check the port is correct: `lsof -i :3000` (or whatever port you specified)
- Try the health endpoint first: `curl http://localhost:3000/health`

### Permission denied error
- Make sure script is executable: `chmod +x test_analyzer.sh`
- Check you have permissions to write databases in the target location

### Network interface doesn't exist
- List available interfaces: `ip link show` or `ifconfig`
- Use `lo` (loopback) for testing without special privileges
- Use `eth0`, `en0`, `wlan0` etc. for testing with actual network traffic

### Permission Denied (when running binaries)
```
error opening interface: Operation not permitted
```

**Solution**: Run with sudo
```bash
sudo cargo run --bin live_analyzer -- eth0 generic test.db pcap
```

### No Packets Captured

```
[10.0s] Packets: 0, Gaps: 0, Flows: 0, Rate: 0 pps
```

**Solutions**:
1. Check interface name:
   ```bash
   ip link show
   # Use the interface name exactly
   ```

2. Verify interface is active:
   ```bash
   ip link show eth0  # Should show "UP"
   ```

3. Try generating traffic:
   ```bash
   # While analyzer is running
   ping 8.8.8.8
   ```

4. Check with tcpdump:
   ```bash
   sudo tcpdump -i eth0 -c 5  # Should see packets
   ```

### Database Locked

```
Failed to lock database
```

**Solution**: Ensure only one process accesses the database
```bash
# Kill any lingering processes
pkill -f live_analyzer
pkill -f rest_api_server

# Remove old database and retry
rm test.db
cargo run --bin live_analyzer -- eth0 generic test.db pcap
```

### libpcap Not Found

```
error: failed to run custom build command for `pcap`
```

**Solution**: Install libpcap development library
```bash
sudo apt-get install libpcap-dev
cargo clean
cargo build --bin live_analyzer
```

### Out of Memory

```
Cannot allocate memory
```

**Solution**: Limit the number of flows or use pagination
```bash
# Use API with limit parameter
curl http://localhost:3000/api/v1/flows?limit=100&offset=0
```

---

## Documentation Links

- [live_analyzer Documentation](src/bin/live_analyzer.rs) - See module documentation for details
- [REST API Server Documentation](src/bin/rest_api_server.rs) - API endpoint documentation
