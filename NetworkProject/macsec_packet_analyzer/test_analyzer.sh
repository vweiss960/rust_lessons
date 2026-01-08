#!/bin/bash

##############################################################################
# Test Analyzer Script
#
# This script simplifies testing the live_analyzer and rest_api_server
# by building both executables, cleaning old test databases, and running
# them together pointing to a single test database.
#
# Usage:
#   ./test_analyzer.sh [OPTIONS]
#
# Options (Live Capture):
#   -i, --interface IFACE    Network interface to capture from (default: lo)
#   -d, --database PATH      Path to test database (default: ./test_analysis.db)
#   -p, --port PORT          REST API server port (default: 3000)
#   --debug                  Enable debug output for live_analyzer
#   --release                Build in release mode (default: debug)
#
# Options (PCAP Replay):
#   --replay <mode>          Enable PCAP replay (modes: fast|original|fixed|speed)
#   --replay-pps <rate>      Packets per second (for fixed mode)
#   --replay-speed <mult>    Speed multiplier (for speed mode)
#   --replay-loop            Enable infinite looping during replay
#
#   -h, --help              Show this help message
#
# Examples:
#   ./test_analyzer.sh                              # Run with defaults (live loopback)
#   ./test_analyzer.sh -i eth0 --debug --release   # Use eth0 with debug and release build
#   ./test_analyzer.sh -p 8080 --release           # Use custom port with release build
#   ./test_analyzer.sh --replay fast                # Replay PCAP at maximum speed
#   ./test_analyzer.sh --replay fixed --replay-pps 1000  # Replay at 1000 packets/sec
#   ./test_analyzer.sh --replay speed --replay-speed 10 --replay-loop  # 10x speed with looping
#
##############################################################################

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
INTERFACE="lo"
DATABASE="./test_analysis.db"
PORT="3000"
DEBUG_FLAG=""
BUILD_MODE="debug"
RELEASE_FLAG=""
REPLAY_MODE=""
REPLAY_PPS=""
REPLAY_SPEED=""
REPLAY_LOOP=""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -i|--interface)
            INTERFACE="$2"
            shift 2
            ;;
        -d|--database)
            DATABASE="$2"
            shift 2
            ;;
        -p|--port)
            PORT="$2"
            shift 2
            ;;
        --debug)
            DEBUG_FLAG="--debug"
            shift
            ;;
        --release)
            BUILD_MODE="release"
            RELEASE_FLAG="--release"
            shift
            ;;
        --replay)
            REPLAY_MODE="$2"
            shift 2
            ;;
        --replay-pps)
            REPLAY_PPS="$2"
            shift 2
            ;;
        --replay-speed)
            REPLAY_SPEED="$2"
            shift 2
            ;;
        --replay-loop)
            REPLAY_LOOP="--loop"
            shift
            ;;
        -h|--help)
            head -n 35 "$0" | tail -n +3
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Resolve absolute path for database
DATABASE=$(cd "$(dirname "$DATABASE")" && pwd)/$(basename "$DATABASE")

# Validate replay mode if specified
if [ -n "$REPLAY_MODE" ]; then
    case "$REPLAY_MODE" in
        fast|original|fixed|speed)
            ;;
        *)
            echo -e "${RED}Invalid replay mode: $REPLAY_MODE${NC}"
            echo -e "${YELLOW}Valid modes: fast, original, fixed, speed${NC}"
            exit 1
            ;;
    esac
fi

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
if [ -n "$REPLAY_MODE" ]; then
    echo -e "${BLUE}║     MACsec Packet Analyzer - PCAP Replay Test Suite      ║${NC}"
else
    echo -e "${BLUE}║       MACsec Packet Analyzer - Live Capture Test        ║${NC}"
fi
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Step 1: Clean old databases
echo -e "${YELLOW}[1/4] Cleaning old test databases...${NC}"
rm -f "$DATABASE" "$DATABASE-shm" "$DATABASE-wal" 2>/dev/null || true
rm -f ./test.db ./test.db-* ./analysis.db ./analysis.db-* 2>/dev/null || true
rm -f ./target/$BUILD_MODE/test.db ./target/$BUILD_MODE/analysis.db 2>/dev/null || true
echo -e "${GREEN}✓ Old databases cleaned${NC}"
echo ""

# Step 2: Build live_analyzer
echo -e "${YELLOW}[2/4] Building live_analyzer ($BUILD_MODE mode)...${NC}"
if cargo build --bin live_analyzer $RELEASE_FLAG 2>&1 | tail -5; then
    echo -e "${GREEN}✓ live_analyzer built successfully${NC}"
else
    echo -e "${RED}✗ Failed to build live_analyzer${NC}"
    exit 1
fi
echo ""

# Step 3: Build rest_api_server
echo -e "${YELLOW}[3/4] Building rest_api_server ($BUILD_MODE mode)...${NC}"
if cargo build --bin rest_api_server $RELEASE_FLAG 2>&1 | tail -5; then
    echo -e "${GREEN}✓ rest_api_server built successfully${NC}"
else
    echo -e "${RED}✗ Failed to build rest_api_server${NC}"
    exit 1
fi
echo ""

# Determine binary paths
if [ "$BUILD_MODE" = "release" ]; then
    LIVE_ANALYZER_BIN="./target/release/live_analyzer"
    REST_API_BIN="./target/release/rest_api_server"
else
    LIVE_ANALYZER_BIN="./target/debug/live_analyzer"
    REST_API_BIN="./target/debug/rest_api_server"
fi

# Step 4: Run the servers
echo -e "${YELLOW}[4/4] Starting servers...${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "${GREEN}live_analyzer configuration:${NC}"
echo "  Binary:    $LIVE_ANALYZER_BIN"
if [ -n "$REPLAY_MODE" ]; then
    echo "  Mode:      PCAP Replay"
    echo "  PCAP File: $INTERFACE (treated as PCAP path in replay mode)"
    echo "  Replay:    $REPLAY_MODE"
    [ -n "$REPLAY_PPS" ] && echo "  PPS:       $REPLAY_PPS"
    [ -n "$REPLAY_SPEED" ] && echo "  Speed:     ${REPLAY_SPEED}x"
    [ -n "$REPLAY_LOOP" ] && echo "  Loop:      enabled"
else
    echo "  Mode:      Live Capture"
    echo "  Interface: $INTERFACE"
fi
echo "  Database:  $DATABASE"
echo "  Debug:     ${DEBUG_FLAG:-disabled}"
echo ""

echo -e "${GREEN}rest_api_server configuration:${NC}"
echo "  Binary:    $REST_API_BIN"
echo "  Port:      $PORT"
echo "  Database:  $DATABASE"
echo ""

echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
echo ""

# Function to cleanup on exit
cleanup() {
    echo ""
    echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
    echo -e "${YELLOW}Shutting down servers...${NC}"

    # Kill background jobs
    jobs -p | xargs -r kill 2>/dev/null || true

    echo -e "${GREEN}✓ Servers stopped${NC}"
    echo ""
    echo -e "${GREEN}Database preserved at: $DATABASE${NC}"
    echo -e "${GREEN}Query with: $REST_API_BIN --db $DATABASE --port $PORT${NC}"
}

trap cleanup EXIT

# Start live_analyzer in the background
echo -e "${GREEN}► Starting live_analyzer...${NC}"

# Build the live_analyzer command based on mode
LIVE_CMD="$LIVE_ANALYZER_BIN"
if [ -n "$REPLAY_MODE" ]; then
    # Replay mode: use INTERFACE as PCAP file path
    LIVE_CMD="$LIVE_CMD --replay $INTERFACE $DATABASE --mode $REPLAY_MODE"
    [ -n "$REPLAY_PPS" ] && LIVE_CMD="$LIVE_CMD --pps $REPLAY_PPS"
    [ -n "$REPLAY_SPEED" ] && LIVE_CMD="$LIVE_CMD --speed $REPLAY_SPEED"
    [ -n "$REPLAY_LOOP" ] && LIVE_CMD="$LIVE_CMD $REPLAY_LOOP"
else
    # Live capture mode: INTERFACE is network interface
    LIVE_CMD="$LIVE_CMD $INTERFACE $DATABASE"
fi

# Add debug flag if specified
[ -n "$DEBUG_FLAG" ] && LIVE_CMD="$LIVE_CMD $DEBUG_FLAG"

# Run the command
if [ -n "$DEBUG_FLAG" ]; then
    eval $LIVE_CMD &
else
    eval $LIVE_CMD > /dev/null 2>&1 &
fi
LIVE_PID=$!
echo -e "${GREEN}✓ live_analyzer started (PID: $LIVE_PID)${NC}"
sleep 1

# Start rest_api_server in the background
echo -e "${GREEN}► Starting rest_api_server...${NC}"
$REST_API_BIN --db "$DATABASE" --port $PORT > /dev/null 2>&1 &
REST_PID=$!
echo -e "${GREEN}✓ rest_api_server started (PID: $REST_PID)${NC}"
echo ""

echo -e "${BLUE}════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${GREEN}Servers running!${NC}"
echo ""
echo -e "${YELLOW}API Endpoints:${NC}"
echo "  Health:         http://localhost:$PORT/health"
echo "  Summary Stats:  http://localhost:$PORT/api/v1/stats/summary"
echo "  All Flows:      http://localhost:$PORT/api/v1/flows"
echo "  Specific Flow:  http://localhost:$PORT/api/v1/flows/<flow_id>"
echo "  Flow Gaps:      http://localhost:$PORT/api/v1/flows/<flow_id>/gaps"
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop the servers${NC}"
echo ""

# Wait for both processes
wait
