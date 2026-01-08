# Proper Stress Test Methodology for MACsec Analyzer

## The Problem with Current Tests

The current test uses a 195-packet PCAP file looped 1,500+ times in 30 seconds:

```
Measured: 9,850 pps
Actually measuring: PCAP file looping speed, not packet processing speed

Reality check:
- 195 packets × 1,515 loops = 295,425 packets in 30 seconds
- This tests: file I/O caching, loop overhead, persistence batching
- This does NOT test: packet processing capacity at high sustained rates
```

**The metric is useless for understanding real processing speed because:**
1. All 195 packets fit in L1/L2 CPU cache after first loop
2. Protocol detection is cached after one pass
3. Loop overhead is negligible
4. Persistence batching hides actual per-packet cost
5. Real networks don't send the same 195 packets 1,500 times

---

## Proper Stress Test Methodology

### Goal
Measure the true per-packet processing time of the analyzer under realistic conditions.

### Key Principles

1. **Diverse packet types** - Not just the same 195 packets repeatedly
2. **Cache-realistic** - Don't let protocol detection cache artificially boost performance
3. **Sustained load** - Test at target throughput for entire test duration
4. **Single packet focus** - Understand per-packet processing time, not loop overhead
5. **Reproducible** - Use synthetic data with known characteristics

---

## Test Design

### Phase 1: Single Packet Processing (Baseline)

**Objective**: Measure the absolute minimum per-packet cost

**Setup**:
- Create PCAP with 1 unique packet
- Process for 10 seconds
- No looping (process same packet once at a time)

**Expected result**: ~120 µs per packet (as documented)

**Command**:
```bash
./stress_test.sh --mode baseline --packets 1 --duration 10
```

**Analysis**:
- Measures steady-state per-packet cost
- All overhead included (persistence, locking, etc.)
- Should show ~8.3K pps (1,000,000 µs ÷ 120 µs)

---

### Phase 2: Packet Diversity (Cache Effect)

**Objective**: Measure effect of protocol detection caching

**Setup**:
- Create PCAP with 10 unique packets (different protocols/flows)
- Process for 60 seconds
- Let caching stabilize

**Hypothesis**: Should perform similar to Phase 1 (caching helps minimal)

**Command**:
```bash
./stress_test.sh --mode diversity --packets 10 --duration 60
```

**Expected result**: ~8.3K pps (similar to baseline)

---

### Phase 3: Large Packet Set (Real Conditions)

**Objective**: Simulate real network traffic with many unique flows

**Setup**:
- Create PCAP with 10,000 unique packets
  - 1,000 unique flows
  - 10 packets per flow
  - Randomized sequence numbers
  - Realistic gaps (inject 5% packet loss)
- Process for 120 seconds
- No looping

**Command**:
```bash
./stress_test.sh --mode realistic --packets 10000 --duration 120
```

**Expected result**: ~8.3K pps (sustained)

**Metrics to capture**:
- Overall pps
- Per-second breakdown (variance)
- Gap detection accuracy
- Database write latency
- Memory usage

---

### Phase 4: Persistence Impact

**Objective**: Measure impact of database persistence

**Setup**:
- Run Phase 3 twice:
  - Version A: Persistence every 100K packets
  - Version B: Persistence every 1M packets
- Compare results

**Command**:
```bash
./stress_test.sh --mode realistic --packets 10000 --duration 120 --persist-interval 100000
./stress_test.sh --mode realistic --packets 10000 --duration 120 --persist-interval 1000000
```

**Expected result**: Should be similar (persistence is async)

---

### Phase 5: Maximum Sustainable Throughput

**Objective**: Find the maximum pps the analyzer can sustain

**Setup**:
- Generate PCAP with N packets where:
  - Vary N: 1K, 10K, 100K, 1M
  - Test each at max speed (fast mode replay)
  - 60-second test duration

**Command**:
```bash
for size in 1000 10000 100000 1000000; do
  ./stress_test.sh --mode stress --packets $size --duration 60 --mode fast
done
```

**Expected results**:
- Graph throughput vs dataset size
- Identify where performance degrades
- Determine hardware bottleneck (memory, CPU, disk)

---

## Implementation: Synthetic PCAP Generator

Create a tool that generates deterministic, diverse PCAPs:

```bash
./pcap_generator.sh --packets 10000 --flows 1000 --output test_10k.pcap

Options:
  --packets N         Total packets to generate
  --flows N           Number of unique flows
  --gap-rate 0.05     Inject 5% packet loss (gaps)
  --seed 12345        Seed for reproducibility
  --output FILE       Output PCAP filename
```

### Generator Logic

```python
def generate_pcap(num_packets, num_flows, gap_rate=0.05, seed=None):
    """
    Generate synthetic PCAP with:
    - N unique packets spread across M flows
    - Realistic packet distribution (20 bytes to 1500 bytes)
    - Protocol mix: 60% MACsec, 40% IPsec
    - Sequence number gaps (packet loss simulation)
    """

    packets = []
    flow_seq_numbers = {i: 0 for i in range(num_flows)}

    for i in range(num_packets):
        flow_id = i % num_flows
        seq_num = flow_seq_numbers[flow_id]

        # Inject gaps (5% of packets are "missing")
        if random.random() < gap_rate:
            seq_num += random.randint(2, 10)

        # Create packet
        packet = create_packet(
            flow_id=flow_id,
            seq_num=seq_num,
            protocol=random.choice(['MACsec', 'IPsec']),
            size=random.choice([64, 128, 512, 1500])
        )

        packets.append(packet)
        flow_seq_numbers[flow_id] = seq_num + 1

    return packets
```

---

## Test Harness Script

Create `stress_test.sh`:

```bash
#!/bin/bash

# Proper stress test for packet processing speed
# Usage: ./stress_test.sh --packets 10000 --duration 60 --mode fast

PACKETS=1000
DURATION=60
MODE="fast"
OUTPUT_DIR="/tmp/stress_tests"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --packets) PACKETS="$2"; shift 2 ;;
        --duration) DURATION="$2"; shift 2 ;;
        --mode) MODE="$2"; shift 2 ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

mkdir -p $OUTPUT_DIR

echo "=== Stress Test Configuration ==="
echo "Packets: $PACKETS"
echo "Duration: ${DURATION}s"
echo "Replay mode: $MODE"
echo ""

# Step 1: Generate synthetic PCAP
echo "[1/4] Generating synthetic PCAP with $PACKETS packets..."
PCAP_FILE="$OUTPUT_DIR/stress_test_${PACKETS}pkt.pcap"
python3 pcap_generator.py \
    --packets $PACKETS \
    --flows $((PACKETS / 10)) \
    --output $PCAP_FILE

PACKET_COUNT=$(python3 -c "import struct; f=open('$PCAP_FILE','rb'); f.seek(24); c=0
while True:
    h=f.read(16)
    if not h: break
    l,=struct.unpack('<I',h[8:12]); f.read(l); c+=1
print(c)")

echo "✓ Generated $PACKET_COUNT packets in $PCAP_FILE"
echo ""

# Step 2: Clean database
echo "[2/4] Preparing test database..."
DB_FILE="$OUTPUT_DIR/stress_test_${PACKETS}pkt.db"
rm -f "$DB_FILE" "$DB_FILE-shm" "$DB_FILE-wal"
echo "✓ Database ready: $DB_FILE"
echo ""

# Step 3: Run analyzer
echo "[3/4] Running analyzer for ${DURATION} seconds..."
echo "Command: ./target/release/live_analyzer $PCAP_FILE $DB_FILE --replay --mode $MODE"
echo ""

ANALYZE_START=$(date +%s%N)
timeout ${DURATION} ./target/release/live_analyzer \
    "$PCAP_FILE" "$DB_FILE" --replay --mode $MODE > /tmp/analyzer_output.txt 2>&1 &
ANALYZER_PID=$!

# Wait for duration or until analyzer finishes
sleep $DURATION
wait $ANALYZER_PID 2>/dev/null || true
ANALYZE_END=$(date +%s%N)

ELAPSED_SECONDS=$(echo "scale=2; ($ANALYZE_END - $ANALYZE_START) / 1000000000" | bc)

echo "✓ Analysis complete"
echo ""

# Step 4: Query results
echo "[4/4] Analyzing results..."

# Start REST API temporarily
./target/release/rest_api_server --db "$DB_FILE" --port 9999 > /dev/null 2>&1 &
REST_PID=$!
sleep 1

# Query summary stats
STATS=$(curl -s http://localhost:9999/api/v1/stats/summary)
TOTAL_PACKETS=$(echo "$STATS" | jq '.total_packets_received')
TOTAL_GAPS=$(echo "$STATS" | jq '.total_gaps')
TOTAL_FLOWS=$(echo "$STATS" | jq '.total_flows')
BANDWIDTH=$(echo "$STATS" | jq '.avg_bandwidth_mbps')

kill $REST_PID 2>/dev/null || true

echo ""
echo "=== STRESS TEST RESULTS ==="
echo "Test duration:        ${ELAPSED_SECONDS}s"
echo "Input packets:        $PACKET_COUNT"
echo "Processed packets:    $TOTAL_PACKETS"
echo "Unique flows:         $TOTAL_FLOWS"
echo "Gaps detected:        $TOTAL_GAPS"
echo ""
echo "Throughput:           $(echo "scale=1; $TOTAL_PACKETS / $ELAPSED_SECONDS" | bc) pps"
echo "Per-packet time:      $(echo "scale=2; 1000000 / ($TOTAL_PACKETS / $ELAPSED_SECONDS)" | bc) µs"
echo "Bandwidth:            ${BANDWIDTH} Mbps"
echo ""

# Variance analysis
echo "=== DETAILED ANALYSIS ==="
echo "Packets processed exactly once: $(($PACKET_COUNT == $TOTAL_PACKETS ? "YES" : "NO (looped $(echo "scale=1; $TOTAL_PACKETS / $PACKET_COUNT" | bc)x)"))"
echo "Gap detection rate:    $(echo "scale=1; ($TOTAL_GAPS / $TOTAL_PACKETS) * 100" | bc)%"
echo "Flows per second:      $(echo "scale=1; $TOTAL_FLOWS / $ELAPSED_SECONDS" | bc)"
echo ""

# Save results to CSV
echo "$PACKET_COUNT,$ELAPSED_SECONDS,$TOTAL_PACKETS,$(echo "scale=1; $TOTAL_PACKETS / $ELAPSED_SECONDS" | bc),$TOTAL_GAPS,$TOTAL_FLOWS" \
    >> "$OUTPUT_DIR/results.csv"

echo "Results saved to: $OUTPUT_DIR/results.csv"
```

---

## Interpreting Results

### What to Expect

**Baseline (1 packet repeated)**: ~8.3K pps
- Purely CPU-bound
- No complex flow state management
- Protocol detection cached

**10 packets**: ~8.3K pps
- Minimal flow table overhead
- Still dominated by protocol detection

**10,000 packets (realistic)**: ~7-8K pps
- Flow table lookups add overhead
- More realistic per-packet cost
- Persistence batching helps

**1M packets (stress)**: ~6-7K pps or degrades
- Where does it drop?
  - If at 10K packets: memory pressure
  - If at 100K packets: database overhead
  - If at 1M packets: cache line contention

---

## Metrics That Matter

### Per-Test Metrics

1. **Throughput (pps)**: Packets processed per second
   - Formula: `total_packets / elapsed_seconds`
   - Should be constant across all tests (~8.3K pps)
   - If varies, indicates bottleneck

2. **Per-Packet Time (µs)**:
   - Formula: `1,000,000 µs / pps`
   - Baseline: ~120 µs
   - Indicates CPU cost per packet

3. **Variance**: How consistent is throughput?
   - Measure pps per second
   - Low variance (<5%): good, CPU-bound processing
   - High variance (>10%): system contention, I/O blocking

4. **Gap Detection Accuracy**:
   - Did it find all injected gaps?
   - False positives/negatives indicate logic issues

### Cross-Test Metrics

**Graph 1: Throughput vs Dataset Size**
```
   pps
    |
 10K +
     |     ·
     |     · ·
  8K +     · · ·
     |   ·   ·   ·
  6K +  ·       ·
     | ·         ·
  4K +           · · ·
     |
     +----+----+----+----+---> Packet Count
         1K  10K 100K  1M
```

**What this tells you**:
- Flat line (8K): CPU-bound, scales well
- Drops at 10K: Flow table contention
- Drops at 100K: Memory pressure
- Drops at 1M: Database/persistence bottleneck

**Graph 2: Variance Over Time**
```
   pps
    |  ┌────────────┐
 10K +  │ ·· ·· ··· │
     |  │ ··· ·· ·  │ Good: <5% variance
  8K +  │ ·· ·· ··· │ Steady state
     |  └────────────┘
  6K +
     |
     +----+----+----+----+---> Time (seconds)
         0  30  60  90
```

---

## Expected Findings

### Current State Analysis

The current 195-packet PCAP shows:
- **Measured**: 9,850 pps
- **Why**: 195 packets looped 1,515 times

**Hypothesis for true processing capacity**:
- Single packet processed repeatedly: ~8.3K pps (per documented 120 µs)
- This is our baseline
- Shouldn't degrade much until memory pressure (100K+ packets)

### Likely Bottleneck Sequence

1. **At 1K-10K packets**: No degradation (~8.3K pps)
   - Flow table still in cache
   - Working set fits in CPU cache

2. **At 10K-100K packets**: Minor degradation (7-8K pps)
   - Flow table lookup penalties
   - Cache misses increase

3. **At 100K-1M packets**: Significant degradation (5-7K pps)
   - Memory bandwidth exhausted
   - Database write stalls

4. **At 1M+ packets**: Potential shutdown
   - Out of memory or database locked

---

## Recommendations

### 1. Establish True Baseline
Run all tests to find the true per-packet cost:
```bash
./stress_test.sh --packets 1000 --duration 60 --mode fast
```

### 2. Find Degradation Point
Test at increasing scales:
```bash
for size in 1000 10000 50000 100000 250000 500000 1000000; do
  ./stress_test.sh --packets $size --duration 60 --mode fast
done
```

### 3. Identify Bottleneck
- If degrades at 10K: Flow table overhead
- If degrades at 100K: Memory pressure
- If degrades at 1M: Database I/O

### 4. Optimize Accordingly
- Flow table issue: Use better hash function, tune DashMap
- Memory issue: Stream processing instead of batch
- Database issue: Async writes, larger batches

---

## Files to Create

1. **pcap_generator.py** - Synthetic PCAP generator
2. **stress_test.sh** - Test harness
3. **analyze_results.py** - Results analysis and graphing
4. **STRESS_TEST_RESULTS.md** - Document findings

---

## Quick Start

```bash
# Build release binary
cargo build --release

# Run baseline test (1,000 packets, 60 seconds)
./stress_test.sh --packets 1000 --duration 60

# Run realistic test (10,000 packets, 120 seconds)
./stress_test.sh --packets 10000 --duration 120

# Find the bottleneck
for size in 1000 10000 100000; do
  ./stress_test.sh --packets $size --duration 60
done

# Analyze results
python3 analyze_results.py /tmp/stress_tests/results.csv
```

---

## Success Criteria

✓ **Baseline established**: Know true per-packet cost (~120 µs = 8.3K pps)
✓ **Degradation point identified**: At what dataset size does throughput drop?
✓ **Bottleneck isolated**: Know if it's CPU, memory, database, or flow table
✓ **Reproducible results**: Same PCAP, same results each time
✓ **Actionable insights**: Know what to optimize next

