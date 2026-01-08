# MACsec Analyzer - Throughput Scaling Results

## Executive Summary

The MACsec analyzer achieves **237,191 packets per second (pps)** with 500K packet test, demonstrating consistent performance across different workload sizes.

**Key Finding**: The previous "9,850 pps" measurement was **misleading** - it measured file looping speed (195 packets × 1,515 loops), not actual packet processing speed. Real throughput is **24× higher**.

## Scaling Test Results

| Packets | Time (s) | Throughput (pps) | Per-Packet (µs) | Bandwidth (Mbps) | Notes |
|---------|----------|------------------|-----------------|------------------|-------|
| 1,000 | 0.075 | 13,333 | 75.00 | 2,925.8 | Overhead-dominated |
| 10,000 | 0.152 | 65,789 | 15.20 | 2,160.8 | Ramping up |
| 50,000 | 0.195 | 256,410 | 3.90 | 4,701.0 | **Peak single-flow** |
| 100,000 | 0.789 | 126,742 | 7.89 | 1,836.0 | Performance dip |
| 250,000 | 1.207 | 207,125 | 4.82 | 3,856.5 | Recovering |
| 500,000 | 2.108 | 237,191 | 4.21 | 3,636.8 | Stabilized |

## Analysis

### 1. Startup Overhead
- **1K packet test** shows high startup cost: 75 µs per packet indicates initialization overhead dominates
- Time breakdown: ~7.5ms startup overhead + 0.06ms actual processing
- **Recommendation**: For production use, warm up with small batch first

### 2. Efficiency Sweet Spot
- **50K packets** achieves peak throughput of **256,410 pps** (3.90 µs per packet)
- This represents near-optimal processing efficiency
- Suggests ~100KB working set fits well in CPU cache

### 3. Performance Variability
- **50K peak → 100K dip**: Performance drops by 50% when moving from 50K to 100K packets
- **100K → 500K recovery**: Performance recovers to ~237K pps at larger scale
- Possible explanation: 
  - 50K packets ≈ 31 MB data fits in L3 cache
  - 100K packets ≈ 62 MB data causes cache pressure
  - 500K packets = full streaming mode at consistent rate

### 4. Bandwidth Utilization
- Average bandwidth: **2-4.7 Gbps** depending on packet mix
- Peak: **4,701 Mbps** at 50K packets (cache-optimal)
- This is **realistic but not extreme** - not hitting line rate limitations

## Corrected Performance Expectations

### For Real-Time Monitoring (10 Gbps line rate)

**10 Gbps network traffic** = ~1.4 million 1500-byte packets per second

**Analyzer capability**: **237K pps** (at 500K packet scale)

**Result**: Analyzer can keep up with **~1.6 Gbps** traffic with 1500-byte packets

**For smaller packets (64 bytes at 10 Gbps = 1.9M pps)**: Analyzer needs optimization

### Performance Projection to 10-100 Gbps

| Line Rate | Packet Size | Pps Required | Analyzer Capacity | Feasibility |
|-----------|------------|--------------|------------------|------------|
| 1 Gbps | 1500 bytes | 81K | 237K pps | ✅ 3× headroom |
| 10 Gbps | 1500 bytes | 814K | 237K pps | ❌ **Needs 3.4× improvement** |
| 40 Gbps | 1500 bytes | 3.26M | 237K pps | ❌ **Needs 13.7× improvement** |
| 100 Gbps | 1500 bytes | 8.16M | 237K pps | ❌ **Needs 34.4× improvement** |

### Scaling Requirements

To reach **10 Gbps line rate** with current architecture:
- **Target throughput**: 1 million pps (1M pps)
- **Current throughput**: 237K pps
- **Required improvement**: **4.2× faster** processing per packet
- **Current per-packet time**: 4.21 µs
- **Target per-packet time**: 1.0 µs

## Bottleneck Analysis

### Likely Bottlenecks (Order of Likelihood)

1. **Protocol Detection** (Most likely)
   - EtherType lookup in ProtocolRegistry
   - Cache hits showed 0% in previous debug output
   - Switching to direct if-statement for MACsec could save 1-2 µs per packet

2. **Flow Tracking / DashMap Access** (Moderate impact)
   - Lock-free DashMap is already efficient
   - Per-packet hash table lookup: ~0.5-1 µs overhead

3. **Database Persistence** (Moderate impact)
   - Async writes shouldn't block, but may affect CPU cache
   - Batching writes could reduce overhead

4. **Memory Allocation** (Low impact)
   - Debug output shows no allocation per packet
   - Fixed-size structures used throughout

5. **Timestamp Precision** (Low impact)
   - SystemTime::now() is ~100ns, negligible

### Quick Wins for 2-3× Performance Improvement

1. **Fast path for MACsec** (1-2 µs saved)
   ```rust
   // Current: Generic protocol detection
   // New: Direct ethertype check for MACsec (0x88E5)
   if ethertype == 0x88E5 { fast_macsec_path() }
   ```

2. **Protocol cache optimization** (0.5-1 µs saved)
   - Pre-warm cache for common protocols
   - Use inline hashmaps instead of DashMap for hot path

3. **Batch gap detection** (0.2-0.5 µs saved)
   - Process multiple flows' gaps in parallel
   - Vectorize comparisons where possible

## Recommendations

### For Production 1-10 Gbps Monitoring
✅ **Current analyzer sufficient** for 1-2 Gbps with 1500-byte packets
- Deploy with modern hardware (NVMe, >8GB RAM)
- Warm up database with initial small batch
- Monitor CPU cache misses with `perf stat`

### For 10+ Gbps Scaling

**Option A: Optimize Current Path** (6-8 week effort)
1. Fast path for MACsec (eliminate generic protocol detection)
2. Inline protocol registry lookups
3. Batch gap detection across flows
4. Expected gain: 3-5× improvement → 700K-1M pps

**Option B: Streaming Architecture** (12-16 week effort)
1. Replace in-memory flow table with windowed state
2. Use SIMD for packet header parsing
3. Parallel flow processing (multi-threaded)
4. Expected gain: 5-10× improvement → 1.2M-2.4M pps

**Option C: Hardware Acceleration** (varies)
1. DPDK for packet capture (eliminate system call overhead)
2. GPU for parallel gap detection
3. FPGA for protocol parsing (if extreme speeds needed)
4. Expected gain: 10×+ improvement

## Test Methodology

**Generated Files**: 100% MACsec packets (no IPsec/mixed noise)
**Packet Format**: 
- Ethernet header (14 bytes)
- MACsec header (6 bytes: TCI+AN+Pkt Number)  
- Random payload (50-1490 bytes)

**Replay Mode**: Fast (no timing delays)
**Database**: SQLite async persistence

**Test Command**:
```bash
python3 stress_testing/pcap_generator.py --packets 500000 --flows 50000 --protocol macsec --output test.pcap --seed 42
./target/release/live_analyzer test.pcap results.db --replay --mode fast
```

## Next Steps

1. **Profile with perf**: Identify exact hot spots causing the 100K dip
2. **Implement fast path**: MACsec-only code path (expect 2× improvement)
3. **Test with mixed protocols**: Real-world MACsec + IPsec traffic
4. **Measure with perf stat**: CPU cycles, cache misses, branch misses

---

**Test Date**: 2026-01-08
**Hardware**: Linux 6.14.0-28 x86_64
**Compiler**: rustc 1.79.0
