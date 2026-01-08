# REST API Server Configuration

The REST API server can be configured using a JSON configuration file and/or command-line arguments.

## Quick Start

### Option 1: Use Defaults (Simplest)

```bash
cargo run --features rest-api --bin rest_api_server
```

This starts the server on `http://127.0.0.1:3000` using `analysis.db` as the database.

### Option 2: Override Database Path via CLI

```bash
cargo run --features rest-api --bin rest_api_server -- --db ./my_results.db
```

### Option 3: Use Configuration File

Create a `config.json` file in your current directory:

```json
{
  "database": {
    "path": "my_analysis.db"
  },
  "server": {
    "host": "127.0.0.1",
    "port": 8080
  }
}
```

Then run:

```bash
cargo run --features rest-api --bin rest_api_server
```

## Configuration File Format

The configuration file must be valid JSON with the following structure:

```json
{
  "database": {
    "path": "path/to/database.db"
  },
  "server": {
    "host": "127.0.0.1",
    "port": 3000
  }
}
```

### Configuration Fields

#### `database`
- **`path`** (string): Path to the SQLite database file
  - Default: `"analysis.db"`
  - Supports relative and absolute paths
  - Example: `"./capture/results.db"`, `"/data/analysis.db"`

#### `server`
- **`host`** (string): IP address to bind to
  - Default: `"127.0.0.1"` (localhost only)
  - Use `"0.0.0.0"` to listen on all interfaces
- **`port`** (number): Port to listen on
  - Default: `3000`
  - Must be between 1 and 65535

## Command-Line Arguments

You can override configuration file settings with command-line arguments:

### `--config <PATH>`
Load configuration from a specific JSON file:
```bash
./rest_api_server --config /etc/app/config.json
```

### `--db <PATH>` or `--database <PATH>`
Override the database path:
```bash
./rest_api_server --db ./results/analysis.db
```

### `--port <NUM>`
Override the server port:
```bash
./rest_api_server --port 8080
```

### `--host <HOST>`
Override the server host:
```bash
./rest_api_server --host 0.0.0.0
```

### `--help` or `-h`
Show help message:
```bash
./rest_api_server --help
```

## Configuration Priority

Configuration is applied in this order (later overrides earlier):

1. **Built-in defaults**
   - Database: `analysis.db`
   - Port: `3000`
   - Host: `127.0.0.1`

2. **Configuration file** (`config.json`)
   - Loaded if it exists in the current directory
   - Can be overridden with `--config`

3. **Command-line arguments**
   - Have the highest priority
   - Override both defaults and config file

### Example Priority Order

```bash
# 1. Defaults apply (db: analysis.db, port: 3000)
./rest_api_server

# 2. Config file overrides defaults
./rest_api_server
# (uses settings from config.json)

# 3. CLI args override config file
./rest_api_server --db custom.db --port 8080
# (uses custom.db from DB, port 8080, host from config.json)

# 4. Specific config file + CLI overrides
./rest_api_server --config production.json --port 9000
# (loads production.json, but uses port 9000)
```

## Common Use Cases

### Development Setup

`config.json`:
```json
{
  "database": {
    "path": "./local_analysis.db"
  },
  "server": {
    "host": "127.0.0.1",
    "port": 3000
  }
}
```

Run:
```bash
cargo run --features rest-api --bin rest_api_server
```

### Production Setup

`config.json`:
```json
{
  "database": {
    "path": "/var/lib/analysis/data.db"
  },
  "server": {
    "host": "0.0.0.0",
    "port": 8080
  }
}
```

Run:
```bash
./rest_api_server --config config.json
```

### Multiple Deployments

Keep separate config files for different environments:

- `config.dev.json` - Development settings
- `config.staging.json` - Staging settings
- `config.prod.json` - Production settings

```bash
# Development
./rest_api_server --config config.dev.json

# Staging
./rest_api_server --config config.staging.json

# Production
./rest_api_server --config config.prod.json
```

### Temporary Override

Use different database for testing without changing config:

```bash
# Normal operation uses config.json settings
./rest_api_server

# Temporary override for testing
./rest_api_server --db test_analysis.db --port 9000
```

## Configuration File Location

The server looks for `config.json` in the current working directory by default.

### Custom Location

Use `--config` to specify a different path:

```bash
# Absolute path
./rest_api_server --config /etc/myapp/config.json

# Relative path
./rest_api_server --config ./config/production.json

# Environment variable (you need to expand it yourself)
./rest_api_server --config $CONFIG_PATH
```

## Example Workflows

### Capture and Analyze

```bash
# Terminal 1: Capture traffic to live.db (auto-detects protocol)
sudo cargo run --bin live_analyzer -- eth0 live.db pcap

# Terminal 2: Start API server pointing to capture
cargo run --bin rest_api_server -- --db live.db

# Terminal 3: Query the API
curl http://localhost:3000/api/v1/stats/summary | jq .
```

### Multiple Captures

```bash
# Capture 1 (auto-detects protocol on eth0)
sudo cargo run --bin live_analyzer -- eth0 capture1.db pcap

# Capture 2 (auto-detects protocol on eth1)
sudo cargo run --bin live_analyzer -- eth1 capture2.db pcap

# Query Capture 1
cargo run --bin rest_api_server -- --db capture1.db --port 3000 &

# Query Capture 2
cargo run --bin rest_api_server -- --db capture2.db --port 3001 &
```

### Remote Database

If your database is on a network drive:

```bash
./rest_api_server --db /mnt/shared/analysis.db
```

Or in config:

```json
{
  "database": {
    "path": "/mnt/shared/analysis.db"
  }
}
```

## Troubleshooting

### "Database not found" Error

Ensure the database path is correct:

```bash
# Check database exists
ls -lh analysis.db

# Check with different path
./rest_api_server --db ./path/to/database.db
```

### "Port already in use" Error

Use a different port:

```bash
./rest_api_server --port 8080
```

Or kill the process using the port:

```bash
# Find process on port 3000
lsof -i :3000

# Kill it
kill <PID>
```

### "Cannot bind to address" Error

If using `--host 0.0.0.0`, ensure you have permission:

```bash
# As root
sudo ./rest_api_server --host 0.0.0.0
```

Or use a different interface:

```bash
./rest_api_server --host 192.168.1.100 --port 8080
```

## Environment Variables (Advanced)

While not directly supported, you can use environment variables in shell commands:

```bash
# Set variables
export DB_PATH="./results.db"
export API_PORT="8080"

# Use them in the command
./rest_api_server --db "$DB_PATH" --port "$API_PORT"
```

Or create a shell wrapper:

```bash
#!/bin/bash
DB_PATH="${REST_API_DB_PATH:-./analysis.db}"
PORT="${REST_API_PORT:-3000}"
HOST="${REST_API_HOST:-127.0.0.1}"

exec ./rest_api_server --db "$DB_PATH" --port "$PORT" --host "$HOST"
```

## Default Configuration Template

Copy `config.json.example` to `config.json` to get started:

```bash
cp config.json.example config.json
# Edit config.json as needed
./rest_api_server
```
