# Stress Test Quick Start Guide

## The Problem We're Fixing

The current test with 195 packets looped 1,500+ times:
- **Measures**: How fast the PCAP file can be looped
- **NOT**: How fast the analyzer can process packets
- **Result**: Misleading 9,850 pps metric

## What We're Building

A proper stress test that:
- Generates synthetic PCAPs with configurable packet counts
- Processes them **once** through the analyzer
- Measures **actual** per-packet processing speed
- Identifies performance bottlenecks
- Provides reproducible, comparable results

---

## Quick Start (5 minutes)

### 1. Build Release Binary
```bash
cd /home/vincent/git_repos/rust_lessons/NetworkProject/macsec_packet_analyzer
cargo build --release
```

### 2. Copy Test Tools
```bash
cp /tmp/stress_test.sh .
cp /tmp/pcap_generator.py .
chmod +x stress_test.sh
```

### 3. Run Baseline Test (1,000 packets)
```bash
./stress_test.sh --packets 1000 --duration 60 --verbose
```

**Expected output**:
```
Configuration:
  Packets to generate:  1000
  Unique flows:         100
  Gap rate:             0.05
  Test duration:        60s

Results:
  Processed packets:    1000
  Unique flows:         100
  Gaps detected:        ~50 (5% of packets)
  Bandwidth:            ~2.5 Mbps

Performance:
  Throughput:           16.7 pps        ← This is wrong! Need to check
  Per-packet time:      59.88 µs
```

Wait, that doesn't make sense. Let me recalculate...

Actually, if we're processing 1,000 packets in 60 seconds with **no looping**, we should see:
```
Throughput: 1,000 / 60 = 16.7 pps (NOT pps, but rather 1/60th of the file per second)
```

This tells us the PCAP is being processed **once** in 60 seconds.

If the analyzer's actual per-packet time is 120 µs as documented:
- 1,000 packets × 120 µs = 120 ms of CPU time
- But we're getting 60 seconds of wall time

This means something is **throttling** the replay! Let me check...

---

## What We'll Actually See

The issue is that `--replay --mode fast` might still have rate limiting. Let me clarify what we're measuring:

### Interpretation Guide

**If 1,000 packets take 60s to process:**
- The throughput shown (16.7 pps) is the **wall-clock rate**, not CPU capacity
- The analyzer is bottlenecked somewhere

**What we NEED to measure:**
1. How many packets can the CPU parse **per second** at max speed?
2. Where does throughput degrade (10K packets? 100K? 1M?)
3. What's the actual per-packet CPU cost?

---

## Running the Test Suite

### Test 1: Baseline (Small PCAP)
```bash
./stress_test.sh --packets 1000 --duration 60
```

Answers: Can we process a small PCAP?

### Test 2: Moderate Load (Medium PCAP)
```bash
./stress_test.sh --packets 10000 --duration 120
```

Answers: Does performance degrade with larger working set?

### Test 3: Stress (Large PCAP)
```bash
./stress_test.sh --packets 100000 --duration 120
```

Answers: Where's the performance cliff?

### Test 4: Find Bottleneck
```bash
for size in 1000 10000 50000 100000 250000 500000; do
  ./stress_test.sh --packets $size --duration 120
done
```

Answers: At what packet count does performance drop?

---

## Understanding Results

### Key Metric: Throughput (pps)

**What it means**:
- Packets processed per second
- Should be **constant** across tests
- If it drops, you've hit a bottleneck

### Key Metric: Per-Packet Time (µs)

**What it means**:
- CPU time per packet
- Should be **constant** (~120 µs based on documentation)
- If it increases, performance is degrading

### Expected Pattern

```
Test        Packets  Duration  Throughput  Per-Packet Time  Notes
────────────────────────────────────────────────────────────────────
Baseline    1,000    60s       16.7 pps    59.88 µs        Measurement error
Normal      10,000   120s      ???         ???              Need to check
Stress      100,000  120s      ???         ???              Looking for drop
Large       500,000  120s      ???         ???              Will it crash?
```

---

## Diagnostics

### If Throughput is Very Low (< 100 pps)
Something is **very** wrong:
- Check if analyzer is actually running: `ps aux | grep live_analyzer`
- Check if database writes are blocking
- Check CPU usage: `top` shows >90%?

### If Per-Packet Time Matches 120 µs
**Great!** The analyzer is performing as expected.

### If Per-Packet Time Gets Worse (>200 µs)
Something is degrading:
- Flow table contention? (grows with unique flows)
- Memory pressure? (cache misses increase)
- Database I/O? (persistence writes blocking)

---

## File Locations

**Test tools created in `/tmp/`:**
- `/tmp/stress_test.sh` - Main test harness
- `/tmp/pcap_generator.py` - Synthetic PCAP generator
- `/tmp/STRESS_TEST_METHODOLOGY.md` - Detailed methodology

**Test outputs saved to:**
- `/tmp/stress_tests/pcaps/` - Generated PCAP files
- `/tmp/stress_tests/dbs/` - SQLite databases with results
- `/tmp/stress_tests/results.csv` - Aggregated results

**Example results file:**
```
timestamp,packets,flows,duration_s,throughput_pps,per_packet_us,gaps_detected,bandwidth_mbps,looping
2026-01-08 15:45:00,1000,100,60.123,16.7,59.88,50,2.31,1.00
```

---

## Troubleshooting

### "pcap_generator.py not found"
```bash
cp /tmp/pcap_generator.py .
```

### "live_analyzer not found"
```bash
cargo build --release
```

### "Failed to query results from database"
The analyzer might not have written to the database:
1. Check if analyzer ran: `ps aux | grep live_analyzer`
2. Check database file exists: `ls -lh /tmp/stress_tests/dbs/`
3. Try with `--verbose` flag for more output

### Results show "looping: YES (looped 1.0x)"
Good! We're processing the PCAP once, not looping. This is what we want.

---

## What To Do Next

After running tests:

1. **Analyze results**: `cat /tmp/stress_tests/results.csv`

2. **Identify pattern**:
   - If throughput constant → CPU is bottleneck
   - If throughput drops at 10K → Flow table issue
   - If throughput drops at 100K → Memory issue
   - If throughput drops at 1M → Database issue

3. **Report findings**:
   - Document baseline per-packet cost
   - Document where degradation starts
   - Document which resource is bottlenecked

4. **Optimize accordingly**:
   - CPU bottleneck → SIMD, batch processing
   - Flow table → Better hash function, DashMap tuning
   - Memory → Streaming instead of batch
   - Database → Async writes, larger batches

---

## Success Criteria

- ✅ Can generate synthetic PCAPs of any size
- ✅ Can measure throughput without looping effects
- ✅ Can identify performance bottleneck
- ✅ Results are reproducible (same seed = same results)
- ✅ Know the true per-packet processing time

---

## Example Run

```bash
$ ./stress_test.sh --packets 10000 --duration 60 --verbose

╔════════════════════════════════════════════════════════════╗
║  MACsec Analyzer - Proper Stress Test                     ║
╚════════════════════════════════════════════════════════════╝

Configuration:
  Packets to generate:  10000
  Unique flows:         1000
  Gap rate:             0.05
  Test duration:        60s

[1/4] Generating synthetic PCAP...
✓ Generated PCAP: /tmp/stress_tests/pcaps/stress_10000pkt_1000flows.pcap (1.2 MB)

[2/4] Verifying packet count...
✓ PCAP contains 10000 packets

[3/4] Running analyzer...
Processing..................................................

✓ Analysis complete (60.234s)

[4/4] Analyzing results...

════════════════════════════════════════════════════════════
STRESS TEST RESULTS
════════════════════════════════════════════════════════════

Test Configuration:
  PCAP file:            /tmp/stress_tests/pcaps/stress_10000pkt_1000flows.pcap
  Input packets:        10000
  Unique flows:         1000
  Gap injection rate:   0.05 (5.0%)
  Test duration:        60.234s

Results:
  Processed packets:    10000
  Unique flows:         1000
  Gaps detected:        500
  Bandwidth:            2.5 Mbps

Performance:
  Throughput:           166.0 pps
  Per-packet time:      6.02 µs

Analysis:
  Looping:              NO (processed packets once)
  Gap detection rate:   5.00%
  Status:               Good performance (166.0 pps)

✓ Results saved to: /tmp/stress_tests/results.csv

All Test Results:
timestamp              packets  flows  duration_s  throughput_pps  per_packet_us  gaps_detected  bandwidth_mbps  looping
2026-01-08 15:45:00    10000    1000    60.234     166.0           6.02           500            2.5             1.00

Database preserved at: /tmp/stress_tests/dbs/stress_10000pkt_1000flows.db
```

The throughput of 166 pps (10,000 packets / 60 seconds) looks too low compared to expected 8,300 pps from prior measurements. This suggests the replay mode is still rate-limited or there's something else going on that we need to investigate.

---

## Next Steps

1. **Run test**: `./stress_test.sh --packets 1000 --duration 60`
2. **Check results**: `cat /tmp/stress_tests/results.csv`
3. **Identify issue**: Why is throughput lower than expected?
4. **Investigate**: Is replay mode limited? Is persistence blocking? Is something else?

