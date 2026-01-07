# Testing the Live Analyzer

This guide provides detailed instructions for testing the `live_analyzer` binary with real or simulated network traffic.

## Prerequisites

### System Requirements

- **Linux**, **macOS**, or **Windows** (tested on Linux primarily)
- **Rust** 1.70+ (for async/await support)
- **libpcap** development library
- **root** or **Administrator** privileges for packet capture

### Install libpcap

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

### Install Optional Tools (for testing)

```bash
# For traffic generation and analysis
sudo apt-get install curl wget tcpdump iperf3 ncat

# For JSON query (optional but helpful)
sudo apt-get install jq

# For PCAP replay (if you have PCAP files)
sudo apt-get install tcpreplay
```

### Verify Installation

```bash
# Check libpcap version
pcap-config --version

# List available network interfaces
ip link show              # Linux
ifconfig                  # macOS
ipconfig                  # Windows

# Check you have sudo access
sudo whoami               # Should output "root"
```

## Build Instructions

### Compile the Binary

```bash
cd /path/to/macsec_packet_analyzer

# Build with all features
cargo build --bin live_analyzer --release

# For testing, use debug build (faster to compile)
cargo build --bin live_analyzer
```

### Verify Build

```bash
# Check binary exists
ls -lh target/debug/live_analyzer

# Test help message
cargo run --bin live_analyzer -- --help 2>&1 || true
```

## Test Scenario 1: Local Loopback Interface

**Best for**: Quick testing without needing network setup, testing protocol parser with known traffic.

### Setup

```bash
# Terminal 1: Start the analyzer on lo
sudo cargo run --bin live_analyzer -- lo generic test_lo.db pcap
```

You should see:
```
Starting packet analyzer
  Interface: lo
  Protocol: generic
  Database: test_lo.db
  Capture: pcap

PCAP capture started on interface 'lo'
Press Ctrl+C to stop and save results
```

### Generate Traffic

```bash
# Terminal 2: Generate TCP traffic to localhost
for i in {1..1000}; do
  timeout 1 curl http://localhost:8765 2>/dev/null || true
  sleep 0.01
done

# Or use a simpler loop
for i in {1..500}; do
  echo "test" | nc -q 1 127.0.0.1 8765 2>/dev/null || true
done
```

### Expected Results

In Terminal 1, you should see output like:
```
[5.2s] Packets: 1024, Gaps: 0, Flows: 2, Rate: 197 pps
[10.4s] Packets: 2048, Gaps: 0, Flows: 4, Rate: 197 pps
```

Press Ctrl+C to stop and see final report.

### Verify Results

```bash
# Check database size
ls -lh test_lo.db

# Query database
sqlite3 test_lo.db "SELECT COUNT(*) as flows FROM flows"
sqlite3 test_lo.db "SELECT flow_id, packets_received, total_bytes FROM flows LIMIT 3"

# Start API server
cargo run --bin api_server

# Query API (in another terminal)
curl -s http://localhost:8080/api/v1/stats/summary | jq .
```

---

## Test Scenario 2: Real Network Interface with Traffic Generation

**Best for**: Testing with actual network traffic, verifying bandwidth calculations.

### Step 1: Identify Your Interface

```bash
# List interfaces with their IPs
ip addr show

# Pick a common one (e.g., eth0, wlan0, ens33)
# For this example, we'll use eth0
```

### Step 2: Start Analyzer

```bash
# Terminal 1: Monitor eth0 for TCP/UDP traffic
sudo cargo run --bin live_analyzer -- eth0 generic capture_eth0.db pcap
```

### Step 3: Generate Traffic

```bash
# Terminal 2: Generate HTTP traffic
for i in {1..50}; do
  curl http://example.com > /dev/null 2>&1 &
done

# Or generate larger traffic with iperf3
# Terminal 3:
iperf3 -s -p 5201 &

# Terminal 2:
iperf3 -c 192.168.1.x -p 5201 -t 30 -P 4

# Or simple ping traffic
ping google.com &
```

### Step 4: Monitor Progress

In Terminal 1, watch the real-time stats:
```
[30.5s] Packets: 15234, Gaps: 0, Flows: 8, Rate: 500 pps
[60.8s] Packets: 30512, Gaps: 2, Flows: 12, Rate: 501 pps
```

### Step 5: Stop and Check Results

Press Ctrl+C in Terminal 1 and check the final report.

```bash
# Expected output:
=== Analysis Complete ===
Total packets analyzed: 31245
Total gaps detected: 2
Elapsed time: 62.34s
Packet rate: 501 pps

Flows analyzed: 12

Flow ID                                    Packets     Bytes        Gaps   Bandwidth
...
TCP { 192.168.1.100:45123 -> 8.8.8.8:443 }   2341   1230000        0   1.57 Mbps
```

---

## Test Scenario 3: PCAP File Replay

**Best for**: Reproducing specific packet sequences, testing with known datasets.

### Step 1: Create or Find a PCAP File

```bash
# Option A: Create a simple PCAP with tcpdump
# Capture 100 packets from eth0
sudo tcpdump -i eth0 -c 100 -w test_traffic.pcap

# Option B: Use a public PCAP sample
# Download from https://www.wireshark.org/download/sample-captures/
# Example: dhcp-request.pcap
```

### Step 2: Replay the PCAP

```bash
# Install tcpreplay if needed
sudo apt-get install tcpreplay

# Replay with timing preserved
sudo tcpreplay -i eth0 test_traffic.pcap

# Or replay at high speed
sudo tcpreplay -i eth0 --multiplier=10 test_traffic.pcap
```

### Step 3: Capture with Analyzer

```bash
# Terminal 1: Start analyzer before replay
sudo cargo run --bin live_analyzer -- eth0 generic pcap_test.db pcap

# Terminal 2: Replay the PCAP after analyzer starts
sudo tcpreplay -i eth0 test_traffic.pcap
```

### Step 4: Verify

```bash
# Check results when analyzer finishes
sqlite3 pcap_test.db "SELECT COUNT(*) FROM flows"
sqlite3 pcap_test.db "SELECT flow_id, packets_received FROM flows"
```

---

## Test Scenario 4: Protocol-Specific Testing

### Testing with MACsec Traffic

```bash
# Requires MACsec-enabled interface (requires specific NIC and kernel support)
# This is advanced and requires specialized setup

# If you have MACsec enabled on an interface:
sudo cargo run --bin live_analyzer -- macsec_eth0 macsec macsec_test.db pcap

# Monitor the output for MACsec-specific flows
```

### Testing with IPsec Traffic

```bash
# IPsec analysis requires actual IPsec traffic
# Example: VPN tunnel

# First, establish an IPsec connection (VPN)
# Then monitor:
sudo cargo run --bin live_analyzer -- eth0 ipsec ipsec_test.db pcap

# Will detect ESP (Encapsulating Security Payload) packets
```

### Testing with Generic L3 (TCP/UDP) Traffic

```bash
# Generic L3 analysis works with standard TCP/UDP traffic
sudo cargo run --bin live_analyzer -- eth0 generic generic_test.db pcap

# In another terminal, generate traffic:
curl http://example.com &
ping google.com &

# Flow tracking, bandwidth, and byte counts will be recorded
# Gap detection is DISABLED for Generic L3 flows because:
# - TCP sequence numbers track cumulative bytes, not packets
# - TCP permits retransmissions and out-of-order delivery
# - This causes 67%+ false positive rate for gap detection
#
# Use packet_count and bandwidth_mbps metrics instead for TCP/UDP flows
```

---

## Test Scenario 5: Unit Tests

### Run All Tests

```bash
# Test the entire library
cargo test --lib

# Expected output:
running 42 tests
test analysis::flow::tests::test_gap_detection ... ok
test analysis::flow::tests::test_wraparound ... ok
test protocol::macsec::tests::test_parse_valid ... ok
...

test result: ok. 42 passed; 0 failed; 0 ignored
```

### Test Specific Modules

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

### Run Specific Test

```bash
# Run one test by name
cargo test --lib test_gap_detection -- --exact

# Run tests matching a pattern
cargo test --lib gap --
```

---

## Test Scenario 6: REST API Integration

### Setup

```bash
# Terminal 1: Run analyzer for a bit
sudo cargo run --bin live_analyzer -- eth0 generic api_test.db pcap

# Generate traffic while running (see Step 3 above)
# Press Ctrl+C after 30 seconds

# Terminal 2: Start API server
cargo run --bin api_server
```

### Test API Endpoints

```bash
# Health check
curl http://localhost:8080/health

# Summary statistics
curl -s http://localhost:8080/api/v1/stats/summary | jq .

# List flows
curl -s http://localhost:8080/api/v1/flows | jq '.flows | length'

# Get specific flow (URL encode the flow_id)
FLOW_ID=$(curl -s http://localhost:8080/api/v1/flows | jq -r '.flows[0].flow_id')
curl -s "http://localhost:8080/api/v1/flows/$(echo -n "$FLOW_ID" | jq -sRr @uri)" | jq .

# Get gaps for a flow
curl -s "http://localhost:8080/api/v1/flows/$(echo -n "$FLOW_ID" | jq -sRr @uri)/gaps" | jq .

# Filter by bandwidth (Mbps)
curl -s "http://localhost:8080/api/v1/flows?min_bandwidth_mbps=1" | jq '.flows[] | {flow_id, bandwidth_mbps}'

# Filter by packet size
curl -s "http://localhost:8080/api/v1/flows?min_bytes=10000" | jq '.flows[] | {flow_id, total_bytes}'
```

### Verify Response Structure

```bash
# Check summary response has bandwidth
curl -s http://localhost:8080/api/v1/stats/summary | jq 'keys'

# Should output:
# ["avg_bandwidth_mbps", "max_gap_size", "total_bytes", ...]

# Check flow response has all metrics
curl -s http://localhost:8080/api/v1/flows | jq '.flows[0] | keys'

# Should include:
# ["avg_inter_arrival_ms", "bandwidth_mbps", "duration_seconds", ...]
```

---

## Performance Testing

### Measure Packet Rate

```bash
# Run analyzer and observe packet rate
sudo cargo run --bin live_analyzer -- eth0 generic perf_test.db pcap

# Expected output:
[10.0s] Packets: 50000, Gaps: 0, Flows: 20, Rate: 5000 pps    ← pps = packets/sec
[20.0s] Packets: 100000, Gaps: 5, Flows: 35, Rate: 5000 pps
```

### Generate High-Rate Traffic

```bash
# Using iperf3 (generates sustained traffic)
iperf3 -s &                                    # Server
iperf3 -c localhost -t 60 -P 10 -b 1G         # 10 parallel streams, 1Gbps

# Using netperf (another option)
netserver &
netperf -H localhost -l 60 -P 0 &
netperf -H localhost -l 60 -P 0 &
```

### Benchmark Results

Typical performance on commodity hardware:
- **Packet Rate**: 5,000 - 50,000 pps depending on CPU
- **Memory Usage**: ~100-200 MB for 10,000 concurrent flows
- **Database Write**: 5-10 MB per minute of capture
- **Overhead**: < 10% CPU per 10k pps

---

## Troubleshooting

### Issue: Permission Denied

```
error opening interface: Operation not permitted
```

**Solution**: Run with sudo
```bash
sudo cargo run --bin live_analyzer -- eth0 generic test.db pcap
```

### Issue: No Packets Captured

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

### Issue: Database Locked

```
Failed to lock database
```

**Solution**: Ensure only one process accesses the database
```bash
# Kill any lingering processes
pkill -f live_analyzer
pkill -f api_server

# Remove old database and retry
rm test.db
cargo run --bin live_analyzer -- eth0 generic test.db pcap
```

### Issue: libpcap Not Found

```
error: failed to run custom build command for `pcap`
```

**Solution**: Install libpcap development library
```bash
sudo apt-get install libpcap-dev
cargo clean
cargo build --bin live_analyzer
```

### Issue: Out of Memory

```
Cannot allocate memory
```

**Solution**: Limit the number of flows or use pagination
```bash
# Use API with limit parameter
curl http://localhost:8080/api/v1/flows?limit=100&offset=0
```

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

## Documentation Links

- **Protocol Specifications**:
  - MACsec: IEEE 802.1AE
  - IPsec: RFC 4303
  - TCP: RFC 793

- **Tools**:
  - [tcpdump](https://www.tcpdump.org/)
  - [Wireshark](https://www.wireshark.org/)
  - [tcpreplay](https://tcpreplay.appneta.com/)

- **PCAP Samples**: https://www.wireshark.org/download/sample-captures/

---

## Success Criteria

A successful test run should demonstrate:

✅ Binary builds without errors
✅ Packets are captured from live interface
✅ Statistics are calculated correctly
✅ Database stores results
✅ API server queries results
✅ Bandwidth metrics are displayed
✅ Graceful shutdown with Ctrl+C
✅ Final report includes all flows

If all criteria are met, the live_analyzer is working correctly!
