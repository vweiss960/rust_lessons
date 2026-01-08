#!/bin/bash

################################################################################
# Proper Stress Test Harness for MACsec Analyzer
#
# This script measures ACTUAL packet processing speed, not looping speed.
# It generates synthetic PCAPs and processes them once to measure throughput.
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
OUTPUT_DIR="/tmp/stress_tests"
VERBOSE=false
KEEP_PCAP=false
PORT=9998

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
        --seed)
            SEED="$2"
            shift 2
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --keep-pcap)
            KEEP_PCAP=true
            shift
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

# Ensure directories exist
mkdir -p "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/pcaps"

# Make sure we have required tools
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}Error: python3 not found${NC}"
    exit 1
fi

if ! [ -f ./target/release/live_analyzer ]; then
    echo -e "${RED}Error: ./target/release/live_analyzer not found${NC}"
    echo "Run: cargo build --release"
    exit 1
fi

if ! [ -f ./target/release/rest_api_server ]; then
    echo -e "${RED}Error: ./target/release/rest_api_server not found${NC}"
    echo "Run: cargo build --release"
    exit 1
fi

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  MACsec Analyzer - Proper Stress Test                     ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

echo -e "${GREEN}Configuration:${NC}"
echo "  Packets to generate:  $PACKETS"
echo "  Unique flows:         $FLOWS"
echo "  Gap rate:             ${GAP_RATE} ($(echo "scale=1; $GAP_RATE * 100" | bc)%)"
echo "  Test duration:        ${DURATION}s"
echo "  Output directory:     $OUTPUT_DIR"
echo ""

# ============================================================================
# Step 1: Generate synthetic PCAP
# ============================================================================

PCAP_FILE="$OUTPUT_DIR/pcaps/stress_${PACKETS}pkt_${FLOWS}flows.pcap"
DB_FILE="$OUTPUT_DIR/dbs/stress_${PACKETS}pkt_${FLOWS}flows.db"

mkdir -p "$OUTPUT_DIR/dbs"

# Only regenerate if it doesn't exist
if [ ! -f "$PCAP_FILE" ]; then
    echo -e "${YELLOW}[1/4] Generating synthetic PCAP...${NC}"

    python3 - <<EOF > /dev/null
import subprocess
import sys

args = [
    'python3', '$(dirname "$0")/pcap_generator.py',
    '--packets', '$PACKETS',
    '--flows', '$FLOWS',
    '--gap-rate', '$GAP_RATE',
    '--seed', '$SEED',
    '--output', '$PCAP_FILE',
]

if '$VERBOSE' == 'true':
    args.append('--verbose')

result = subprocess.run(args, capture_output=False)
sys.exit(result.returncode)
EOF

    if [ $? -ne 0 ]; then
        echo -e "${RED}✗ Failed to generate PCAP${NC}"
        exit 1
    fi

    PCAP_SIZE=$(du -h "$PCAP_FILE" | cut -f1)
    echo -e "${GREEN}✓ Generated PCAP: $PCAP_FILE ($PCAP_SIZE)${NC}"
else
    echo -e "${GREEN}[1/4] Using existing PCAP: $(du -h "$PCAP_FILE" | cut -f1)${NC}"
fi
echo ""

# ============================================================================
# Step 2: Verify packet count in PCAP
# ============================================================================

echo -e "${YELLOW}[2/4] Verifying packet count...${NC}"

ACTUAL_PACKETS=$(python3 - <<'PYEOF'
import struct
import sys

try:
    with open(''"$PCAP_FILE"'', 'rb') as f:
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
# Step 3: Clean and run analyzer
# ============================================================================

echo -e "${YELLOW}[3/4] Running analyzer...${NC}"

# Clean database
rm -f "$DB_FILE" "${DB_FILE}-shm" "${DB_FILE}-wal"

# Run analyzer with timeout
echo "Command: ./target/release/live_analyzer $PCAP_FILE $DB_FILE --replay --mode fast"
echo ""

ANALYZE_START=$(date +%s%3N)

if [ "$VERBOSE" = "true" ]; then
    timeout ${DURATION} ./target/release/live_analyzer \
        "$PCAP_FILE" "$DB_FILE" --replay --mode fast --debug 2>&1 | head -50 &
    ANALYZER_PID=$!
else
    timeout ${DURATION} ./target/release/live_analyzer \
        "$PCAP_FILE" "$DB_FILE" --replay --mode fast > /dev/null 2>&1 &
    ANALYZER_PID=$!
fi

# Show progress dots
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
# Step 4: Query results
# ============================================================================

echo -e "${YELLOW}[4/4] Analyzing results...${NC}"

# Increase port to avoid conflicts
PORT=$((9998 + RANDOM % 100))

# Start REST API server
./target/release/rest_api_server --db "$DB_FILE" --port $PORT > /dev/null 2>&1 &
REST_PID=$!

# Give it time to start
sleep 1

# Query stats
STATS=$(curl -s http://localhost:$PORT/api/v1/stats/summary)

# Kill REST server
kill $REST_PID 2>/dev/null || true
wait $REST_PID 2>/dev/null || true

# Extract fields (handle jq failures gracefully)
PROCESSED_PACKETS=$(echo "$STATS" | jq -r '.total_packets_received // "0"' 2>/dev/null || echo "0")
TOTAL_GAPS=$(echo "$STATS" | jq -r '.total_gaps // "0"' 2>/dev/null || echo "0")
TOTAL_FLOWS=$(echo "$STATS" | jq -r '.total_flows // "0"' 2>/dev/null || echo "0")
BANDWIDTH=$(echo "$STATS" | jq -r '.avg_bandwidth_mbps // "0"' 2>/dev/null || echo "0")

if [ "$PROCESSED_PACKETS" = "0" ] || [ -z "$PROCESSED_PACKETS" ]; then
    echo -e "${RED}✗ Failed to query results from database${NC}"
    echo "Database file: $DB_FILE"
    echo "Stats response: $STATS"
    exit 1
fi

# Calculate metrics
THROUGHPUT=$(echo "scale=1; $PROCESSED_PACKETS / $ELAPSED_SECONDS" | bc)
PER_PACKET_US=$(echo "scale=2; 1000000 / $THROUGHPUT" | bc)

echo ""
echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}STRESS TEST RESULTS${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "${YELLOW}Test Configuration:${NC}"
echo "  PCAP file:            $PCAP_FILE"
echo "  Input packets:        $ACTUAL_PACKETS"
echo "  Unique flows:         $FLOWS"
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

# ============================================================================
# Save results
# ============================================================================

RESULTS_CSV="$OUTPUT_DIR/results.csv"

# Create header if file doesn't exist
if [ ! -f "$RESULTS_CSV" ]; then
    echo "timestamp,packets,flows,duration_s,throughput_pps,per_packet_us,gaps_detected,bandwidth_mbps,looping" > "$RESULTS_CSV"
fi

# Append results
TIMESTAMP=$(date -u +"%Y-%m-%d %H:%M:%S")
echo "$TIMESTAMP,$ACTUAL_PACKETS,$TOTAL_FLOWS,$ELAPSED_SECONDS,$THROUGHPUT,$PER_PACKET_US,$TOTAL_GAPS,$BANDWIDTH,$LOOPS" >> "$RESULTS_CSV"

echo -e "${GREEN}✓ Results saved to: $RESULTS_CSV${NC}"

# Display summary across all tests
echo ""
echo -e "${YELLOW}All Test Results:${NC}"
cat "$RESULTS_CSV" | column -t -s, | tail -5

# Database preserved
echo ""
echo -e "${GREEN}Database preserved at: $DB_FILE${NC}"
if ! [ "$KEEP_PCAP" = "true" ]; then
    echo -e "${YELLOW}(PCAP will be reused for subsequent runs)${NC}"
fi

echo ""
echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
