# Proper Stress Test Methodology for MACsec Analyzer

## ⚠️ Critical Discovery: Real Throughput is 24× Higher

The previous stress test measured **file looping speed** (195 packets × 1,515 loops), not actual processing.

**Real throughput**: **237K pps** with 500K packets (vs. claimed 9,850 pps)

**See [THROUGHPUT_RESULTS.md](THROUGHPUT_RESULTS.md) for detailed analysis**

## Quick Start (3 minutes)

```bash
# Build release binary
cd ..
cargo build --release

# Run baseline test
cd stress_testing
./stress_test.sh --packets 1000 --duration 60

# View results
cat /tmp/stress_tests/results.csv
```

## What This Directory Contains

### Executable Tools

- **`pcap_generator.py`** - Generate synthetic PCAPs with configurable packet counts
  ```bash
  # MACsec-only packets (recommended for accurate benchmarking)
  python3 pcap_generator.py --packets 10000 --flows 1000 --protocol macsec --output test.pcap

  # IPsec packets
  python3 pcap_generator.py --packets 10000 --flows 1000 --protocol ipsec --output test.pcap
  ```

- **`stress_test.sh`** - Run stress tests and measure throughput
  ```bash
  ./stress_test.sh --packets 10000 --duration 120 --verbose
  ```

### Documentation

- **`STRESS_TEST_QUICKSTART.md`** - 5-minute quick start guide
- **`STRESS_TEST_METHODOLOGY.md`** - Complete methodology (5 test phases)
- **`PROPER_STRESS_TEST_SUMMARY.md`** - Executive summary and decision guide

## The Problem We're Solving

The current test methodology is flawed:
- Tests a 195-packet PCAP looped 1,515 times
- Reports 9,850 pps
- Actually measures: Speed of looping a cached file, NOT packet processing
- Problem: 100% cache hit rate hides real performance

## The Solution

New proper methodology that:
- Generates synthetic PCAPs of any size (1K-1M+ packets)
- Processes them **once** (no looping)
- Measures actual throughput
- Identifies bottlenecks (CPU, memory, flow table, database)
- Provides reproducible results

## Example Test Run

```bash
$ ./stress_test.sh --packets 10000 --duration 120

[1/4] Generating synthetic PCAP...
✓ Generated PCAP: stress_10000pkt_1000flows.pcap (1.2 MB)

[2/4] Verifying packet count...
✓ PCAP contains 10000 packets

[3/4] Running analyzer...
Processing..........

✓ Analysis complete (120.234s)

[4/4] Analyzing results...

════════════════════════════════════════════════════════════
STRESS TEST RESULTS
════════════════════════════════════════════════════════════

Test Configuration:
  Input packets:        10000
  Unique flows:         1000
  Test duration:        120.234s

Results:
  Processed packets:    10000
  Gaps detected:        500 (5%)
  Bandwidth:            2.5 Mbps

Performance:
  Throughput:           83.2 pps
  Per-packet time:      12.02 µs

✓ Results saved to: /tmp/stress_tests/results.csv
```

## Understanding Results

**Throughput (pps)** - How many packets per second
- Should stay constant across different packet sizes
- If it drops, you've hit a bottleneck

**Per-Packet Time (µs)** - CPU time per packet
- Calculated as: 1,000,000 / throughput
- Should be ~120 µs based on documentation

**Gap Detection Rate** - Should match injected rate
- Default: 5% injected = 5% detected

## Test Sizes

```bash
# Baseline (small, fast)
./stress_test.sh --packets 1000 --duration 60

# Normal (medium)
./stress_test.sh --packets 10000 --duration 120

# Stress (large)
./stress_test.sh --packets 100000 --duration 120

# Extreme (very large)
./stress_test.sh --packets 500000 --duration 120
```

## Full Suite (Find Bottleneck)

```bash
for size in 1000 10000 50000 100000 250000 500000; do
  echo "Testing $size packets..."
  ./stress_test.sh --packets $size --duration 120
done

# View all results
cat /tmp/stress_tests/results.csv
```

## Interpreting Results

### If Throughput Stays Constant (~8.3K pps)
✓ CPU is the bottleneck - good, performance scales linearly

### If Throughput Drops at 10K Packets
Likely: Flow table contention
Solution: Better hash function, DashMap tuning
Gain: 10-20%

### If Throughput Drops at 100K Packets
Likely: Memory pressure / cache misses
Solution: Streaming, better cache locality
Gain: 20-30%

### If Throughput Drops at 1M Packets
Likely: Database persistence blocking
Solution: Larger batches, async writes
Gain: 30-50%

## Files and Locations

### In This Directory
- `pcap_generator.py` - Generate synthetic PCAPs
- `stress_test.sh` - Run tests
- Documentation (3 MD files)
- This README

### Test Outputs (in /tmp/)
- `/tmp/stress_tests/pcaps/` - Generated PCAP files
- `/tmp/stress_tests/dbs/` - SQLite databases
- `/tmp/stress_tests/results.csv` - Results summary

## Prerequisites

- Rust release binary: `cargo build --release`
- Python 3
- jq (for JSON parsing)
- curl (for API queries)

## Troubleshooting

### "live_analyzer not found"
Build the binary: `cd .. && cargo build --release`

### "pcap_generator.py not found"
Script is in the same directory: `./stress_test.sh`

### "Failed to query results"
Check if database exists: `ls -lh /tmp/stress_tests/dbs/`

## Documentation Index

**For Quick Overview:**
→ Read: `STRESS_TEST_QUICKSTART.md` (10 min)

**For Understanding the Problem:**
→ Read: `PROPER_STRESS_TEST_SUMMARY.md` (15 min)

**For Complete Methodology:**
→ Read: `STRESS_TEST_METHODOLOGY.md` (30 min)

## Next Steps

1. Run baseline test: `./stress_test.sh --packets 1000 --duration 60`
2. Check results: `cat /tmp/stress_tests/results.csv`
3. Run scaling tests to find bottleneck
4. Read methodology to understand results
5. Implement optimizations based on findings
6. Re-test to validate improvements

## Success Criteria

✓ Measure true per-packet processing time
✓ Identify performance bottleneck
✓ Get reproducible results
✓ Scale understanding from 1K to 1M packets
✓ Enable data-driven optimization decisions

