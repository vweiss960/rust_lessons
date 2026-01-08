# Live Capture Quick How-To

Fast reference for capturing and analyzing live network traffic.

## One-Minute Quick Start

```bash
# 1. Install libpcap (one time)
sudo apt-get install libpcap-dev

# 2. Build (one time)
cargo build --bin live_analyzer --release

# 3. Start capturing (requires sudo)
# The analyzer automatically detects the protocol from packet headers
sudo cargo run --bin live_analyzer --release -- eth0 live.db pcap

# 4. Generate traffic in another terminal
ping example.com &

# 5. Press Ctrl+C to stop and see results

# 6. Query results with API
cargo run --bin rest_api_server -- --db live.db &
curl http://localhost:3000/api/v1/stats/summary | jq .
```

## Automatic Protocol Detection

The live analyzer automatically detects:
- **MACsec** (EtherType 0x88E5)
- **IPsec ESP** (IPv4 + IP protocol 50)
- **Generic L3** (IPv4 TCP/UDP)

No need to specify the protocol - just provide the interface name!

## Common Commands

### Capture All Traffic (Auto-Detect)
```bash
sudo cargo run --bin live_analyzer -- eth0 live.db pcap
```

The analyzer automatically identifies the protocol and detects gaps for MACsec/IPsec flows.

### Monitor Loopback (no sudo, test only)
```bash
cargo run --bin live_analyzer -- lo test.db pcap
```

**Note**: Gap detection is disabled for Generic L3 (TCP/UDP) traffic. This is intentional because TCP sequence numbers track cumulative bytes, not packets, and permit retransmissions and out-of-order delivery. The analyzer tracks flow statistics (packet count, bytes, bandwidth, timing metrics) for all protocols.

## View Results

### Terminal Output
The analyzer shows real-time progress while running:
```
[5.2s] Packets: 1024, Gaps: 0, Flows: 2, Rate: 197 pps
[10.4s] Packets: 2048, Gaps: 0, Flows: 4, Rate: 197 pps
```

### Final Report
After Ctrl+C:
```
=== Analysis Complete ===
Total packets analyzed: 287451
Total gaps detected: 125
Elapsed time: 28.23s
Packet rate: 10184 pps

Flow ID                                 Packets      Bytes    Gaps   Bandwidth
MACsec { sci: 0x001122334455 }           51234   26234000      25   7.43 Mbps
```

### Database Query
```bash
sqlite3 live.db "SELECT flow_id, packets_received, total_bytes FROM flows"
```

### REST API
```bash
# Start API server
cargo run --bin api_server

# Query in another terminal
curl http://localhost:8080/api/v1/stats/summary | jq .
curl "http://localhost:8080/api/v1/flows?limit=10" | jq .
```

## Arguments Explained

```
live_analyzer <interface> <db_path> <capture_method>
```

- **interface**: Network interface name (eth0, wlan0, lo, etc.)
  - List with: `ip link show`
  - Protocol is auto-detected from packet headers

- **db_path**: Where to save the SQLite database
  - Example: `./results.db` or `/tmp/capture.db`

- **capture_method**: Only `pcap` is supported currently
  - AF_PACKET on Linux requires fixing a pre-existing issue

## Permissions

Live capture requires elevated privileges:

```bash
# Linux/macOS - use sudo
sudo cargo run --bin live_analyzer -- eth0 out.db pcap

# Windows - run as Administrator
# Open PowerShell as Administrator, then:
cargo run --bin live_analyzer -- eth0 out.db pcap
```

## Find Your Interface

```bash
# List all interfaces
ip link show                    # Linux
ifconfig                        # macOS/Linux
ipconfig                        # Windows

# Find the one with traffic
ip link show | grep "state UP"  # Linux
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "Permission denied" | Use `sudo` before the command |
| "No packets captured" | Check interface name with `ip link show` |
| "libpcap not found" | Run `sudo apt-get install libpcap-dev` |
| "Database locked" | Stop any other instances, then retry |
| "Can't bind to localhost:8080" | API server already running, kill it with `pkill api_server` |

## Performance Expectations

- **Packet Rate**: 5,000 - 50,000 packets/sec
- **Memory**: ~100-200 MB for 10,000 flows
- **Database Write**: 5-10 MB per minute
- **CPU Overhead**: <10% per 10k pps

## What Gets Saved

The SQLite database includes:
- Per-flow statistics (packets, bytes)
- Timing metrics (inter-arrival times)
- Bandwidth calculations
- Protocol distribution
- Sequence gap details **(MACsec and IPsec flows only)**

**Note**: Generic L3 (TCP/UDP) flows have gap detection disabled and will not have entries in the sequence_gaps table. For these flows, focus on packet counts, bytes, and bandwidth metrics instead.

Query it with:
```bash
sqlite3 live.db

# Inside sqlite3:
.schema flows
SELECT * FROM flows LIMIT 5;
SELECT COUNT(*) FROM flows;
SELECT * FROM sequence_gaps LIMIT 5;
```

## Analyze Results

Three ways to view results:

1. **Terminal report** (immediate, from Ctrl+C output)
2. **Database queries** (detailed, sqlite3)
3. **REST API** (programmatic, JSON responses)

Example combining all three:

```bash
# 1. Capture
sudo cargo run --bin live_analyzer -- eth0 generic test.db pcap

# 2. Stop with Ctrl+C (see terminal report)

# 3. Query database
sqlite3 test.db "SELECT flow_id, total_bytes FROM flows ORDER BY total_bytes DESC LIMIT 5"

# 4. Start API server
cargo run --bin api_server

# 5. Get JSON results
curl http://localhost:8080/api/v1/flows | jq '.flows[] | select(.bandwidth_mbps > 1)'
```

## Next Steps

For detailed testing procedures, see **TESTING_GUIDE.md**.

For API endpoint documentation, see **QUICK_START.md** → "Querying Results via REST API" section.

For architecture details, see **ARCHITECTURE.md**.
