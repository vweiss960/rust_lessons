#!/bin/bash

################################################################################
# Enhanced Stress Test Harness for MACsec Analyzer
#
# This script:
# 1. Cleans old binaries, databases, and PCAP files from stress_testing folder
# 2. Builds fresh release binaries and copies them to stress_testing folder
# 3. Generates synthetic PCAP test data in stress_testing folder
# 4. Processes packets through live_analyzer with timing
# 5. Makes results queryable via rest_api_server running against same database
#
# All artifacts (binaries, databases, PCAP files) are stored in the
# stress_testing folder itself for convenience.
#
# Usage:
#   ./stress_test.sh --packets 10000 --duration 60
#   ./stress_test.sh --packets 100000 --duration 120 --gap-rate 0.1
#   ./stress_test.sh --packets 1000000 --flows 50000 --duration 120
#
################################################################################

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
PACKETS=1000
FLOWS=""
DURATION=60
GAP_RATE=0.05
SEED=42
PROTOCOL="macsec"
VERBOSE=false
PORT=9998

# Get paths - stress_testing folder is where this script lives
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PCAP_FILE="$SCRIPT_DIR/test.pcap"
DB_FILE="$SCRIPT_DIR/stress_tests.db"
LIVE_ANALYZER_BIN="$SCRIPT_DIR/live_analyzer"
REST_API_BIN="$SCRIPT_DIR/rest_api_server"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --packets)
            PACKETS="$2"
            shift 2
            ;;
        --flows)
            FLOWS="$2"
            shift 2
            ;;
        --duration)
            DURATION="$2"
            shift 2
            ;;
        --gap-rate)
            GAP_RATE="$2"
            shift 2
            ;;
        --protocol)
            PROTOCOL="$2"
            shift 2
            ;;
        --seed)
            SEED="$2"
            shift 2
            ;;
        --port)
            PORT="$2"
            shift 2
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            head -n 40 "$0" | tail -n +3
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Calculate flows if not specified
if [ -z "$FLOWS" ]; then
    FLOWS=$((PACKETS / 10))
fi

# Make sure we have required tools
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}Error: python3 not found${NC}"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo not found${NC}"
    exit 1
fi

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  MACsec Analyzer - Enhanced Stress Test                   ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

echo -e "${GREEN}Configuration:${NC}"
echo "  Packets to generate:  $PACKETS"
echo "  Unique flows:         $FLOWS"
echo "  Protocol:             $PROTOCOL"
echo "  Gap rate:             ${GAP_RATE} ($(echo "scale=1; $GAP_RATE * 100" | bc)%)"
echo "  Test duration:        ${DURATION}s"
echo "  PCAP file:            $PCAP_FILE"
echo "  Database:             $DB_FILE"
echo "  Binaries location:    $SCRIPT_DIR"
echo ""

# ============================================================================
# Step 1: Clean old binaries, databases, and PCAP files
# ============================================================================

echo -e "${YELLOW}[1/6] Cleaning old artifacts...${NC}"

# Remove old binaries from stress_testing folder
rm -f "$LIVE_ANALYZER_BIN" "$REST_API_BIN" 2>/dev/null || true

# Remove old databases
rm -f "$DB_FILE" "$DB_FILE-shm" "$DB_FILE-wal" 2>/dev/null || true

# Remove old PCAP files
rm -f "$SCRIPT_DIR"/test.pcap 2>/dev/null || true

echo -e "${GREEN}✓ Old artifacts cleaned${NC}"
echo ""

# ============================================================================
# Step 2: Build release binaries
# ============================================================================

echo -e "${YELLOW}[2/6] Building release binaries...${NC}"

cd "$PROJECT_ROOT"

if cargo build --bin live_analyzer --release 2>&1 | tail -5; then
    echo -e "${GREEN}✓ live_analyzer built successfully${NC}"
else
    echo -e "${RED}✗ Failed to build live_analyzer${NC}"
    exit 1
fi

if cargo build --bin rest_api_server --release 2>&1 | tail -5; then
    echo -e "${GREEN}✓ rest_api_server built successfully${NC}"
else
    echo -e "${RED}✗ Failed to build rest_api_server${NC}"
    exit 1
fi
echo ""

# ============================================================================
# Step 3: Copy binaries to stress_testing folder
# ============================================================================

echo -e "${YELLOW}[3/6] Copying binaries to stress_testing folder...${NC}"

cp "$PROJECT_ROOT/target/release/live_analyzer" "$LIVE_ANALYZER_BIN"
cp "$PROJECT_ROOT/target/release/rest_api_server" "$REST_API_BIN"

echo -e "${GREEN}✓ Binaries copied to $SCRIPT_DIR${NC}"
echo ""

# ============================================================================
# Step 4: Generate synthetic PCAP
# ============================================================================

echo -e "${YELLOW}[4/6] Generating synthetic PCAP...${NC}"

cd "$PROJECT_ROOT"

python3 stress_testing/pcap_generator.py \
    --packets "$PACKETS" \
    --flows "$FLOWS" \
    --protocol "$PROTOCOL" \
    --gap-rate "$GAP_RATE" \
    --seed "$SEED" \
    --output "$PCAP_FILE" \
    $([ "$VERBOSE" = "true" ] && echo "--verbose" || echo "")

if [ ! -f "$PCAP_FILE" ]; then
    echo -e "${RED}✗ Failed to generate PCAP${NC}"
    exit 1
fi

PCAP_SIZE=$(du -h "$PCAP_FILE" | cut -f1)
echo -e "${GREEN}✓ Generated PCAP: $PCAP_FILE ($PCAP_SIZE)${NC}"
echo ""

# ============================================================================
# Step 5: Verify packet count in PCAP
# ============================================================================

echo -e "${YELLOW}[5/6] Verifying packet count...${NC}"

ACTUAL_PACKETS=$(python3 << PYEOF
import struct
import sys

try:
    with open("$PCAP_FILE", 'rb') as f:
        # Skip global header
        f.seek(24)
        count = 0
        while True:
            header = f.read(16)
            if not header or len(header) < 16:
                break
            incl_len = struct.unpack('<I', header[8:12])[0]
            f.read(incl_len)
            count += 1
        print(count)
except Exception as e:
    print(f"Error: {e}", file=sys.stderr)
    sys.exit(1)
PYEOF
)

if [ -z "$ACTUAL_PACKETS" ] || [ "$ACTUAL_PACKETS" -eq 0 ]; then
    echo -e "${RED}✗ Failed to read PCAP file${NC}"
    exit 1
fi

echo -e "${GREEN}✓ PCAP contains $ACTUAL_PACKETS packets${NC}"
echo ""

# ============================================================================
# Step 6: Run analyzer and measure processing time
# ============================================================================

echo -e "${YELLOW}[6/6] Processing packets and measuring performance...${NC}"

# Clean database before running
rm -f "$DB_FILE" "$DB_FILE-shm" "$DB_FILE-wal"

echo "Command: $LIVE_ANALYZER_BIN $PCAP_FILE $DB_FILE --replay --mode fast"
echo ""

ANALYZE_START=$(date +%s%3N)

if [ "$VERBOSE" = "true" ]; then
    "$LIVE_ANALYZER_BIN" "$PCAP_FILE" "$DB_FILE" --replay --mode fast --debug 2>&1 | head -50 &
    ANALYZER_PID=$!
else
    "$LIVE_ANALYZER_BIN" "$PCAP_FILE" "$DB_FILE" --replay --mode fast > /dev/null 2>&1 &
    ANALYZER_PID=$!
fi

# Show progress dots with timeout
echo -n "Processing."
for i in $(seq 1 $DURATION); do
    sleep 1
    echo -n "."
    if ! kill -0 $ANALYZER_PID 2>/dev/null; then
        break
    fi
done
echo ""

# Wait for analyzer to finish
wait $ANALYZER_PID 2>/dev/null || true

ANALYZE_END=$(date +%s%3N)
ELAPSED_MS=$((ANALYZE_END - ANALYZE_START))
ELAPSED_SECONDS=$(echo "scale=3; $ELAPSED_MS / 1000" | bc)

echo -e "${GREEN}✓ Analysis complete (${ELAPSED_SECONDS}s)${NC}"
echo ""

# ============================================================================
# Query results via REST API
# ============================================================================

echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}Querying results via REST API...${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
echo ""

# Find an available port
ACTUAL_PORT=$PORT
while netstat -tln 2>/dev/null | grep -q ":$ACTUAL_PORT "; do
    ACTUAL_PORT=$((ACTUAL_PORT + 1))
done

echo "Starting rest_api_server on port $ACTUAL_PORT..."

# Start REST API server
"$REST_API_BIN" --db "$DB_FILE" --port $ACTUAL_PORT > /dev/null 2>&1 &
REST_PID=$!

# Give it time to start
sleep 2

# Query stats
echo ""
echo -e "${GREEN}Fetching statistics from REST API...${NC}"
STATS=$(curl -s "http://localhost:$ACTUAL_PORT/api/v1/stats/summary" || echo "{}")

# Kill REST server
kill $REST_PID 2>/dev/null || true
wait $REST_PID 2>/dev/null || true

# Extract fields with error handling
PROCESSED_PACKETS=$(echo "$STATS" | python3 -c "import sys, json; d=json.load(sys.stdin); print(d.get('total_packets_received', 0))" 2>/dev/null || echo "0")
TOTAL_GAPS=$(echo "$STATS" | python3 -c "import sys, json; d=json.load(sys.stdin); print(d.get('total_gaps', 0))" 2>/dev/null || echo "0")
TOTAL_FLOWS=$(echo "$STATS" | python3 -c "import sys, json; d=json.load(sys.stdin); print(d.get('total_flows', 0))" 2>/dev/null || echo "0")
BANDWIDTH=$(echo "$STATS" | python3 -c "import sys, json; d=json.load(sys.stdin); print(d.get('avg_bandwidth_mbps', 0))" 2>/dev/null || echo "0")

if [ "$PROCESSED_PACKETS" = "0" ] || [ -z "$PROCESSED_PACKETS" ]; then
    echo -e "${RED}✗ Failed to query results from database${NC}"
    echo "Database file: $DB_FILE"
    echo "Stats response: $STATS"
    exit 1
fi

# Calculate metrics
THROUGHPUT=$(echo "scale=1; $PROCESSED_PACKETS / $ELAPSED_SECONDS" | bc)
PER_PACKET_US=$(echo "scale=2; 1000000 / $THROUGHPUT" | bc)

# Read expected lost packets from JSON file
JSON_FILE="$SCRIPT_DIR/test.json"
if [ -f "$JSON_FILE" ]; then
    # Parse expected lost packets by protocol from JSON
    EXPECTED_LOST_MACSEC=$(python3 << JSONEOF
import json
import sys
try:
    with open("$JSON_FILE", 'r') as f:
        data = json.load(f)
    total = 0
    for flow_id, flow_stats in data.get('flows', {}).items():
        if flow_stats.get('protocol') == 'macsec':
            total += flow_stats.get('lost_packets', 0)
    print(total)
except Exception as e:
    print(f"Error: {e}", file=sys.stderr)
    print(0)
JSONEOF
)

    EXPECTED_LOST_IPSEC=$(python3 << JSONEOF
import json
import sys
try:
    with open("$JSON_FILE", 'r') as f:
        data = json.load(f)
    total = 0
    for flow_id, flow_stats in data.get('flows', {}).items():
        if flow_stats.get('protocol') == 'ipsec':
            total += flow_stats.get('lost_packets', 0)
    print(total)
except Exception as e:
    print(f"Error: {e}", file=sys.stderr)
    print(0)
JSONEOF
)

    DETECTED_TOTAL_LOST=$(echo "$STATS" | python3 -c "import sys, json; d=json.load(sys.stdin); print(d.get('total_lost_packets', 0))" 2>/dev/null || echo "0")
fi

echo ""
echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}STRESS TEST RESULTS${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "${YELLOW}Test Configuration:${NC}"
echo "  PCAP file:            $PCAP_FILE"
echo "  Input packets:        $ACTUAL_PACKETS"
echo "  Unique flows:         $FLOWS"
echo "  Protocol:             $PROTOCOL"
echo "  Gap injection rate:   ${GAP_RATE} ($(echo "scale=1; $GAP_RATE * 100" | bc)%)"
echo "  Test duration:        ${ELAPSED_SECONDS}s"
echo ""

echo -e "${YELLOW}Results:${NC}"
echo "  Processed packets:    $PROCESSED_PACKETS"
echo "  Unique flows:         $TOTAL_FLOWS"
echo "  Gaps detected:        $TOTAL_GAPS"
echo "  Bandwidth:            ${BANDWIDTH} Mbps"
echo ""

echo -e "${YELLOW}Performance:${NC}"
echo "  Throughput:           $THROUGHPUT pps"
echo "  Per-packet time:      ${PER_PACKET_US} µs"
echo ""

# Analysis
echo -e "${YELLOW}Analysis:${NC}"

LOOPS=$(echo "scale=2; $PROCESSED_PACKETS / $ACTUAL_PACKETS" | bc)
if (( $(echo "$LOOPS < 1.1" | bc -l) )); then
    echo "  Looping:              NO (processed packets once)"
else
    echo "  Looping:              YES (looped $(echo "scale=1; $LOOPS" | bc)x)"
fi

GAP_DETECTION_RATE=$(echo "scale=2; ($TOTAL_GAPS / $PROCESSED_PACKETS) * 100" | bc)
echo "  Gap detection rate:   ${GAP_DETECTION_RATE}%"

if (( $(echo "$THROUGHPUT > 8000" | bc -l) )); then
    echo "  Status:               ${GREEN}Good performance (${THROUGHPUT} pps)${NC}"
elif (( $(echo "$THROUGHPUT > 5000" | bc -l) )); then
    echo "  Status:               ${YELLOW}Moderate (${THROUGHPUT} pps)${NC}"
else
    echo "  Status:               ${RED}Degraded (${THROUGHPUT} pps)${NC}"
fi

echo ""
echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}VALIDATION: Generated vs Detected Lost Packets${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
echo ""

if [ -f "$JSON_FILE" ]; then
    echo -e "${YELLOW}Expected (from PCAP generator):${NC}"
    echo "  MACsec lost packets:   $EXPECTED_LOST_MACSEC"
    echo "  IPsec lost packets:    $EXPECTED_LOST_IPSEC"
    echo "  Total expected:        $((EXPECTED_LOST_MACSEC + EXPECTED_LOST_IPSEC))"
    echo ""

    echo -e "${YELLOW}Detected (from analyzer):${NC}"
    echo "  Total detected:        $DETECTED_TOTAL_LOST"
    echo ""

    # Calculate if they match (with tolerance for protocol filtering)
    EXPECTED_TOTAL=$((EXPECTED_LOST_MACSEC + EXPECTED_LOST_IPSEC))
    if [ "$DETECTED_TOTAL_LOST" = "$EXPECTED_TOTAL" ]; then
        echo -e "${GREEN}✓ MATCH: Generated and detected lost packets are equal${NC}"
    elif [ -z "$DETECTED_TOTAL_LOST" ] || [ "$DETECTED_TOTAL_LOST" = "0" ]; then
        echo -e "${RED}✗ NO MATCH: Analyzer detected 0 lost packets${NC}"
        echo "  Expected: $EXPECTED_TOTAL"
        echo "  Detected: $DETECTED_TOTAL_LOST"
        echo ""
        echo -e "${YELLOW}Possible issues:${NC}"
        echo "  - Analyzer may not be detecting gaps correctly"
        echo "  - Protocol filtering may be excluding packets"
        echo "  - Database may not be storing gap information"
    else
        DIFFERENCE=$((EXPECTED_TOTAL - DETECTED_TOTAL_LOST))
        PERCENTAGE=$(echo "scale=1; ($DIFFERENCE / $EXPECTED_TOTAL) * 100" | bc)
        echo -e "${YELLOW}⚠ MISMATCH: Generated and detected lost packets differ${NC}"
        echo "  Expected: $EXPECTED_TOTAL"
        echo "  Detected: $DETECTED_TOTAL_LOST"
        echo "  Difference: $DIFFERENCE packets ($PERCENTAGE%)"
        echo ""
        echo -e "${YELLOW}Possible issues:${NC}"
        echo "  - Only MACsec gaps are being detected (IPsec gaps filtered)"
        echo "  - Some gaps may not be properly recorded"
        echo "  - Wraparound handling may differ between generator and analyzer"
    fi
else
    echo -e "${RED}✗ JSON file not found: $JSON_FILE${NC}"
    echo "  Gap statistics file was not generated"
fi

echo ""
echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "${GREEN}Artifacts stored in:${NC}"
echo "  PCAP file:        $PCAP_FILE"
echo "  Database:         $DB_FILE"
echo "  Binaries:         $SCRIPT_DIR/{live_analyzer,rest_api_server}"
echo ""

echo -e "${YELLOW}To query results later:${NC}"
echo "  $REST_API_BIN --db $DB_FILE --port 9999"
echo "  curl http://localhost:9999/api/v1/stats/summary"
echo ""

echo -e "${GREEN}✓ Stress test complete${NC}"
