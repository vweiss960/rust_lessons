# REST API Server Configuration - Quick Reference

## Simplest Usage

```bash
# Use defaults (analysis.db, port 3000)
cargo run --features rest-api --bin rest_api_server
```

## Override Database

```bash
# Via command line
cargo run --features rest-api --bin rest_api_server -- --db my_results.db

# Via config file
cat > config.json << 'EOF'
{
  "database": {"path": "my_results.db"},
  "server": {"port": 3000}
}
EOF
cargo run --features rest-api --bin rest_api_server
```

## Override Port

```bash
cargo run --features rest-api --bin rest_api_server -- --port 8080
```

## Use Custom Config File

```bash
cargo run --features rest-api --bin rest_api_server -- --config production.json
```

## All Options Together

```bash
cargo run --features rest-api --bin rest_api_server -- \
  --db ./results/analysis.db \
  --port 8080 \
  --host 0.0.0.0
```

## Configuration File (config.json)

```json
{
  "database": {
    "path": "analysis.db"
  },
  "server": {
    "host": "127.0.0.1",
    "port": 3000
  }
}
```

## Command-Line Options

| Option | Example | Purpose |
|--------|---------|---------|
| `--db <PATH>` | `--db ./data.db` | Specify database path |
| `--port <NUM>` | `--port 8080` | Specify port |
| `--host <HOST>` | `--host 0.0.0.0` | Specify host/IP |
| `--config <PATH>` | `--config prod.json` | Load config file |
| `--help` | `--help` | Show help |

## Workflow: Capture & Query

```bash
# Terminal 1: Capture to live.db
sudo cargo run --bin live_analyzer -- eth0 generic live.db pcap

# Terminal 2: Query with REST API pointing to live.db
cargo run --bin rest_api_server -- --db live.db

# Terminal 3: Use the API
curl http://localhost:3000/api/v1/stats/summary | jq .
```

## Configuration Priority (Lowest → Highest)

1. **Built-in defaults**: `analysis.db`, port `3000`, host `127.0.0.1`
2. **config.json file**: Overrides defaults
3. **CLI arguments**: Override everything

For detailed configuration guide, see [REST_API_CONFIG.md](REST_API_CONFIG.md)
